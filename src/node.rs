use crate::blob::Blob;
use crate::component::{ComponentContainer, MappedComponent};
use crate::element::{Element, ElementMapDirect, ElementRemap, MappedElement};
use crate::event::Subscription;
use crate::node_builder::NodeBuilder;
use crate::Id;
use std::fmt::{Debug, Error, Formatter};
use std::sync::{Arc, Mutex};

/// Represents a DOM node which might emit a message of type `T`.
///
/// `Node` objects are produced by `Render::render()` functions.
/// Each [Render](../trait.Render.html) implementation has an associated `Message` type,
/// which is emitted by the returned `Node`s, thus feeding information back into the
/// `update()` cycle of the application.
///
/// Nodes may be constructed using a [NodeBuilder](../node_builder/struct.NodeBuilder.html):
///  * Elements by using an [ElementBuilder](../node_builder/struct.ElementBuilder.html)
///  * Text
///  * [Component](../component/struct.Component.html) instances
///  * Event subscriptions
///  * Blobs
///
/// Furthermore, `Node`s can be constructed using the `html!()` and `svg!()` macros.
/// Nodes may not necessarily lead to a node rendered in the DOM but merely represent
/// *state in the frontend*.
///
/// # Example
///
/// ```
/// # use greenhorn::{Render, Component};
/// # use greenhorn::node::Node;
/// # use greenhorn::blob::Blob;
/// # use greenhorn::event::Event;
/// #
/// # struct Button {
/// #     clicked: Event<()>
/// # }
/// # impl Render for Button {
/// #    type Message = ();
/// #    fn render(&self) -> Node<Self::Message> {
/// #        unimplemented!()
/// #    }
/// # }
/// #
/// struct MyRenderable {
///     blob: Blob,
///     button: Component<Button>,
/// };
/// #
/// # enum MyMsg {
/// #    ButtonMsg(()),
/// #    Clicked,
/// # }
///
/// impl Render for MyRenderable {
///     type Message = MyMsg;
///
///     fn render(&self) -> Node<Self::Message> {
///         Node::html().elem("div").class("primary").id("my-div")  // an HTML element
///             .add(Node::text("Some Text"))                       // a text node
///             .add(&self.blob)                                    // a blob
///             .add(self.button.mount().map(MyMsg::ButtonMsg))     // a component
///             .add(self.button.lock().clicked.subscribe(|_| MyMsg::Clicked)) // an event
///             .build()
///     }
/// }
/// ```
///
pub struct Node<T: 'static + Send>(pub(crate) NodeItems<T>);

pub(crate) enum NodeItems<T: 'static + Send> {
    ElementMap(MappedElement<T>),
    Component(ComponentContainer<T>),
    Text(String),
    Element(Element<T>),
    Blob(Blob),
    EventSubscription(Id, Subscription<T>),
}

impl<T: 'static + Send> Debug for Node<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match &self.0 {
            NodeItems::ElementMap(x) => std::fmt::Debug::fmt(&x, f),
            NodeItems::Component(x) => std::fmt::Debug::fmt(&x, f),
            NodeItems::Text(text) => f.write_str(&text),
            NodeItems::Element(elem) => elem.fmt(f),
            NodeItems::EventSubscription(_, subs) => subs.fmt(f),
            NodeItems::Blob(blob) => blob.fmt(f),
        }
    }
}

