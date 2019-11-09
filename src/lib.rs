mod component;

//
//
//
//struct MyApp {
//    btn: Component<Button>,
//}
//
//impl App for MyApp {
//    type Message = ();
//
//    fn render(&self) -> _ {
//        let template = template! {
//             <h1 class=foo />
//             <div>
//                { btn }
//             </div>
//        };
//        template.render(self)
//    }
//
//    fn update(&mut self, msg: _) -> Update {
//        unimplemented!()
//    }
//}
//
//
//struct Button {
//
//}
//
//impl Component for Button {
//    type Message = ();
//
//    fn update(&mut self, msg: _) -> Update {
//        unimplemented!()
//    }
//
//    fn render(&self) -> Node<_> {
//        unimplemented!()
//    }
//}
//
//
//
