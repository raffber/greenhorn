use wasm_bindgen::prelude::*;
use greenhorn::wasm_pipe::WasmPipe;
use greenhorn::prelude::*;
use greenhorn::platform::spawn;
use todomvc::{MainApp, CSS};

#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

pub fn set_panic_hook() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

struct CssLoad<T: App>(T);

impl<T: App> Render for CssLoad<T> {
    type Message = T::Message;

    fn render(&self) -> Node<Self::Message> {
        self.0.render()
    }
}

impl<T: App> App for CssLoad<T> {
    fn update(&mut self, msg: Self::Message, ctx: Context<Self::Message>) -> Updated {
        self.0.update(msg, ctx)
    }

    fn mount(&mut self, ctx: Context<Self::Message>) {
        ctx.load_css(CSS);
        self.0.mount(ctx);
    }
}


#[wasm_bindgen(start)]
pub fn start() {
    set_panic_hook();
    let pipe = WasmPipe::new();
    let app = CssLoad(MainApp::new());
    let (rt, _control) = Runtime::new(app, pipe);
    spawn(rt.run());
}