impl<T: 'static + Send> Node<T> {
    /// Create a [NodeBuilder](../node_builder/struct.NodeBuilder.html) for HTML elements
    pub fn html() -> NodeBuilder<T> {
        NodeBuilder::new()
    }

    /// Create a [NodeBuilder](../node_builder/struct.NodeBuilder.html) for SVG elements
    pub fn svg() -> NodeBuilder<T> {
        NodeBuilder::new_with_ns("http://www.w3.org/2000/svg")
    }

    /// Produce a text node
    pub fn text<S: ToString>(data: S) -> Self {
        Node(NodeItems::Text(data.to_string()))
    }

    /// Maps the message type of the node to a new message type
    ///
    /// When nesting different `Render` implementations or components, the message types need
    /// to be nested as well. The `Node::map()` function allows the emitted messages
    /// of type `T` to be mapped to another message type `U`.
    ///
    /// ## Example
    ///
    /// ```
    /// # use greenhorn::dom::DomEvent;
    /// # use greenhorn::node::Node;
    /// # use greenhorn::html;
    ///
    /// enum ButtonMsg {
    ///     KeyDown(DomEvent),
    ///     Clicked(DomEvent),
    /// }
    ///
    /// enum MyAppMsg {
    ///     ButtonOk(ButtonMsg),
    ///     ButtonCancel(ButtonMsg),
    /// }
    ///
    /// fn render_button(text: &str) -> Node<ButtonMsg> {
    ///     // ...
    /// #    unimplemented!()
    /// }
    ///
    /// fn render() -> Node<MyAppMsg> {
    ///     html!(
    ///         <div>
    ///             {render_button("Ok").map(MyAppMsg::ButtonOk)}
    ///             {render_button("Cancel").map(MyAppMsg::ButtonCancel)}
    ///         </>
    ///     ).into()
    /// }
    /// ```
    pub fn map<U: 'static + Send, F: 'static + Send + Fn(T) -> U>(self, fun: F) -> Node<U> {
        let fun: Arc<Mutex<Box<dyn 'static + Send + Fn(T) -> U>>> =
            Arc::new(Mutex::new(Box::new(fun)));
        self.map_shared(fun)
    }

    /// same as `Node::map()` but uses a shared reference of an already created mapping function
    pub fn map_shared<U: 'static + Send>(
        self,
        fun: Arc<Mutex<dyn 'static + Send + Fn(T) -> U>>,
    ) -> Node<U> {
        let ret = match self.0 {
            NodeItems::ElementMap(inner) => {
                let ret = ElementRemap::new_box(fun, inner.inner);
                NodeItems::ElementMap(ret)
            }
            NodeItems::Component(inner) => {
                NodeItems::Component(MappedComponent::new_container(fun, inner.inner))
            }
            NodeItems::Text(text) => NodeItems::Text(text),
            NodeItems::Element(elem) => NodeItems::ElementMap(ElementMapDirect::new_box(fun, elem)),
            NodeItems::EventSubscription(id, evt) => NodeItems::EventSubscription(id, evt.map(fun)),
            NodeItems::Blob(blob) => NodeItems::Blob(blob),
        };
        Node(ret)
    }

    /// Maps Node() objects without providing a mapping-functions.
    ///
    /// Panics in case there are listeners installed on this node or
    /// any child node.
    /// This allows mapping node-hierarchies without listeners efficiently without
    /// keeping the target message type around, for example when caching rendered nodes.
    pub fn empty_map<U: 'static + Send>(self) -> Node<U> {
        match self.0 {
            NodeItems::ElementMap(_) => panic!(),
            NodeItems::Component(_) => panic!(),
            NodeItems::Text(x) => Node(NodeItems::Text(x)),
            NodeItems::Element(elem) => {
                if !elem.listeners.unwrap().is_empty() {
                    panic!();
                }
                if elem.rpc.is_some() {
                    panic!();
                }
                let children = elem
                    .children
                    .map(|mut x| x.drain(..).map(|x| x.empty_map()).collect());
                Node(NodeItems::Element(Element {
                    id: elem.id,
                    tag: elem.tag,
                    attrs: elem.attrs,
                    js_events: elem.js_events,
                    listeners: Some(vec![]),
                    children,
                    namespace: elem.namespace,
                    rpc: None,
                }))
            }
            NodeItems::EventSubscription(_, _) => panic!(),
            NodeItems::Blob(blob) => Node(NodeItems::Blob(blob)),
        }
    }

    /// Attempt to clone this `Node`.
    ///
    /// If the `Node` has been mapped to a different type, the `Node` cannot be cloned anymore.
    /// Also, mounted components and event subscriptions cannot be cloned.
    pub fn try_clone(&self) -> Option<Self> {
        match &self.0 {
            NodeItems::Element(elem) => {
                if let Some(ret) = elem.try_clone() {
                    Some(Node(NodeItems::Element(ret)))
                } else {
                    None
                }
            }
            NodeItems::Text(txt) => Some(Node(NodeItems::Text(txt.clone()))),
            NodeItems::Blob(blob) => Some(Node(NodeItems::Blob(blob.clone()))),
            _ => None,
        }
    }
}
