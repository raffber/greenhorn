use wasm_bindgen::prelude::*;
use futures::StreamExt;
use greenhorn::wasm_pipe::WasmPipe;
use greenhorn::prelude::*;
use greenhorn::html;
use greenhorn::platform::spawn;
use greenhorn::pipe::{TxMsg, Pipe};

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

pub fn set_panic_hook() {
    // When the `console_error_panic_hook` feature is enabled, we can call the
    // `set_panic_hook` function at least once during initialization, and then
    // we will get better error messages if our code ever panics.
    //
    // For more details see
    // https://github.com/rustwasm/console_error_panic_hook#readme
    #[cfg(feature = "console_error_panic_hook")]
        console_error_panic_hook::set_once();
}


struct MyApp {
    state: i32,
}

impl Render for MyApp {
    type Message = ();

    fn render(&self) -> Node<Self::Message> {
        html!(
            <div>{"Hello, World!!"}</>
        ).into()
    }
}

impl App for MyApp {
    fn update(&mut self, _msg: Self::Message, _ctx: Context<Self::Message>) -> Updated {
        Updated::yes()
    }
}

#[wasm_bindgen]
pub fn start() {
    set_panic_hook();
    println!("test");
    let pipe = WasmPipe::new();
    let app = MyApp { state: 123 };
    let (rt, _control) = Runtime::new(app, pipe);
    spawn(rt.run());
    // let (tx, rx) = pipe.split();
    // tx.0.unbounded_send(TxMsg::Ping());
    // spawn(async move {
    //     let mut rx = rx;
    //     while let Some(x) = rx.next().await {
    //         alert("hia!");
    //     }
    // });
}

#[wasm_bindgen]
extern {
    fn alert(s: &str);
}

#[wasm_bindgen]
pub fn greet() {
    alert("foo!!!");
}