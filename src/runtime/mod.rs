use crate::event::{Emission};
use crate::context::{Context, ContextMsg, ContextReceiver};
use crate::pipe::{Pipe, RxMsg, TxMsg};
use crate::runtime::service_runner::{ServiceCollection, ServiceMessage};
use crate::vdom::{Differ, patch_serialize, Patch};
use crate::{Id, App};
use crate::platform::{spawn, spawn_blocking};
use async_timer::Interval;
use futures::channel::mpsc::{unbounded, UnboundedReceiver, UnboundedSender};
use futures::{select, FutureExt, StreamExt};
use std::collections::{VecDeque, HashSet};
use crate::runtime::state::RenderedState;
pub(crate) use crate::runtime::render::RenderResult;
pub(crate) use crate::runtime::state::Frame;
use crate::runtime::metrics::Metrics;
use std::time::{Instant, Duration};
use crate::dialog::DialogBinding;
use futures::SinkExt;

mod service_runner;
mod render;
mod metrics;
mod component;
mod state;

/// Wait time from an update() and a subsequent render
/// Defines the maximum frame rate.
const DEFAULT_RENDER_INTERVAL_MS: u64 = 30;

/// applies in case a render is still in progress on the frontend
/// but a render is scheduled in the runtime
const RENDER_RETRY_INTERVAL_MS: u64 = 10;


/// `RuntimeControl` objects are used to control a `Runtime`, which in turn manages a user-defined
/// application (implementing [`App`](../component/trait.App.html)).
#[derive(Clone)]
pub struct RuntimeControl<A: App> {
    tx: UnboundedSender<RuntimeMsg<A>>,
}

impl<A: App> RuntimeControl<A> {
    /// Quits the event loop of the `Runtime`.
    ///
    /// As a result, the `Runtime::run()` future resolves and the `Runtime::run_blocking()` returns.
    pub fn quit(&self) {
        self.tx.unbounded_send(RuntimeMsg::Quit).unwrap();
    }

    /// Sends a message into the update cycle of the application.
    pub fn update(&self, msg: A::Message) {
        self.tx.unbounded_send(RuntimeMsg::Update(msg)).unwrap();
    }
}

/// Message passed to the runtime from different actors (and/or threads) to modify its state
enum RuntimeMsg<A: App> {
    Quit,
    Update(A::Message),
    ApplyNextFrame(Frame<A>, Duration),
    NextFrameRendering(Frame<A>, Duration),
    AsyncMsg(A::Message),
}


/// The `Runtime` object manages the main application life-cycle as well as event distribution.
/// It is the central object executing the backend portion of an application.
///
/// A `Runtime` owns a type implementing `Pipe` which allows it to communicate with the frontend
/// part of the application.
/// Paired to a runtime, there is a set of `Clone`-able and `Send`-able
/// [`RuntimeControl`](struct.RuntimeControl.html) objects.
/// They are used to update the `App` underlying the `Runtime` or to affect its lifecycle.
///
/// # Example
///
/// ```
/// use greenhorn::prelude::*;
/// use greenhorn::html;
/// # use std::net::SocketAddr;
/// # use std::str::FromStr;
///
/// struct MyApp {
///     my_app_state: u32
/// }
///
/// impl Render for  MyApp {
///    type Message = ();
///
///    fn render(&self) -> Node<Self::Message> {
///         html!(
///             <div .app>
///                 {format!("Hello, World: {}", self.my_app_state)}
///             </>
///         ).into()
///     }
/// }
///
/// impl App for MyApp {
///     fn update(&mut self,msg: Self::Message,ctx: Context<Self::Message>) -> Updated {
///         self.my_app_state += 1;
///         Updated::yes()
///     }
///
///     fn mount(&mut self, ctx: Context<Self::Message>) {
///         // perform tasks on application startup
/// #       ctx.quit(); // such that doctest exits
///     }
/// }
///
///let app = MyApp { my_app_state: 123 };
///let pipe =  WebSocketPipe::listen_to_addr(SocketAddr::from_str("127.0.0.1:1234").unwrap());
///let (runtime, control) = Runtime::new(app, pipe);
///runtime.run_blocking();
/// ```
///
/// To execute the runtime, it features to methods, the async `run` function and the `run_blocking`
/// function. Both return a [`Metrics`](struct.Metrics.html) object which provides performance
/// data of the executed application.
///
pub struct Runtime<A: 'static + App, P: 'static + Pipe> {
    tx: UnboundedSender<RuntimeMsg<A>>,
    rx: UnboundedReceiver<RuntimeMsg<A>>,
    app: A,
    sender: P::Sender,
    receiver: P::Receiver,
    event_queue: VecDeque<Emission>,
    rendered: RenderedState<A>,
    current_frame: Option<Frame<A>>,
    next_frame: Option<Frame<A>>,
    services: ServiceCollection<A::Message>,
    render_tx: UnboundedSender<()>,
    render_rx: UnboundedReceiver<()>,
    invalidated_components: Option<HashSet<Id>>,
    invalidate_all: bool,
    not_applied_counter: i32,
    dirty: bool,
    metrics: Metrics,
    dialogs: VecDeque<DialogBinding<A::Message>>
}

