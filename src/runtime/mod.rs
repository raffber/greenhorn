use crate::component::App;
use crate::event::{Emission};
use crate::context::{Context, ContextMsg, MailboxReceiver};
use crate::pipe::Sender;
use crate::pipe::{Pipe, RxMsg, TxMsg};
use crate::runtime::service_runner::{ServiceCollection, ServiceMessage};
use crate::vdom::{Differ, patch_serialize, Patch};
use crate::Id;
use async_std::task;
use async_timer::Interval;
use futures::channel::mpsc::{unbounded, UnboundedReceiver, UnboundedSender};
use futures::{select, FutureExt, StreamExt};
use std::collections::{HashMap, VecDeque, HashSet};
use crate::component::{ComponentMap, ComponentContainer};
use crate::runtime::state::RenderedState;
pub(crate) use crate::runtime::render::RenderResult;
pub(crate) use crate::runtime::state::Frame;
use crate::runtime::metrics::Metrics;
use std::time::{Instant, Duration};
use crate::dialog::DialogBinding;

mod service_runner;
mod render;
mod metrics;
mod component;
mod state;

const DEFAULT_RENDER_INTERVAL: u64 = 30;
const RENDER_RETRY_INTERVAL: u64 = 10;

enum PendingEvent {
    Component(Emission),
}

struct ComponentDom<M: 'static> {
    component: Box<dyn ComponentMap<M>>,
}

pub struct Runtime<A: 'static + App, P: 'static + Pipe> {
    tx: UnboundedSender<RuntimeMsg<A>>,
    rx: UnboundedReceiver<RuntimeMsg<A>>,
    app: A,
    sender: P::Sender,
    receiver: P::Receiver,
    event_queue: VecDeque<PendingEvent>,
    rendered: RenderedState<A>,
    current_frame: Option<Frame<A>>,
    next_frame: Option<Frame<A>>,
    services: ServiceCollection<A::Message>,
    render_tx: UnboundedSender<()>,
    render_rx: UnboundedReceiver<()>,
    invalidated_components: Option<HashSet<Id>>,
    components: HashMap<Id, ComponentContainer<A::Message>>,
    invalidate_all: bool,
    not_applied_counter: i32,
    dirty: bool,
    metrics: Metrics,
    dialogs: VecDeque<DialogBinding<A::Message>>
}

pub struct RuntimeControl<A: App> {
    tx: UnboundedSender<RuntimeMsg<A>>,
}

impl<A: App> RuntimeControl<A> {
    pub fn cancel(&self) {
        self.tx.unbounded_send(RuntimeMsg::Cancel).unwrap();
    }

    pub fn update(&self, msg: A::Message) {
        self.tx.unbounded_send(RuntimeMsg::Update(msg)).unwrap();
    }
}

enum RuntimeMsg<A: App> {
    Cancel,
    Update(A::Message),
    ApplyNextFrame(Frame<A>, Duration),
    NextFrameRendering(Frame<A>, Duration),
    AsyncMsg(A::Message),
}

impl<A: 'static + App, P: 'static + Pipe> Runtime<A, P> {
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
            components: Default::default(),
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

    pub async fn run(mut self) -> Metrics {
        self.schedule_render(DEFAULT_RENDER_INTERVAL);
        let (ctx, receiver) = Context::<A::Message>::new();
        self.app.mount(ctx);
        self.handle_mailbox_result(receiver);
        loop {
            select! {
                _ = self.render_rx.next().fuse() => {
                    self.dirty = false;
                    self.render_dom()
                },
                msg = self.receiver.next().fuse() => {
                    if let Some(msg) = msg {
                        if !self.handle_pipe_msg(msg).await {
                            break;
                        }
                    } else {
                        break;
                    }
                },
                msg = self.rx.next().fuse() => {
                    if let Some(msg) = msg {
                        if !self.handle_msg(msg) {
                            break;
                        }
                    } else {
                        break;
                    }
                },
                msg = self.services.next().fuse() => {
                    if let Some(msg) = msg {
                        self.handle_service_msg(msg);
                    }
                }
            }
        }
        self.metrics
    }

    pub fn run_blocking(self) -> Metrics {
        async_std::task::block_on(self.run())
    }

    fn handle_service_msg(&mut self, msg: ServiceMessage<A::Message>) {
        match msg {
            ServiceMessage::Update(msg) => self.update(msg),
            ServiceMessage::Tx(id, msg) => self.sender.send(TxMsg::Service(id.data(), msg)),
            ServiceMessage::Stopped() => {}
        }
    }

