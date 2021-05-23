#[allow(unused_imports)]

use greenhorn_web_view::ViewBuilder;
use greenhorn::prelude::Runtime;
use ::todomvc::{MainApp, CSS};

fn main() {

    ViewBuilder::new()
        .css(CSS)
        .title("Greenhorn - TodoMVC")
        .size(1200, 900)
        .run(move |pipe| {
            async {
                let (rt, _control) = Runtime::new(MainApp::new(), pipe);
                rt.run().await;
            }
        });
}
