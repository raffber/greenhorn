use wasm_bindgen::prelude::*;
use greenhorn::wasm_pipe::WasmPipe;
use greenhorn::prelude::*;
use greenhorn::html;
use greenhorn::platform::spawn;

#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

pub fn set_panic_hook() {
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

#[wasm_bindgen(start)]
pub fn start() {
    set_panic_hook();
    let pipe = WasmPipe::new();
    let app = MyApp { state: 123 };
    let (rt, _control) = Runtime::new(app, pipe);
    spawn(rt.run());
}
