
# Application Structure

Let's first consider the following "Hello, World"-application from the quickstart repository:

```rust
use greenhorn::prelude::*;

pub enum MainMsg {
    SayHello,
    SayGoodbye,    
}

pub struct MainApp {
    msg: String,
}

impl MainApp {
    pub fn new() -> Self {
        Self {
            msg: "something".into(),
        }
    }
}

impl Render for MainApp {
    type Message = MainMsg;

    fn render(&self) -> Node<Self::Message> {
        use greenhorn::html;

        html!(
            <div>
                <h1>{format!("Say {}!", self.msg)}</>
                <input type="button" @click={|_| MainMsg::SayHello} value="Hello" />
                <input type="button" @click={|_| MainMsg::SayGoodbye} value="Goodbye" />
            </>
        ).into()
    }
}

impl App for MainApp {
    fn update(&mut self, msg: Self::Message, _ctx: Context<Self::Message>) -> Updated {
        match msg {
            MainMsg::SayHello => self.msg = "hello".into(),
            MainMsg::SayGoodbye => self.msg = "goodbye".into()
        }
        Updated::yes()
    }    
}
```

First note, that the struct `MainApp`, which implements the root application view, implements two traits, `Render` and `App`:

 * The `Render` trait implements the `render()` function. It is responsible for returning the **frontend representation** of the UI, i.e. the DOM which is to be rendered by the display layer.
 Note that `render(&self)` only obtains an immutable reference to `self`. It should not change the internal application state.

 * The `App` trait implements the `update()` function which is responsible for mutating the application state by receiving and processing `Render::Message` objects. These messages may
 originate from the frontend (such as the user clicking on a button) or from within the application (such as the successful completion of a future).

Note that all structs that implement `App` also have to implement `Render`. 
At the same time, these two traits represent two stages of how `greenhorn` renders a UI: 

 * The `render()` cycle: The `greenhorn` runtime invokes the `render()` function of the root application and renders its result on the frontend
 * The `update()` cycle: Messages are received by the runtime, dispatched to the application and mutate its state.

These two stages are independent, i.e. the `greenhorn` runtime will not run a `render()` after each `update()` call. At the same time, `render()` may also be invoked even if the `update()` function has not been called. In general, the `greenhorn` runtime will try to be smart at only re-rendering the UI if required.

Note that your application can provide a hint to the runtime when a re-render is required: As a result of the `update()` call, you may return an `Updated` object based on whether the application requires re-rendering.


## The Render trait


## The App trait