impl<A: 'static + App, P: 'static + Pipe> Runtime<A, P> {

    /// Create a new `Runtime`, which allows executing a given Application.
    /// Also returns an associated control object, which allows controlling the runtime and changing
    /// the state of the application.
    pub fn new(app: A, pipe: P) -> (Runtime<A, P>, RuntimeControl<A>) {
        let (tx, rx) = unbounded();
        let (sender, receiver) = pipe.split();
        let (render_tx, render_rx) = unbounded();
        let runtime = Runtime {
            tx: tx.clone(),
            rx,
            app,
            sender,
            receiver,
            event_queue: VecDeque::new(),
            rendered: RenderedState::new(),
            services: ServiceCollection::new(),
            render_tx,
            render_rx,
            invalidated_components: Some(HashSet::new()),
            next_frame: None,
            dirty: false,
            invalidate_all: false,
            current_frame: None,
            not_applied_counter: 0,
            metrics: Default::default(),
            dialogs: Default::default(),
        };
        let control = RuntimeControl { tx };
        (runtime, control)
    }

    /// Async runs this application and returns the collected
    /// performance metrics upon completion.
    pub async fn run(mut self) -> Metrics {
        // schedule a first render, but wait a few milliseconds in case some
        // startup services decide to update the application state immediately
        self.schedule_render(DEFAULT_RENDER_INTERVAL_MS);
        let (ctx, receiver) = Context::<A::Message>::new();
        self.app.mount(ctx);
        self.handle_context_result(receiver).await;
        loop {
            select! {
                _ = self.render_rx.next().fuse() => {
                    self.dirty = false;
                    self.render_dom()
                },
                msg = self.receiver.next().fuse() => {
                    if let Some(msg) = msg {
                        if !self.handle_frontend_msg(msg).await {
                            break;
                        }
                    } else {
                        break;
                    }
                },
                msg = self.rx.next().fuse() => {
                    if let Some(msg) = msg {
                        if !self.handle_runtime_msg(msg).await {
                            break;
                        }
                    } else {
                        break;
                    }
                },
                msg = self.services.next().fuse() => {
                    if let Some(msg) = msg {
                        self.handle_service_msg(msg).await;
                    }
                }
            }
        }
        self.metrics
    }

    /// Execute the application. This function blocks until the application exits.
    ///
    /// Returns: The performance metrics collected during exeuction of the application
    #[cfg(not(target_arch = "wasm32"))]
    pub fn run_blocking(self) -> Metrics {
        async_std::task::block_on(self.run())
    }

    /// Handle messages as received from services
    async fn handle_service_msg(&mut self, msg: ServiceMessage<A::Message>) {
        match msg {
            ServiceMessage::Update(msg) => self.update(msg).await,
            ServiceMessage::Tx(id, msg) => {
                self.sender.send(TxMsg::Service(id.data(), msg)).await.unwrap();
            },
            ServiceMessage::Stopped() => {}
        }
    }

