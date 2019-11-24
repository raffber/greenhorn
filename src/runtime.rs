use crate::component::{App, ComponentMap, Listener, Node, ComponentContainer};
use crate::event::{Emission, Subscription};
use crate::mailbox::Mailbox;
use crate::pipe::Sender;
use crate::pipe::{Pipe, RxMsg, TxMsg};
use crate::service::ServiceSubscription;
use crate::vdom::{diff, patch_serialize, Patch, VElement, VNode, EventHandler};
use crate::Id;
use async_timer::Interval;
use futures::channel::mpsc::{unbounded, UnboundedReceiver, UnboundedSender};
use futures::{select, FutureExt, StreamExt};
use std::collections::{HashMap, VecDeque};

enum PendingEvent {
    Component(Emission),
}

struct ComponentDom<M: 'static> {
    component: Box<dyn ComponentMap<M>>,
}

pub struct Runtime<A: 'static + App, P: Pipe> {
    tx: UnboundedSender<RuntimeMsg>,
    rx: UnboundedReceiver<RuntimeMsg>,
    app: A,
    sender: P::Sender,
    receiver: P::Receiver,
    event_queue: VecDeque<PendingEvent>,
    rendered: RenderedState<A::Message>,
    next_frame: Option<Frame<A::Message>>,
    services: Vec<ServiceSubscription<A::Message>>,
}

pub struct RuntimeControl {
    tx: UnboundedSender<RuntimeMsg>,
}

impl RuntimeControl {
    pub fn cancel(&self) {
        self.tx.unbounded_send(RuntimeMsg::Cancel).unwrap();
    }
}

enum RuntimeMsg {
    Cancel,
}


struct Frame<Msg> {
    vdom: Option<VNode>,
    subscriptions: Vec<(Id, Subscription<Msg>)>,
    listeners: Vec<Listener<Msg>>,
    rendered_components: Vec<Box<dyn ComponentMap<Msg>>>,
    translations: Vec<(Id, Id)>,
}

impl<Msg> Frame<Msg> {
    fn new() -> Self {
        Self {
            vdom: None,
            subscriptions: vec![],
            listeners: vec![],
            rendered_components: vec![],
            translations: vec![]
        }
    }

    fn borrow_render_result(&mut self) -> RenderResult<Msg> {
        RenderResult {
            subscriptions: &mut self.subscriptions,
            listeners: &mut self.listeners,
            components: &mut self.rendered_components
        }
    }
}


struct RenderedState<Msg> {
    vdom: Option<VNode>,
    subscriptions: HashMap<Id, Subscription<Msg>>,
    listeners: HashMap<Id, Listener<Msg>>,
    translations: HashMap<Id, Id>,
    components: HashMap<Id, ComponentContainer<Msg>>,
}

impl<Msg> RenderedState<Msg> {
    fn new() -> Self {
        Self {
            vdom: None,
            subscriptions: Default::default(),
            listeners: Default::default(),
            translations: Default::default(),
            components: Default::default()
        }
    }

    fn apply(&mut self, frame: Frame<Msg>) {
        self.translations.clear();
        for (k, v) in frame.translations {
            self.translations.insert(k, v);
        }

        self.listeners.clear();
        for listener  in frame.listeners {
            self.listeners.insert(listener.node_id, listener);
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
    pub fn new(app: A, pipe: P) -> (Runtime<A, P>, RuntimeControl) {
        let (tx, rx) = unbounded::<RuntimeMsg>();
        let (sender, receiver) = pipe.split();
        let runtime = Runtime {
            tx: tx.clone(),
            rx,
            app,
            sender,
            receiver,
            event_queue: VecDeque::new(),
            rendered: RenderedState::new(),
            services: Vec::new(),
            next_frame: None
        };
        let control = RuntimeControl { tx };
        (runtime, control)
    }

    pub async fn run(mut self) {
        let mut interval = Interval::platform_new(core::time::Duration::from_millis(30));
        loop {
            select! {
                _ = interval.as_mut().fuse() => {
                    self.render_dom()
                },
                msg = self.receiver.next().fuse() => {
                    if let Some(msg) = msg {
                        if !self.handle_pipe_msg(msg).await {
                            return;
                        }
                    } else {
                        return;
                    }
                },
                msg = self.rx.next().fuse() => {
                    if let Some(msg) = msg {
                        if !self.handle_msg(msg) {
                            return;
                        }
                    } else {
                        return;
                    }
                }
            }
        }
    }

    async fn handle_pipe_msg(&mut self, msg: RxMsg) -> bool {
        match msg {
            RxMsg::Event(evt) => {
                let msg = self
                    .rendered
                    .listeners
                    .get(&evt.id())
                    .map(|listener| listener.call(evt));
                if let Some(msg) = msg {
                    self.update(msg);
                }
                self.process_events();
            }
            RxMsg::FrameApplied() => {
                if let Some(frame) = self.next_frame.take() {
                    self.rendered.apply(frame);
                }
            }
        };
        true
    }

    fn handle_msg(&mut self, msg: RuntimeMsg) -> bool {
        match msg {
            RuntimeMsg::Cancel => true,
        }
    }

    fn update(&mut self, msg: A::Message) {
        let (mailbox, receiver) = Mailbox::<A::Message>::new();
        self.app.update(msg, mailbox);
        while let Ok(e) = receiver.emissions.try_recv() {
            self.event_queue.push_back(PendingEvent::Component(e));
        }
        while let Ok(service) = receiver.services.try_recv() {
            self.services.push(service);
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
                    events.push(EventHandler {
                        name: listener.event_name.clone(),
                        no_propagate: listener.no_propagate,
                        prevent_default: listener.prevent_default,
                    });
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
            },
            Node::Component(comp) => {
                let rendered = comp.render();
                result.components.push(comp);
                self.render_recursive(result, rendered)
            }
            Node::Text(id, text) => Some(VNode::text(text, id)),
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
                    events.push(EventHandler {
                        name: listener.event_name.clone(),
                        no_propagate: listener.no_propagate,
                        prevent_default: listener.prevent_default,
                    });
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
        let new_dom = frame.vdom.as_mut().expect("Expected an actual DOM to render.");

        // create a patch
        let mut patch = if let Some(old_dom) = old_dom {
            diff(old_dom, new_dom.clone())
        } else {
            Patch::from_dom(new_dom.clone())
        };
        frame.translations.append(&mut patch.translations);

        // schedule next frame
        self.next_frame = Some(frame);

        // serialize the patch and send it to the client
        let serialized = patch_serialize(patch);
        self.sender.send(TxMsg::Patch(serialized));
    }
}
