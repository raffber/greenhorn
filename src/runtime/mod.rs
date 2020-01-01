use crate::component::App;
use crate::event::{Emission};
use crate::mailbox::{Mailbox, MailboxMsg, MailboxReceiver};
use crate::pipe::Sender;
use crate::pipe::{Pipe, RxMsg, TxMsg};
use crate::runtime::service_runner::{ServiceCollection, ServiceMessage};
use crate::vdom::{diff, patch_serialize, Patch, Path};
use crate::Id;
use async_std::task;
use async_timer::Interval;
use futures::channel::mpsc::{unbounded, UnboundedReceiver, UnboundedSender};
use futures::{select, FutureExt, StreamExt};
use std::collections::{HashMap, VecDeque, HashSet};
use crate::node::{ComponentMap, ComponentContainer};
use crate::runtime::render::{RenderResult, Frame, RenderedState};

mod service_runner;
mod render;

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
    rendered: RenderedState<A>,
    next_frame: Option<Frame<A>>,
    services: ServiceCollection<A::Message>,
    render_tx: UnboundedSender<()>,
    render_rx: UnboundedReceiver<()>,
    invalidated_components: HashSet<Id>,
    components: HashMap<Id, ComponentContainer<A::Message>>,
    invalidate_all: bool,
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
            invalidated_components: Default::default(),
            components: Default::default(),
            next_frame: None,
            dirty: false,
            invalidate_all: false
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
            self.invalidate_all = true;
            self.schedule_render();
        } else if let Some(invalidated) = updated.components_render {
            invalidated.iter().for_each(|x| {
                self.invalidated_components.insert(*x);
            });
            self.schedule_render();
        };
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
                PendingEvent::Component(e) => self.rendered.get_subscription(&e.event_id)
                    .map(|subs| subs.call(e.data)),
            };
            if let Some(msg) = msg {
                self.update(msg);
            }
        }
    }

    fn render_dom(&mut self) {
        let old_dom = self.rendered.take_vdom();
        let dom = self.app.render();


        // render new DOM
        let mut path = Path::new();
        let result = RenderResult::new(dom, &mut path);

        // create a patch
        let patch = if let Some(old_dom) = &old_dom {
            diff(Some(&old_dom), &result.vdom)
        } else {
            Patch::from_dom(&result.vdom)
        };

        self.dirty = false;
        if patch.is_empty() {
            let translations = patch.translations;
            let mut frame = Frame::new(result, &translations);
            frame.back_annotate();
            self.rendered.apply(frame);
        } else {
            let serialized = patch_serialize(&patch);
            let translations = patch.translations;
            let mut frame = Frame::new(result, &translations);
            frame.back_annotate();

            // schedule next frame
            self.next_frame = Some(frame);

            // serialize the patch and send it to the client
            self.sender.send(TxMsg::Patch(serialized));
        }
    }
}