    /// Handles a message received from the frontend
    async fn handle_frontend_msg(&mut self, msg: RxMsg) -> bool {
        match msg {
            RxMsg::Event(evt) => {
                // search in listeners and get a message
                let msg = self.rendered
                    .get_listener(evt.target(), evt.name())
                    .map(|listener| listener.call(evt));

                // inject the message back into the app
                if let Some(msg) = msg {
                    self.update(msg).await;
                    self.process_events().await;
                }
            }
            RxMsg::FrameApplied() => {
                // a patch was applied by the frontend
                // thus we swap the current state to the newly rendered frame
                if let Some(frame) = self.next_frame.take() {
                    self.rendered.apply(&frame);
                    self.current_frame = Some(frame);
                }
            }
            RxMsg::Service(id, msg) => {
                self.services.send(Id::new_from_data(id), msg);
            },
            RxMsg::Dialog(data) => {
                // cannot receive a dialog message if no dialog is active
                // since this is the only place where we pop
                let dialog = self.dialogs.pop_front().unwrap();
                // panic if data was ill formated since that is a bug in the backend
                let msg = dialog.resolve(data).unwrap();
                self.update(msg).await;
                self.process_events().await;
                if !self.dialogs.is_empty() {
                    // show next dialog
                    let data = self.dialogs.get(0).unwrap().serialize();
                    self.sender.send(TxMsg::Dialog(data)).await.unwrap();
                }
            }
            RxMsg::ElementRpc(id,  value) => {
                let id = Id::new_from_data(id);
                let msg = self.rendered.get_rpc(id)
                    .map(|rpc| rpc.call(value));
                if let Some(msg) = msg {
                    self.update(msg).await;
                    self.process_events().await;
                }
            }
        };
        true
    }

    /// Processes a message as received by the runtime control handle.
    async fn handle_runtime_msg(&mut self, msg: RuntimeMsg<A>) -> bool {
        match msg {
            RuntimeMsg::Quit => {return false;},
            RuntimeMsg::Update(msg) => {
                self.update(msg).await;
            }
            RuntimeMsg::ApplyNextFrame(frame, duration) => {
                self.next_frame = None;
                self.rendered.apply(&frame);
                self.current_frame = Some(frame);
                self.metrics.empty_patch.record(duration);
            }
            RuntimeMsg::NextFrameRendering(frame, duration) => {
                // schedule next frame
                self.next_frame = Some(frame);
                self.metrics.diff.record(duration);
            },
            RuntimeMsg::AsyncMsg(msg) => {
                self.update(msg).await;
            }
        }
        true
    }

    /// Schedules a render of the application.
    ///
    /// Rendering happens with at most a certain period. Once an `update()` was issued
    /// the runtime checks if the application requires re-rendering. If yes, the
    /// runtime sets a flag and starts a timer. Once this timer has expired, the rendering happens.
    ///
    /// This debouncing is used to limit the maximum frame rate and thus maximum CPU usage
    /// of the application.
    fn schedule_render(&mut self, wait_time: u64) {
        if self.dirty {
            return;
        }
        let render_tx = self.render_tx.clone();
        spawn(async move {
            let mut timer = Interval::platform_new(core::time::Duration::from_millis(wait_time));
            timer.as_mut().await;
            timer.cancel();
            let _ = render_tx.unbounded_send(());
        });
        self.dirty = true;
    }

    /// Inserts a message into the update loop of the application.
    async fn update(&mut self, msg: A::Message) {
        let (ctx, receiver) = Context::<A::Message>::new();
        let updated = self.app.update(msg, ctx);
        if updated.should_render {
            self.invalidate_all = true;
            self.schedule_render(DEFAULT_RENDER_INTERVAL_MS);
        } else if let Some(invalidated) = updated.components_render {
            let invalidated_components = self.invalidated_components.as_mut().unwrap();
            invalidated.iter().for_each(|x| { invalidated_components.insert(*x); });
            self.schedule_render(DEFAULT_RENDER_INTERVAL_MS);
        };
        self.handle_context_result(receiver).await;
    }

