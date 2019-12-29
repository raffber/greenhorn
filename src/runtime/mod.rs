use crate::component::App;
use crate::event::{Emission, Subscription};
use crate::mailbox::{Mailbox, MailboxMsg, MailboxReceiver};
use crate::pipe::Sender;
use crate::pipe::{Pipe, RxMsg, TxMsg};
use crate::runtime::service_runner::{ServiceCollection, ServiceMessage};
use crate::vdom::{diff, patch_serialize, EventHandler, Patch, VElement, VNode};
use crate::Id;
use async_std::task;
use async_timer::Interval;
use futures::channel::mpsc::{unbounded, UnboundedReceiver, UnboundedSender};
use futures::{select, FutureExt, StreamExt};
use std::collections::{HashMap, VecDeque};
use crate::node::{ComponentMap, Node, ComponentContainer};
use crate::listener::Listener;

mod service_runner;

enum PendingEvent {
    Component(Emission),
}

struct ComponentDom<M: 'static> {
    component: Box<dyn ComponentMap<M>>,
}

pub struct Runtime<A: 'static + App, P: Pipe> {
    tx: UnboundedSender<RuntimeMsg<A::Message>>,
    rx: UnboundedReceiver<RuntimeMsg<A::Message>>,
    app: A,
    sender: P::Sender,
    receiver: P::Receiver,
    event_queue: VecDeque<PendingEvent>,
    rendered: RenderedState<A::Message>,
    next_frame: Option<Frame<A::Message>>,
    services: ServiceCollection<A::Message>,
    render_tx: UnboundedSender<()>,
    render_rx: UnboundedReceiver<()>,
    dirty: bool,
}

pub struct RuntimeControl<M: 'static + Send> {
    tx: UnboundedSender<RuntimeMsg<M>>,
}

impl<M: 'static + Send> RuntimeControl<M> {
    pub fn cancel(&self) {
        self.tx.unbounded_send(RuntimeMsg::Cancel).unwrap();
    }

    pub fn update(&self, msg: M) {
        self.tx.unbounded_send(RuntimeMsg::Update(msg)).unwrap();
    }
}

enum RuntimeMsg<M: 'static + Send> {
    Cancel,
    Update(M)
}

struct Frame<Msg> {
    vdom: Option<VNode>,
    subscriptions: Vec<(Id, Subscription<Msg>)>,
    listeners: Vec<Listener<Msg>>,
    rendered_components: Vec<Box<dyn ComponentMap<Msg>>>,
    translations: HashMap<Id, Id>,
}

impl<Msg> Frame<Msg> {
    fn new() -> Self {
        Self {
            vdom: None,
            subscriptions: vec![],
            listeners: vec![],
            rendered_components: vec![],
            translations: HashMap::new(),
        }
    }

    fn borrow_render_result(&mut self) -> RenderResult<Msg> {
        RenderResult {
            subscriptions: &mut self.subscriptions,
            listeners: &mut self.listeners,
            components: &mut self.rendered_components,
        }
    }

    fn back_annotate(&mut self) {
        if let Some(ref mut vdom) = self.vdom {
            vdom.back_annotate(&self.translations);
        }
    }
}

struct RenderedState<Msg> {
    vdom: Option<VNode>,
    subscriptions: HashMap<Id, Subscription<Msg>>,
    listeners: HashMap<ListenerKey, Listener<Msg>>,
    components: HashMap<Id, ComponentContainer<Msg>>,
}

#[derive(Hash, Eq, Debug)]
struct ListenerKey {
    id: Id,
    name: String,
}

impl PartialEq for ListenerKey {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && self.name == other.name
    }
}

impl<Msg> RenderedState<Msg> {
    fn new() -> Self {
        Self {
            vdom: None,
            subscriptions: Default::default(),
            listeners: Default::default(),
            components: Default::default(),
        }
    }

    fn apply(&mut self, frame: Frame<Msg>) {
        self.listeners.clear();

        for listener in frame.listeners {
            let key = if let Some(new_id) = frame.translations.get(&listener.node_id) {
                ListenerKey {
                    id: *new_id,
                    name: listener.event_name.clone(),
                }
            } else {
                ListenerKey {
                    id: listener.node_id,
                    name: listener.event_name.clone(),
                }
            };
            self.listeners.insert(key, listener);
        }

        self.subscriptions.clear();
        for (k, v) in frame.subscriptions {
            self.subscriptions.insert(k, v);
        }

        self.vdom = frame.vdom;
    }
}

struct RenderResult<'a, Msg> {
    subscriptions: &'a mut Vec<(Id, Subscription<Msg>)>,
    listeners: &'a mut Vec<Listener<Msg>>,
    components: &'a mut Vec<Box<dyn ComponentMap<Msg>>>,
}

