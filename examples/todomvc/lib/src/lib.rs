#![recursion_limit="512"]
use greenhorn::prelude::*;
use greenhorn::html;
use greenhorn::components::{checkbox, TextInput, TextInputMsg};

pub const CSS: &'static str = include_str!("../dist/styles.css");

pub struct MainApp {
    todos: Vec<Todo>,
    filter: Filter,
    new_todo: TextInput,
}

#[derive(Debug)]
struct Todo {
    id: Id,
    title: String,
    completed: bool,
    editing: bool,
}

pub enum MainMsg {
    NewTodoMsg(TextInputMsg),
    NewTodoKeyUp(DomEvent),
    SelectAll,
    SetFilter(Filter),
    RemoveCompleted,

    // item messages
    TodoStartEdit(Id),
    TodoRemove(Id),
    TodoEditDone(Id),
    TodoInputKeyUp(Id, DomEvent),
    TodoChanged(Id, DomEvent),
    TodoToggle(Id),
}

#[derive(Clone)]
pub enum Filter {
    All,
    Active,
    Completed,
}

impl Todo {
    fn new(title: String) -> Self {
        Self {
            id: Default::default(),
            title,
            completed: false,
            editing: false
        }
    }

    fn check_filter(&self, filter: &Filter) -> bool {
        match filter {
            Filter::All => true,
            Filter::Active => !self.completed,
            Filter::Completed => self.completed,
        }
    }

    fn render(&self) -> Node<MainMsg> {
        let id = self.id;
        let text_input: Node<_> = if self.editing {
            html!(<input .edit type="text"
                    value={&self.title}
                    @blur={move |_| MainMsg::TodoEditDone(id)}
                    @keyup={move |evt| MainMsg::TodoInputKeyUp(id, evt)}
                    @change={move |evt| MainMsg::TodoChanged(id, evt)}
                    $render="event.target.focus()"
                />).into()
        } else {
            html!(<input .edit type="hidden" />).into()
        };
        let mut class = "todo".to_string();
        if self.editing {
            class.push_str(" editing");
        }
        if self.completed {
            class.push_str(" completed");
        }

        html!(
            <li class={&class}>
                <div .view>
                    {checkbox(self.completed, move || MainMsg::TodoToggle(id)).class("toggle")}
                    <label @dblclick={move |_| MainMsg::TodoStartEdit(id)}>{&self.title}</>
                    <button .destroy @click={move |_| MainMsg::TodoRemove(id)} />
                </>
                {text_input}
            </>
        ).into()
    }
}

impl App for MainApp {
    fn update(&mut self, msg: Self::Message, ctx: Context<Self::Message>) -> Updated {
        match msg {
            MainMsg::NewTodoKeyUp(evt) => {
                let evt = evt.into_keyboard().unwrap();
                if evt.key == "Enter" && !self.new_todo.get().is_empty() {
                    let todo = Todo::new(self.new_todo.get().to_string());
                    self.new_todo.set("");
                    self.todos.push(todo);
                }
            },

            MainMsg::TodoStartEdit(id) => { self.todo_mut(id).editing = true; },

            MainMsg::TodoRemove(id) => { self.remove_todo(id); },

            MainMsg::TodoEditDone(id) => { self.todo_edit_done(id); },

            MainMsg::TodoInputKeyUp(id, evt) => {
                if evt.into_keyboard().unwrap().key == "Enter" {
                    self.todo_edit_done(id);
                }
            },

            MainMsg::SetFilter(filter) => { self.filter = filter; },

            MainMsg::RemoveCompleted => {
                self.todos = self.todos.drain(..)
                    .filter(|x| !x.completed)
                    .collect::<Vec<_>>();
            },

            MainMsg::TodoChanged(id, evt) => {
                self.todo_mut(id).title = evt.target_value().get_text().unwrap();
            }

           MainMsg::TodoToggle(id) => {
               let todo = self.todo_mut(id);
               todo.completed = !todo.completed;
            }
            MainMsg::SelectAll => {
                let cur_state = self.todos.iter()
                    .filter(|x| x.check_filter(&self.filter))
                    .all(|x| x.completed);
                let new_state = !cur_state;
                let filter = self.filter.clone();
                for todo in self.todos.iter_mut()
                        .filter(|x| x.check_filter(&filter)) {
                    todo.completed = new_state;
                }
            }
            MainMsg::NewTodoMsg(msg) => {self.new_todo.update(msg, &ctx)}
        }
        Updated::yes()
    }
}

