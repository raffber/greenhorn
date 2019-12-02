use ::greenhorn::prelude::*;
use async_std::task;
use std::net::SocketAddr;
use std::str::FromStr;
use ::greenhorn::services::ElementSizeNotifier;

enum ButtonMsg {
    Clicked(DomEvent),
}

struct Button {
    click_count: i32,
    clicked: Event<i32>,
}

impl Button {
    fn new() -> Self {
        Button {
            click_count: 0,
            clicked: Event::new(),
        }
    }
}

impl App for Button {
    fn update(&mut self, msg: Self::Message, mailbox: Mailbox<Self::Message>) -> Updated {
        match msg {
            ButtonMsg::Clicked(_msg) => {
                self.click_count += 1;
                mailbox.emit(self.clicked, self.click_count);
            }
        }
        true.into()
    }
}

impl Render for Button {
    type Message = ButtonMsg;

    fn render(&self) -> Node<Self::Message> {
        self.html()
            .elem("div")
            .attr("class", "button")
            .on("click", ButtonMsg::Clicked)
            .text(format!("Clicked {} times!", self.click_count))
            .build()
    }
}

struct Main {
    btn: Component<Button>,
}

impl Main {
    fn new() -> Self {
        Main {
            btn: Component::new(Button::new()),
        }
    }
}

enum MainMsg {
    Clicked(i32),
    Btn(ButtonMsg),
    Resized( (i32,i32) )
}

impl Render for Main {
    type Message = MainMsg;

    fn render(&self) -> Node<Self::Message> {
        self.html()
            .elem("div")
            .attr("class", "Main")
            .attr("id", "size-test")
            .text("Hello, World!")
            .add(self.html().mount(&self.btn, MainMsg::Btn))
            .add(self.btn.borrow().clicked.subscribe(MainMsg::Clicked))
            .build()
    }
}

impl App for Main {
    fn update(&mut self, msg: Self::Message, mailbox: Mailbox<Self::Message>) -> Updated {
        match msg {
            MainMsg::Btn(msg) => {
                self.btn.borrow_mut().update(msg, mailbox.map(MainMsg::Btn));
            }
            MainMsg::Clicked(count) => {
                println!("Clicked: {} times", count);
            }
            MainMsg::Resized((_dx,_dy)) => {
            }
        }
        true.into()
    }

    fn mount(&mut self, mut mailbox: Mailbox<Self::Message>) {
        mailbox.spawn(ElementSizeNotifier::create("size-test"),  MainMsg::Resized)
    }
}

fn main() {
    loop {
        println!("------------------- STARTUP -----------------------------------");
        let addr = SocketAddr::from_str("127.0.0.1:44123").unwrap();
        let pipe = WebsocketPipe::build(addr).listen();
        let (rt, _control) = Runtime::new(Main::new(), pipe);
        task::block_on(rt.run());
    }
}
