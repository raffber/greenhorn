<h1 align="center">Greenhorn</h1>
<div align="center">
 <strong>
   Write desktop and web applications in pure server-side rust
 </strong>
</div>
<div align="center">
    <h3>
        <a href="https://github.com/raffber/greenhorn/actions">
            <img src="https://github.com/raffber/greenhorn/workflows/Tests/badge.svg"
            alt="CI Status" />
        </a>
        <span> | </span>
        <a href="https://raffber.github.io/greenhorn/greenhorn/index.html">
        API Docs
        </a>
        <span> | </span>
        <a href="https://raffber.github.io/greenhorn/book/intro.html">
        Book
        </a>
    </h3>
</div>
<br />

Greenhorn is a rust library for building desktop and web applications with web technologies in (almost)
pure rust.

This is accomplished by separating the application into a server-side process
(the backend) and web view implemented in javascript (the frontend).
While most HTML-based desktop applications leave state synchronization up to the
application logic, this library synchronizes its state at DOM-level.
Thus, the user may implement the application logic purely in the backend using rust.
This facilitates the integration of a desktop GUI with system
services and simplifies application development considerably.

## Features

* Elm-like architecture
* Components support fine-grained update/render cycle
* Components are owned by the application state and may interact with each other using events
* Macros to write SVG and HTML in-line with Rust code
* Most tasks can be accomplished using pure-rust. If required, injecting and calling js is supported.
* Built-in performance metrics
* Spawning system dialogs
* This crate does not itself implement a frontend. A frontend is implemented in `greenhorn_web_view`.
  It makes use of [web_view](https://github.com/Boscop/web-view) and [tinyfiledialogs-rs](https://github.com/jdm/tinyfiledialogs-rs).

## Example

```rust
use greenhorn::prelude::*;
use greenhorn::html;

struct MyApp {
    text: String,
}

enum MyMsg {
    Clicked(DomEvent),
    KeyDown(DomEvent),
}

impl Render for MyApp {
    type Message = MyMsg;

    fn render(&self) -> Node<Self::Message> {
        html!(
            <div #my-app>
                <button type="button"
                        @keydown={MyMsg::KeyDown}
                        @mousedown={MyMsg::Clicked}>
                    {&self.text}
                </>
            </>
        ).into()
    }
}

impl App for MyApp {
    fn update(&mut self,msg: Self::Message,ctx: Context<Self::Message>) -> Updated {
        match msg {
            MyMsg::Clicked(evt) => self.text = "Button clicked!".into(),
            MyMsg::KeyDown(evt) => self.text = "Button keypress!".into()
        }
        Updated::yes()
    }

}
```

## Acknowledgments

The concpet of this library is not at all new. It was already implemented before at least by the
[Threepenny-GUI library](https://github.com/HeinrichApfelmus/threepenny-gui).
The API was inspired by the many great rust frontend libraries:
 * [Yew](https://github.com/yewstack/yew)
 * [Seed-rs](https://github.com/seed-rs/seed)