impl MainApp {
    pub fn new() -> Self {
        Self {
            todos: vec![],
            filter: Filter::All,
            new_todo: Default::default()
        }
    }

    fn remove_todo(&mut self, id: Id) {
        let rm_idx = self.todos.iter().enumerate()
            .find(|(_,v)| v.id == id)
            .map(|(k,_)| k);
        rm_idx.map(|x| self.todos.remove(x));
    }

    fn todo_mut(&mut self, id: Id) -> &mut Todo {
        self.todos.iter_mut().filter(|x| x.id == id).next().unwrap()
    }

    fn todo_edit_done(&mut self, id: Id) {
        let todo = self.todo_mut(id);
        todo.title = todo.title.trim().to_string();
        todo.editing = false;
    }
}

impl Render for MainApp {
    type Message = MainMsg;

    fn render(&self) -> Node<Self::Message> {
        let filtered_todos = self.todos.iter()
            .filter(|x| x.check_filter(&self.filter))
            .map(|x| x.render())
            .collect::<Vec<_>>();

        let active_count = self.todos.iter().filter(|x| !x.completed).count();
        let completed_count = self.todos.iter().filter(|x| x.completed).count();

        let all_filtered_done = self.todos.iter()
            .filter(|x| x.check_filter(&self.filter))
            .all(|x| x.completed);

        let new_todo = self.new_todo.render(MainMsg::NewTodoMsg)
            .class("new-todo").attr("autofocus", "").attr("autocomplete", "off")
            .attr("placeholder", "What needs to be done?").on("keyup", MainMsg::NewTodoKeyUp);

        let app = html!(<section .todoapp>
            <header .header>
                <h1>{"todos"}</>
                {new_todo}
            </>
            <section class={if self.todos.len() > 0 { "main" } else { "main hide" } }>
                {checkbox(all_filtered_done, || MainMsg::SelectAll).class("toggle-all").id("toggle-all")}
                <label for="toggle-all">{"Mark all as complete"}</>
                <ul .todo-list> {filtered_todos} </>
            </>
            <footer class={format!("footer {}", if self.todos.len() > 0 {""} else {"hide"})}>
                <span .todo-count>
                    <strong>{format!("{}", active_count)}</>
                    {format!(" item{} left", if active_count != 1 {"s"} else {""})}
                </>
                <ul .filters>
                    <li><a href="#" class={if matches!(self.filter, Filter::All) { "selected" } else { "" }}
                        @click={|_| MainMsg::SetFilter(Filter::All)}>All</></>
                    <li><a href="#" class={if matches!(self.filter, Filter::Active) { "selected" } else { "" }}
                        @click={|_| MainMsg::SetFilter(Filter::Active)}>Active</></>
                    <li><a href="#" class={if matches!(self.filter, Filter::Completed) { "selected" } else { "" }}
                        @click={|_| MainMsg::SetFilter(Filter::Completed)}>Completed</></>
                </>
                <button href="#"
                    class={format!("clear-completed {}", if completed_count > 0 {""} else {"hide"})}
                    @click={|_| MainMsg::RemoveCompleted}>
                        {format!("Clear completed ({})", completed_count)}
                </>
            </>
        </>);

        html!( <div>
            {app}
            <footer class="info">
                <p>{"Double-click to edit a todo"}</>
                <p>{"Written by Raphael Bernhard"}</>
            </>
        </> ).into()
    }
}
