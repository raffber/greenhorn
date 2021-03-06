//! The context module exports the [`Context`](struct.Context.html) type,
//! which is passed into each `update()` cycle and allows interfacing with system services.
//!
//! [`Context`](struct.Context.html) objects support the following features:
//! * Emitting events - This may trigger a new `update()` cycle if an [`Event`](../event/struct.Event.html)
//!     was subscribed to.
//! * Loading CSS or JS on the frontend
//! * Spawning futures or streams - the results of either trigger a new `update()` cycle.
//! * Propagating or invoking DOM events on the frontend
//! * Showing system dialogs
//! * Quitting the application
//!
//! For more details, refer to the [`Context`](struct.Context.html) type.
//!

use crate::dialog::{Dialog, DialogBinding};
use crate::dom::DomEvent;
use crate::event::{Emission, Event};
use crate::service::{Service, ServiceSubscription};
use futures::{Stream, StreamExt};
use serde::{Deserialize, Serialize};
use std::any::Any;
use std::future::Future;
use std::marker::PhantomData;
use std::pin::Pin;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Arc;

enum MapSender<T> {
    Direct(Sender<T>),
    Mapped(Box<dyn MappedSender<T>>),
}

impl<T> Clone for MapSender<T> {
    fn clone(&self) -> Self {
        match self {
            MapSender::Direct(sender) => MapSender::Direct(sender.clone()),
            MapSender::Mapped(mapped) => MapSender::Mapped(mapped.clone_box()),
        }
    }
}

impl<T: 'static> MapSender<T> {
    fn new(sender: Sender<T>) -> Self {
        MapSender::Direct(sender)
    }

    fn map<U: 'static, Mapper: 'static + Fn(U) -> T>(self, mapper: Mapper) -> MapSender<U> {
        MapSender::Mapped(Box::new(MappedSenderImpl {
            mapper: Arc::new(mapper),
            sender: self,
            phantom: PhantomData,
        }))
    }

    fn send(&self, value: T) {
        match self {
            MapSender::Direct(sender) => {
                sender.send(value).unwrap();
            }
            MapSender::Mapped(mapped) => mapped.send(value),
        }
    }
}

trait MappedSender<T> {
    fn send(&self, value: T);
    fn clone_box(&self) -> Box<dyn MappedSender<T>>;
}

struct MappedSenderImpl<T, U, Mapper: Fn(U) -> T> {
    mapper: Arc<Mapper>,
    sender: MapSender<T>,
    phantom: PhantomData<U>,
}

impl<T: 'static, U: 'static, Mapper: 'static + Fn(U) -> T> MappedSender<U>
    for MappedSenderImpl<T, U, Mapper>
{
    fn send(&self, value: U) {
        let value = (self.mapper)(value);
        self.sender.send(value);
    }

    fn clone_box(&self) -> Box<dyn MappedSender<U>> {
        Box::new(MappedSenderImpl {
            mapper: self.mapper.clone(),
            sender: self.sender.clone(),
            phantom: PhantomData,
        })
    }
}

/// Messages sent from the `Context` to communicate with the `Runtime`.
pub(crate) enum ContextMsg<T: 'static + Send> {
    Emission(Emission),
    LoadCss(String),
    RunJs(String),
    Propagate(EventPropagate),
    Subscription(ServiceSubscription<T>),
    Future(Pin<Box<dyn Send + Future<Output = T>>>, bool), // (future, blocking)
    Stream(Pin<Box<dyn Send + Stream<Item = T>>>),
    Dialog(DialogBinding<T>),
    Quit,
}

impl<T: Send + 'static> ContextMsg<T> {
    pub(crate) fn map<U, Mapper>(self, mapper: Arc<Mapper>) -> ContextMsg<U>
    where
        U: 'static + Send,
        Mapper: 'static + Fn(T) -> U + Send + Sync,
    {
        match self {
            ContextMsg::Subscription(subs) => ContextMsg::Subscription(subs.map(mapper)),
            ContextMsg::Future(fut, blocking) => {
                ContextMsg::Future(Box::pin(async move { (mapper)(fut.await) }), blocking)
            }
            ContextMsg::Stream(stream) => {
                ContextMsg::Stream(Box::pin(stream.map(move |x| (mapper)(x))))
            }
            ContextMsg::Dialog(d) => ContextMsg::Dialog(d.map(mapper)),
            ContextMsg::Emission(x) => ContextMsg::Emission(x),
            ContextMsg::LoadCss(x) => ContextMsg::LoadCss(x),
            ContextMsg::RunJs(x) => ContextMsg::RunJs(x),
            ContextMsg::Propagate(x) => ContextMsg::Propagate(x),
            ContextMsg::Quit => ContextMsg::Quit,
        }
    }
}

