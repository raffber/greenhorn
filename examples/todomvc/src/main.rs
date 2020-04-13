#[allow(unused_imports)]

use greenhorn_web_view::ViewBuilder;
use greenhorn::prelude::Runtime;
use ::todomvc::MainApp;

fn main() {
    ViewBuilder::new()
        .css(include_str!("../dist/styles.css"))
        .title("Greenhorn - TodoMVC")
        .size(1200, 900)
        .run(move |pipe| {
            let (rt, _control) = Runtime::new(MainApp::new(), pipe);
            rt.run_blocking();
        });
}
