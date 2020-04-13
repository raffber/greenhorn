#![recursion_limit="512"]

use greenhorn::prelude::*;
use greenhorn::{html, Id};


pub struct MainApp {
    todos: Vec<Todo>,
    filter: Filter,
}

#[derive(Debug)]
struct Todo {
    id: Id,
    title: String,
    completed: bool,
    editing: bool,
}

pub enum MainMsg {
    // add todo field
    NewTodoKeyUp(DomEvent),

    // messages from the item editor
    TodoStartEdit(Id),
    RemoveTodo(Id),
    TodoEditDone(Id),
    TodoInputKeyUp(Id, DomEvent),
    TodoChanged(Id, DomEvent),
    TodoToggle(Id),

    // footer buttons
    SetFilter(Filter),
    RemoveCompleted,
}

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
        html!(
            <li class={if self.editing { "todo editing" } else { "todo" }}>
                <div .view>
                    <input .toggle type="checkbox" @click={move |_| MainMsg::TodoToggle(id)} checked={self.completed}/>
                    <label @dblclick={move |_| MainMsg::TodoStartEdit(id)}>{&self.title}</>
                    <button .destroy @click={move |_| MainMsg::RemoveTodo(id)} />
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
                let value = evt.target_value().get_text().unwrap();
                let evt = evt.into_keyboard().unwrap();
                if evt.key == "Enter" && !value.is_empty() {
                    let todo = Todo::new(value);
                    ctx.run_js("document.getElementById('new-todo').value = ''");
                    self.todos.push(todo);
                }
            },

            MainMsg::TodoStartEdit(id) => { self.todo_mut(id).editing = true; },

            MainMsg::RemoveTodo(id) => { self.remove_todo(id); },

            MainMsg::TodoEditDone(id) => { self.todo_mut(id).editing = false; },

            MainMsg::TodoInputKeyUp(id, evt) => {
                if evt.into_keyboard().unwrap().key == "Enter" {
                    self.todo_mut(id).editing = false;
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
        }
        Updated::yes()
    }
}

impl MainApp {
    pub fn new() -> Self {
        Self {
            todos: vec![],
            filter: Filter::All,
        }
    }

    fn remove_todo(&mut self, id: Id) {
        let mut rm_idx = None;
        for (k, v) in self.todos.iter().enumerate() {
            if v.id == id {
                rm_idx = Some(k);
            }
        }
        rm_idx.map(|x| self.todos.remove(x));
    }

    fn todo_mut(&mut self, id: Id) -> &mut Todo {
        self.todos.iter_mut().filter(|x| x.id == id).next().unwrap()
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

        let app = html!(
            <header .header>
                <h1>{"todos"}</>
                <input #new-todo .new-todo autofocus="" autocomplete="off"
                    placeholder="What needs to be done?" @keyup={MainMsg::NewTodoKeyUp} />
            </>
            <section class={if self.todos.len() > 0 { "main" } else { "main hide" } }>
                <input #toggle-all .toggle-all type="checkbox" />
                <label>{"Mark all as complete"}</>
                <ul .todo-list>
                    {filtered_todos}
                </>
            </>
            <footer class={format!("footer {}", choose(self.todos.len() > 0, "", "hide"))}>
                <span .todo-count>
                    <strong>{format!("{}", active_count)}</>
                    {format!(" item{} left", choose(active_count != 1, "s", ""))}
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
                    class={format!("clear-completed {}", choose(completed_count > 0, "", "hide"))}
                    @click={|_| MainMsg::RemoveCompleted}>
                        {format!("Clear completed ({})", completed_count)}
                </>
            </>
        );

        html!(
            <div>
                <section .todoapp> {app} </>
                <footer class="info">
                    <p>{"Double-click to edit a todo"}</>
                    <p>{"Written by Raphael Bernhard"}</>
                </>
            </>
        ).into()
    }
}

fn choose<T>(cond: bool, if_true: T, if_false: T) -> T {
    if cond { if_true } else { if_false }
}
