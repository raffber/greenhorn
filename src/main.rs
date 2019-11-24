use ::greenhorn::prelude::*;

struct Main {
    btn: Component<Button>,
}

enum ButtonMsg {
    CLicked(DomEvent),
}

struct Button {
    click_count: i32,
    clicked: Event<i32>,
}

impl App for Button {
    fn update(&mut self, msg: Self::Message, mailbox: Mailbox<Self::Message>) -> Updated {
        match msg {
            ButtonMsg::CLicked(_msg) => {
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
            .on("click", ButtonMsg::CLicked)
            .text("foo")
            .build()
    }
}

enum MainMsg {
    Clicked(i32),
    Btn(ButtonMsg),
}

impl Render for Main {
    type Message = MainMsg;

    fn render(&self) -> Node<Self::Message> {
        self.html()
            .elem("div")
            .attr("class", "Main")
            .text("Hello, World!")
            .add(self.html().mount(self.btn.clone(), MainMsg::Btn))
            .add(
                self.btn
                    .borrow()
                    .clicked
                    .subscribe(MainMsg::Clicked),
            )
            .build()
    }
}

impl App for Main {
    fn update(&mut self, msg: Self::Message, mailbox: Mailbox<Self::Message>) -> Updated {
        match msg {
            MainMsg::Btn(msg) => {
                self.btn
                    .borrow_mut()
                    .update(msg, mailbox.map(MainMsg::Btn));
            }
            MainMsg::Clicked(count) => {
                println!("Clicked: {} times", count);
            }
        }
        true.into()
    }
}

fn main() {
    println!("Hello, world!");
}