    async fn handle_pipe_msg(&mut self, msg: RxMsg) -> bool {
        match msg {
            RxMsg::Event(evt) => {

                // search in listeners and get a message
                let msg = self.rendered
                    .get_listener(&evt.target(), evt.name())
                    .map(|listener| listener.call(evt));

                // inject the message back into the app
                if let Some(msg) = msg {
                    self.update(msg);
                    self.process_events();
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
                self.services.send(Id::from_data(id), msg);
            },
            RxMsg::Dialog(data) => {
                // cannot receive a dialog message if no dialog is active
                // since this is the only place where we pop
                let dialog = self.dialogs.pop_front().unwrap();
                // panic if data was ill formated since that is a bug in the backend
                let msg = dialog.resolve(data).unwrap();
                self.update(msg);
                self.process_events();
                if self.dialogs.len() > 0 {
                    // show next dialog
                    let data = self.dialogs.get(0).unwrap().serialize();
                    self.sender.send(TxMsg::Dialog(data))
                }
            }
        };
        true
    }

    fn handle_msg(&mut self, msg: RuntimeMsg<A>) -> bool {
        match msg {
            RuntimeMsg::Cancel => {return false;},
            RuntimeMsg::Update(msg) => {
                self.update(msg);
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
                self.update(msg);
            }
        }
        true
    }

    fn schedule_render(&mut self, wait_time: u64) {
        if self.dirty {
            return;
        }
        let render_tx = self.render_tx.clone();
        task::spawn(async move {
            let mut timer = Interval::platform_new(core::time::Duration::from_millis(wait_time));
            timer.as_mut().await;
            timer.cancel();
            let _ = render_tx.unbounded_send(());
        });
        self.dirty = true;
    }

    fn update(&mut self, msg: A::Message) {
        let (ctx, receiver) = Context::<A::Message>::new();
        let updated = self.app.update(msg, ctx);
        if updated.should_render {
            self.invalidate_all = true;
            self.schedule_render(DEFAULT_RENDER_INTERVAL);
        } else if let Some(invalidated) = updated.components_render {
            let invalidated_components = self.invalidated_components.as_mut().unwrap();
            invalidated.iter().for_each(|x| { invalidated_components.insert(*x); });
            self.schedule_render(DEFAULT_RENDER_INTERVAL);
        };
        self.handle_mailbox_result(receiver);
    }

    fn handle_mailbox_result(&mut self, receiver: MailboxReceiver<A::Message>) {
        while let Ok(cmd) = receiver.rx.try_recv() {
            match cmd {
                ContextMsg::Emission(e) => {
                    self.event_queue.push_back(PendingEvent::Component(e));
                },
                ContextMsg::LoadCss(css) => {
                    self.sender.send(TxMsg::LoadCss(css));
                },
                ContextMsg::RunJs(js) => {
                    self.sender.send(TxMsg::RunJs(js));
                },
                ContextMsg::Propagate(prop) => {
                    self.sender.send(TxMsg::Propagate(prop));
                },
                ContextMsg::Subscription(service) => {
                    self.services.spawn(service);
                }
                ContextMsg::Future(fut) => {
                    let tx = self.tx.clone();
                    async_std::task::spawn(async move {
                        let result = fut.await;
                        let _ = tx.unbounded_send(RuntimeMsg::AsyncMsg(result));
                    });
                }
                ContextMsg::Stream(mut stream) => {
                    let tx = self.tx.clone();
                    async_std::task::spawn(async move {
                        while let Some(value) = stream.next().await {
                            let _ = tx.unbounded_send(RuntimeMsg::AsyncMsg(value));
                        }
                    });
                }
                ContextMsg::Dialog(dialog) => {
                    if self.dialogs.len() == 0 {
                        self.sender.send(TxMsg::Dialog(dialog.serialize()))
                    }
                    self.dialogs.push_back(dialog);
                }
            }
        }
    }

    fn process_events(&mut self) {
        while let Some(evt) = self.event_queue.pop_front() {
            let msg = match evt {
                PendingEvent::Component(e) => self.rendered.get_subscription(&e.event_id)
                    .map(|subs| subs.call(e.data)),
            };
            if let Some(msg) = msg {
                self.update(msg);
            }
        }
    }

    fn render_dom(&mut self) {
        if self.next_frame.is_some() && self.current_frame.is_none() && self.not_applied_counter < 3 {
            self.not_applied_counter += 1;
            self.dirty = false;
            self.schedule_render(RENDER_RETRY_INTERVAL);
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
        let sender = self.sender.clone();

        async_std::task::spawn_blocking(move || {
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
                sender.send(TxMsg::Patch(serialized));
            }
        });
    }
}