pub(crate) struct ContextReceiver<T: 'static + Send> {
    pub(crate) rx: Receiver<ContextMsg<T>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EventPropagate {
    event: DomEvent,
    propagate: bool,
    default_action: bool,
}

/// `Context` objects are passed into the `update()` function of the application
/// and allow interacting with the component hierarchy, controlling the application lifecycle
/// and provide access to system services.
pub struct Context<T: 'static + Send> {
    tx: MapSender<ContextMsg<T>>,
}

impl<T: 'static + Send> Clone for Context<T> {
    fn clone(&self) -> Self {
        Self {
            tx: self.tx.clone(),
        }
    }
}

impl<T: Send + 'static> Context<T> {
    pub(crate) fn new() -> (Self, ContextReceiver<T>) {
        let (tx, rx) = channel();
        (
            Context {
                tx: MapSender::new(tx),
            },
            ContextReceiver { rx },
        )
    }

    /// Emits an event from the current component.
    ///
    /// This will result in an `update()` call for all components subscribed to this event.
    /// Emitting [Events](../event/struct.Event.html) triggers new `update()` cycles of the application
    /// and thus allows interchanging messages between components.
    pub fn emit<D: Any>(&self, event: &Event<D>, data: D) {
        let emission = event.emit(data);
        self.tx.send(ContextMsg::Emission(emission));
    }

    /// Loads a CSS string on the frontend
    pub fn load_css<Css: Into<String>>(&self, css: Css) {
        self.tx.send(ContextMsg::LoadCss(css.into()));
    }

    /// Runs a piece of javascript code on the frontend
    pub fn run_js<Js: Into<String>>(&self, js: Js) {
        self.tx.send(ContextMsg::RunJs(js.into()));
    }

    /// Spawn a [`Service`](../service/trait.Service.html) using a mapping function to map
    /// the services data items to the current message type
    pub fn run_service<S, F>(&self, service: S, fun: F)
    where
        S: 'static + Service,
        F: 'static + Send + Fn(S::Data) -> T,
    {
        let subs = ServiceSubscription::new(service, fun);
        self.tx.send(ContextMsg::Subscription(subs));
    }

    /// Spawns a future. The result of the future will be used to `update()` the application.
    pub fn spawn<Fut: 'static + Send + Future<Output = T>>(&self, fut: Fut) {
        self.tx.send(ContextMsg::Future(Box::pin(fut), false));
    }

    /// Spawns a future which contains blocking operations. This future might be spawned on
    /// a different thread-pool to avoid stalling non-blocking futures.
    pub fn spawn_blocking<Fut: 'static + Send + Future<Output = T>>(&self, fut: Fut) {
        self.tx.send(ContextMsg::Future(Box::pin(fut), true));
    }

    /// Subscribe to a stream. Each item the stream issues will be used to `udpate()` the application.
    pub fn subscribe<S: 'static + Send + Stream<Item = T>>(&self, stream: S) {
        self.tx.send(ContextMsg::Stream(Box::pin(stream)));
    }

    /// Maps this context object to a new a new message type
    pub fn map<U: Send + 'static, F: 'static + Send + Sync + Fn(U) -> T>(
        &self,
        fun: F,
    ) -> Context<U> {
        let mapper = Arc::new(fun);
        let new_sender = self.tx.clone();
        let mapped = new_sender.map(move |msg: ContextMsg<U>| msg.map(mapper.clone()));
        Context { tx: mapped }
    }

    /// Propagates a previously intercepted DOM event to the frontend.
    pub fn propagate(&self, e: DomEvent) {
        self.tx.send(ContextMsg::Propagate(EventPropagate {
            event: e,
            propagate: true,
            default_action: false,
        }));
    }

    /// Propagates a DOM event to the frontend executing the default action.
    ///
    /// This is useful in case a DOM event handler was registered with
    /// [`prevent_default()`](../node_builder/struct.ListenerBuilder.html#method.prevent_default).
    /// and the application has determined that the default action should be executed anyway.
    pub fn default_action(&self, e: DomEvent) {
        self.tx.send(ContextMsg::Propagate(EventPropagate {
            event: e,
            propagate: false,
            default_action: true,
        }));
    }

    /// Propagates a previously intercepted js event to the frontend and execute the default
    /// action.
    pub fn propagate_and_default(&self, e: DomEvent) {
        self.tx.send(ContextMsg::Propagate(EventPropagate {
            event: e,
            propagate: true,
            default_action: true,
        }));
    }

    /// Opens a dialog on the frontend.
    ///
    /// Once the dialog resolves, the `fun` function maps the dialog message type to the
    /// message type of the application.
    /// Refer to the [`dialog` module](../dialog/index.html) for builtin dialogs.
    /// Note that only one dialog is shown at the same time, since dialogs are usully modal.
    /// However, multiple dialogs may be enqueued, which will then be shown one after the next.
    pub fn dialog<D: 'static + Dialog, F: 'static + Fn(D::Msg) -> T>(&self, dialog: D, fun: F) {
        let binding = DialogBinding::new(dialog, fun);
        self.tx.send(ContextMsg::Dialog(binding));
    }

    /// Quits the currently running application and shuts down all associated services.
    pub fn quit(&self) {
        self.tx.send(ContextMsg::Quit);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::tests::MsgA::ItemA;
    use crate::context::tests::MsgB::ItemB;
    use crate::context::Context;
    use crate::context::ContextMsg::Subscription;
    use crate::service::Mailbox;
    use assert_matches::assert_matches;
    use async_std::task::block_on;
    use futures::channel::mpsc::{unbounded, UnboundedReceiver};
    use futures::task::Context as TaskContext;
    use futures::task::Poll;
    use futures::{Stream, StreamExt};
    use std::pin::Pin;

    #[derive(Debug)]
    enum MsgA {
        ItemA(MsgB),
    }

    #[derive(Debug)]
    enum MsgB {
        ItemB(i32),
    }

    struct MyService {}

    impl Service for MyService {
        type Data = i32;
        type DataStream = UnboundedReceiver<i32>;

        fn start(self, _: Mailbox) -> Self::DataStream {
            let (tx, rx) = unbounded();
            let _ = tx.unbounded_send(1);
            rx
        }
    }

    impl Stream for MyService {
        type Item = i32;

        fn poll_next(self: Pin<&mut Self>, _cx: &mut TaskContext<'_>) -> Poll<Option<Self::Item>> {
            Poll::Ready(Some(1))
        }
    }

    #[test]
    fn test_service() {
        let (mb, rx) = Context::<MsgA>::new();
        let mapped = mb.map(MsgA::ItemA);
        let service = MyService {};
        mapped.run_service(service, ItemB);
        if let Ok(Subscription(mut subs)) = rx.rx.try_recv() {
            let result = block_on(subs.next());
            assert_matches!(result, Some(MsgA::ItemA(MsgB::ItemB(1))));
        } else {
            panic!();
        }
    }

    #[test]
    fn test_future() {
        let fut = async { MsgB::ItemB(123) };

        let (mb, rx) = Context::<MsgA>::new();
        let mapped = mb.map(MsgA::ItemA);
        mapped.spawn(fut);
        if let Ok(ContextMsg::Future(fut, _blocking)) = rx.rx.try_recv() {
            let result = block_on(fut);
            assert_matches!(result, MsgA::ItemA(MsgB::ItemB(123)));
        } else {
            panic!();
        }
    }

    #[test]
    fn test_stream() {
        let stream = futures::stream::unfold(0, |state| async move {
            if state <= 2 {
                let next_state = state + 1;
                Some((MsgB::ItemB(state), next_state))
            } else {
                None
            }
        });
        let (mb, rx) = Context::<MsgA>::new();
        let mapped = mb.map(MsgA::ItemA);
        mapped.subscribe(stream);
        if let Ok(ContextMsg::Stream(stream)) = rx.rx.try_recv() {
            let data: Vec<MsgA> = block_on(stream.collect::<Vec<MsgA>>());
            assert_eq!(data.len(), 3);
            assert_matches!(data[0], ItemA(ItemB(0)));
            assert_matches!(data[1], ItemA(ItemB(1)));
            assert_matches!(data[2], ItemA(ItemB(2)));
        } else {
            panic!();
        }
    }
}