    /// Handles the result of calling a function with a `Context`. The `Context` may be used by
    /// components to interface with the current application state and to execute global services
    /// such as dialogs, running futures, streams, ...
    async fn handle_context_result(&mut self, receiver: ContextReceiver<A::Message>) {
        // TODO: refactor ContextReceiver to Vec<> since not async anymore
        while let Ok(cmd) = receiver.rx.try_recv() {
            match cmd {
                ContextMsg::Emission(e) => {
                    self.event_queue.push_back(e);
                },
                ContextMsg::LoadCss(css) => {
                    self.sender.send(TxMsg::LoadCss(css)).await.unwrap();
                },
                ContextMsg::RunJs(js) => {
                    self.sender.send(TxMsg::RunJs(js)).await.unwrap();
                },
                ContextMsg::Propagate(prop) => {
                    self.sender.send(TxMsg::Propagate(prop)).await.unwrap();
                },
                ContextMsg::Subscription(service) => {
                    self.services.spawn(service);
                }
                ContextMsg::Future(fut, blocking) => {
                    let tx = self.tx.clone();
                    if blocking {
                        spawn_blocking(async move {
                            let result = fut.await;
                            let _ = tx.unbounded_send(RuntimeMsg::AsyncMsg(result));
                        });
                    } else {
                        spawn(async move {
                            let result = fut.await;
                            let _ = tx.unbounded_send(RuntimeMsg::AsyncMsg(result));
                        });
                    }
                }
                ContextMsg::Stream(mut stream) => {
                    let tx = self.tx.clone();
                    spawn(async move {
                        while let Some(value) = stream.next().await {
                            let _ = tx.unbounded_send(RuntimeMsg::AsyncMsg(value));
                        }
                    });
                }
                ContextMsg::Dialog(dialog) => {
                    if self.dialogs.is_empty() {
                        self.sender.send(TxMsg::Dialog(dialog.serialize())).await.unwrap();
                    }
                    self.dialogs.push_back(dialog);
                }
                ContextMsg::Quit => {
                    self.tx.unbounded_send(RuntimeMsg::Quit).unwrap();
                }
            }
        }
    }

    /// Processes all events in the event queue, retrieves subscriptions and updates the app-state
    /// accordingly.
    async fn process_events(&mut self) {
        while let Some(evt) = self.event_queue.pop_front() {
            let msg = self.rendered
                .get_subscription(evt.event_id)
                .map(|subs| subs.call(evt.data));
            if let Some(msg) = msg {
                self.update(msg).await;
            }
        }
    }

    /// This function manages rendering and DOM diffing. Its invocation may be scheduled by calling
    /// `self.schedule_render()`.
    ///
    /// In case the a render was not yet processed by the frontend, this function delays the rendering
    /// operation several time to avoid overloading the frontend process.
    fn render_dom(&mut self) {
        if self.next_frame.is_some() && self.current_frame.is_none() && self.not_applied_counter < 3 {
            self.not_applied_counter += 1;
            self.dirty = false;
            self.schedule_render(RENDER_RETRY_INTERVAL_MS);
            return;
        }
        self.not_applied_counter = 0;
        let old_frame = self.current_frame.take();

        let metrics = &mut self.metrics;
        let app = &mut self.app;
        let dom = metrics.root.run(|| app.render() );

        let updated = self.invalidated_components.take().unwrap();
        self.invalidated_components = Some(HashSet::new());

        let result = if self.invalidate_all {
            RenderResult::new_from_root(dom, &mut self.metrics)
        } else if let Some(old_frame) = &old_frame {
            RenderResult::new_from_frame(old_frame, &updated, &mut self.metrics)
        } else {
            RenderResult::new_from_root(dom, &mut self.metrics)
        };
        self.invalidate_all = false;
        self.dirty = false;
        let tx = self.tx.clone();
        let mut sender = self.sender.clone();

        spawn_blocking(async move {
            // create a patch
            let before = Instant::now();
            let patch= if let Some(old_frame) = &old_frame {
                Differ::new(&old_frame, &result, updated).diff()
            } else {
                Patch::from_dom(&result)
            };
            let after = Instant::now();
            let delta = after.duration_since(before);

            if patch.is_empty() {
                let translations = patch.translations;
                let frame = Frame::new(result, translations);
                let _ = tx.unbounded_send(RuntimeMsg::ApplyNextFrame(frame, delta));
            } else {
                let serialized = patch_serialize(&result, &patch);
                let translations = patch.translations;
                let frame = Frame::new(result, translations);
                let _ = tx.unbounded_send(RuntimeMsg::NextFrameRendering(frame, delta));
                // serialize the patch and send it to the client
                sender.send(TxMsg::Patch(serialized)).await.unwrap();
            }
        });
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Render, Updated};
    use crate::node::Node;
    use crate::pipe::tests::DummyPipe;
    use async_std::task::{block_on, spawn_blocking};
    use futures::stream::StreamExt;
    use crate::vdom::{VNode, VElement, PatchItem, Patch};
    use crate::vdom::Attr;