impl<A: App, P: 'static + Pipe> Runtime<A, P> {
    pub fn new(app: A, pipe: P) -> (Runtime<A, P>, RuntimeControl<A::Message>) {
        let (tx, rx) = unbounded::<RuntimeMsg<A::Message>>();
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
            next_frame: None,
            dirty: false,
        };
        let control = RuntimeControl { tx };
        (runtime, control)
    }

    pub async fn run(mut self) {
        self.schedule_render();
        let (mailbox, receiver) = Mailbox::<A::Message>::new();
        self.app.mount(mailbox);
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
                let key = ListenerKey { id: evt.target(), name: evt.name().into() };

                // search in listeners and get a message
                let msg = self
                    .rendered
                    .listeners
                    .get(&key)
                    .map(|listener| listener.call(evt));

                // inject the message back into the app
                if let Some(msg) = msg {
                    self.update(msg);
                    self.process_events();
                }
            }
            RxMsg::FrameApplied() => {
                if let Some(frame) = self.next_frame.take() {
                    self.rendered.apply(frame);
                }
            }
            RxMsg::Ping() => {}
            RxMsg::Service(id, msg) => {
                self.services.send(Id::from_data(id), msg);
            }
        };
        true
    }

    fn handle_msg(&mut self, msg: RuntimeMsg<A::Message>) -> bool {
        match msg {
            RuntimeMsg::Cancel => false,
            RuntimeMsg::Update(msg) => {
                self.update(msg);
                true
            }
        }
    }

    fn schedule_render(&mut self) {
        if self.dirty {
            return;
        }
        let render_tx = self.render_tx.clone();
        task::spawn(async move {
            let mut timer = Interval::platform_new(core::time::Duration::from_millis(30));
            timer.as_mut().await;
            timer.cancel();
            let _ = render_tx.unbounded_send(());
        });
        self.dirty = true;
    }

    fn update(&mut self, msg: A::Message) {
        let (mailbox, receiver) = Mailbox::<A::Message>::new();
        let updated = self.app.update(msg, mailbox);
        if updated.should_render {
            self.schedule_render();
        }
        self.handle_mailbox_result(receiver);
    }

    fn handle_mailbox_result(&mut self, receiver: MailboxReceiver<A::Message>) {
        while let Ok(cmd) = receiver.rx.try_recv() {
            match cmd {
                MailboxMsg::Emission(e) => {
                    self.event_queue.push_back(PendingEvent::Component(e));
                },
                MailboxMsg::LoadCss(css) => {
                    self.sender.send(TxMsg::LoadCss(css));
                },
                MailboxMsg::RunJs(js) => {
                    self.sender.send(TxMsg::RunJs(js));
                },
                MailboxMsg::Propagate(prop) => {
                    self.sender.send(TxMsg::Propagate(prop));
                },
            }
        }
        while let Ok(service) = receiver.services.try_recv() {
            self.services.spawn(service);
        }
    }

    fn process_events(&mut self) {
        while let Some(evt) = self.event_queue.pop_front() {
            let msg = match evt {
                PendingEvent::Component(e) => self
                    .rendered
                    .subscriptions
                    .get(&e.event_id)
                    .map(|subs| subs.call(e.data)),
            };
            if let Some(msg) = msg {
                self.update(msg);
            }
        }
    }

    fn render_recursive<'a>(
        &mut self,
        result: &mut RenderResult<'a, A::Message>,
        dom: Node<A::Message>,
    ) -> Option<VNode> {
        match dom {
            Node::ElementMap(mut elem) => {
                let mut children = Vec::new();
                for child in elem.take_children().drain(..) {
                    let child = self.render_recursive(result, child);
                    if let Some(child) = child {
                        children.push(child);
                    }
                }
                let mut events = Vec::new();
                for listener in elem.take_listeners().drain(..) {
                    events.push(EventHandler::from_listener(&listener));
                    result.listeners.push(listener);
                }
                Some(VNode::element(VElement {
                    id: elem.id(),
                    tag: elem.take_tag(),
                    attr: elem.take_attrs(),
                    events,
                    children,
                    namespace: elem.take_namespace(),
                }))
            }
            Node::Component(comp) => {
                let rendered = comp.render();
                result.components.push(comp);
                self.render_recursive(result, rendered)
            }
            Node::Text(text) => Some(VNode::text(text)),
            Node::Element(mut elem) => {
                let mut children = Vec::new();
                for child in elem.children.take().unwrap().drain(..) {
                    let child = self.render_recursive(result, child);
                    if let Some(child) = child {
                        children.push(child);
                    }
                }
                let mut events = Vec::new();
                for listener in elem.listeners.take().unwrap().drain(..) {
                    events.push(EventHandler::from_listener(&listener));
                    result.listeners.push(listener);
                }
                Some(VNode::element(VElement {
                    id: elem.id,
                    tag: elem.tag.take().unwrap(),
                    attr: elem.attrs.take().unwrap(),
                    events,
                    children,
                    namespace: elem.namespace,
                }))
            }
            Node::EventSubscription(event_id, subs) => {
                result.subscriptions.push((event_id, subs));
                None
            }
        }
    }

    fn render_dom(&mut self) {
        let old_dom = self.rendered.vdom.take();
        let dom = self.app.render();

        let mut frame = Frame::new();
        let mut result = frame.borrow_render_result();

        // render new DOM
        frame.vdom = self.render_recursive(&mut result, dom);
        let new_dom = frame
            .vdom
            .as_mut()
            .expect("Expected an actual DOM to render.");

        // create a patch
        let patch = if let Some(old_dom) = &old_dom {
            diff(Some(&old_dom), &new_dom)
        } else {
            Patch::from_dom(&new_dom)
        };

        let mut translations = HashMap::new();
        for (k, v) in &patch.translations {
            translations.insert(k.clone(), v.clone());
        }
        frame.translations = translations;

        self.dirty = false;
        if patch.is_empty() {
            frame.back_annotate();
            self.rendered.apply(frame);
        } else {
            let serialized = patch_serialize(patch);

            // schedule next frame
            frame.back_annotate();
            self.next_frame = Some(frame);

            // serialize the patch and send it to the client
            self.sender.send(TxMsg::Patch(serialized));
        }
    }
}