    struct DummyComponent(u32);
    impl Render for DummyComponent {
        type Message = ();

        fn render(&self) -> Node<Self::Message> {
            Node::html()
                .elem("div")
                .id("html-id")
                .add(Node::text(self.0.to_string()) )
                .build()
        }
    }

    impl App for DummyComponent {
        fn update(&mut self, _msg: Self::Message, _ctx: Context<Self::Message>) -> Updated {
            self.0 += 1;
            Updated::yes()
        }
    }

    fn make_patch(patch: Vec<PatchItem>) -> Vec<u8> {
        let patch = Patch {
            items: patch,
            translations: Default::default()
        };
        let render_result = RenderResult::<DummyComponent>::new_empty();
        patch_serialize(&render_result, &patch)
    }

    #[test]
    fn test_empty_render() {
        let app = DummyComponent(1);
        let (pipe, mut frontend) = DummyPipe::new();
        let (rt, _control) = Runtime::new(app, pipe);
        let handle = spawn_blocking(move || {
            match block_on(frontend.sender_rx.next()) {
                Some(TxMsg::Patch(msg)) => {
                    let elem = VNode::element(VElement {
                        id: Id::new_empty(),
                        tag: "div".to_string(),
                        attr: vec![Attr::new("id", "html-id")],
                        js_events: vec![],
                        events: vec![],
                        children: vec![VNode::Text("1".to_string())],
                        namespace: None
                    });
                    let serialized = make_patch(vec![PatchItem::Replace(&elem)]);
                    assert_eq!(serialized, msg);
                },
                _ => panic!()
            }
        });
        rt.run_blocking();
        block_on(handle);
    }

    #[test]
    fn test_empty_render_plus_update() {
        let app = DummyComponent(1);
        let (pipe, mut frontend) = DummyPipe::new();
        let (rt, control) = Runtime::new(app, pipe);
        let handle = spawn_blocking(move || {
            let _ = block_on(frontend.sender_rx.next()).unwrap();
            control.update(());
            block_on(frontend.receiver_tx.send(RxMsg::FrameApplied())).unwrap();
            let msg2 = block_on(frontend.sender_rx.next());
            match msg2 {
                Some(TxMsg::Patch(msg)) => {
                    let new_text = "2";
                    let serialized = make_patch(vec![PatchItem::Descend(), PatchItem::ChangeText(&new_text)]);
                    assert_eq!(serialized, msg);
                },
                _ => panic!()
            }

        });
        rt.run_blocking();
        block_on(handle);
    }

    #[test]
    fn test_rerender_if_timeout() {
        let app = DummyComponent(1);
        let (pipe, mut frontend) = DummyPipe::new();
        let (rt, control) = Runtime::new(app, pipe);
        let handle = spawn_blocking(move || {
            let _ = block_on(frontend.sender_rx.next()).unwrap();
            control.update(());
            // don't do this now
            // task::block_on(frontend.receiver_tx.send(RxMsg::FrameApplied())).unwrap();
            let msg2 = block_on(frontend.sender_rx.next());

            let elem = VNode::element(VElement {
                id: Id::new_empty(),
                tag: "div".to_string(),
                attr: vec![Attr::new("id", "html-id")],
                js_events: vec![],
                events: vec![],
                children: vec![VNode::Text("2".to_string())],
                namespace: None
            });
            let serialized = make_patch(vec![PatchItem::Replace(&elem)]);

            match msg2 {
                Some(TxMsg::Patch(msg)) => {
                    assert_eq!(msg, serialized);
                },
                _ => panic!(),
            }
        });
        rt.run_blocking();
        block_on(handle);
    }

    #[test]
    fn test() {

    }

}
