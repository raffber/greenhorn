#![recursion_limit="512"]

use greenhorn::prelude::*;
use greenhorn::{html, Id};

#[derive(Debug)]
struct Todo {
    id: Id,
    title: String,
    completed: bool,
    editing: bool,
}

impl Todo {
    fn check_filter(&self, filter: &Visibility) -> bool {
        match filter {
            Visibility::All => true,
            Visibility::Active => !self.completed,
            Visibility::Completed => self.completed,
        }
    }


    fn render(&self) -> Node<MainMsg> {
        let id = self.id;
        html!(
            <li .todo>
                <div .view>
                    <input .toggle type="checkbox" @click={move |_| MainMsg::Toggle(id)} checked={self.completed}/>
                    <label @dblclick={move |_| MainMsg::TodoDblClick(id)}>{&self.title}</>
                    <button .destroy @click={move |_| MainMsg::RemoveTodo(id)} />
                </>
                <input .edit type="text"
                    @focus={move |_| MainMsg::TodoInputFocus(id)}
                    @blur={move |_| MainMsg::DoneEdit(id)}
                    @keyup={move |evt| MainMsg::TodoInputKeyUp(id, evt)}
                    @change={move |evt| MainMsg::TodoChanged(id, evt)}
                    />
            </>
        ).into()
    }
}

pub struct MainApp {
    todos: Vec<Todo>,
    visibility: Visibility,
}

pub enum MainMsg {
    NewTodoKeyUp(DomEvent),
    TodoDblClick(Id),
    RemoveTodo(Id),
    DoneEdit(Id),
    TodoInputKeyUp(Id, DomEvent),
    TodoChanged(Id, DomEvent),
    TodoInputFocus(Id),
    Toggle(Id),
    FilterAll,
    FilterActive,
    FilterCompleted,
    RemoveCompleted,
}

enum Visibility {
    All,
    Active,
    Completed,
}

impl App for MainApp {
    fn update(&mut self, msg: Self::Message, ctx: Context<Self::Message>) -> Updated {
        match msg {
            MainMsg::NewTodoKeyUp(evt) => {
                let value = evt.target_value().get_text().unwrap();
                let evt = evt.into_keyboard().unwrap();
                if evt.key == "Enter" && !value.is_empty() {
                    self.add_todo(value, ctx)
                }
            },
            MainMsg::TodoDblClick(id) => {
                self.get_todo_mut(id).unwrap().editing = true;
            },
            MainMsg::RemoveTodo(id) => {
                self.remove_todo(id);
            },
            MainMsg::DoneEdit(id) => {
                self.get_todo_mut(id).unwrap().editing = false;
            },
            MainMsg::TodoInputKeyUp(_, _) => {},
            MainMsg::FilterAll => {
                self.visibility = Visibility::All;
            },
            MainMsg::FilterActive => {
                self.visibility = Visibility::Active;
            },
            MainMsg::FilterCompleted => {
                self.visibility = Visibility::Completed;
            },
            MainMsg::RemoveCompleted => {
                self.todos = self.todos.drain(..)
                    .filter(|x| !x.completed)
                    .collect::<Vec<_>>();
            },
            MainMsg::TodoChanged(id, evt) => {
                self.get_todo_mut(id).unwrap().title = evt.target_value().get_text().unwrap();
            }
            MainMsg::TodoInputFocus(_) => {

            }
            MainMsg::Toggle(id) => {
                let todo = self.get_todo_mut(id).unwrap();
                println!("{}", todo.completed);
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
            visibility: Visibility::All,
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

    fn add_todo(&mut self, value: String, ctx: Context<MainMsg>) {
        let todo = Todo {
            id: Default::default(),
            title: value,
            completed: false,
            editing: false
        };
        ctx.run_js("document.getElementById('new-todo').value = ''");
        self.todos.push(todo);
    }

    fn get_todo(&self, id: Id) -> Option<&Todo> {
        self.todos.iter().filter(|x| x.id == id).next()
    }

    fn get_todo_mut(&mut self, id: Id) -> Option<&mut Todo> {
        self.todos.iter_mut().filter(|x| x.id == id).next()
    }

    fn footer() -> Node<MainMsg> {
        html!(
            <footer class="info">
                <p>{"Double-click to edit a todo"}</>
                <p>{"Written by Raphael Bernhard"}</>
            </>
        ).into()
    }

}

impl Render for MainApp {
    type Message = MainMsg;

    fn render(&self) -> Node<Self::Message> {
        let cls_clear_completed = if self.todos.len() > 0 { "clear-completed" } else { "clear-completed hide" };

        let todos = self.todos.iter()
            .filter(|x| x.check_filter(&self.visibility))
            .map(|x| x.render())
            .collect::<Vec<_>>();
        let todos_len = todos.len();

        let app = html!(
            <section .todoapp>
                <header .header>
                    <h1>{"todos"}</>
                    <input #new-todo .new-todo autofocus="" autocomplete="off" placeholder="What needs to be done?"
                        @keyup={MainMsg::NewTodoKeyUp}
                         />
                </>
                <section class={if todos.len() > 0 { "main" } else { "main hide" } }>
                    <input #toggle-all .toggle-all type="checkbox" />
                    <label>{"Mark all as complete"}</>
                    <ul .todo-list>
                        {todos}
                    </>
                </>
                <footer .footer>
                    <span .todo-count>
                        <strong>
                            {format!("Remaining: {} ", todos_len)}
                            {if todos_len > 1 { "items left" } else {"item left"} }
                        </>
                    </>
                    <ul .filters>
                        <li><a href="#" @click={|_| MainMsg::FilterAll}>All</></>
                        <li><a href="#" @click={|_| MainMsg::FilterActive}>Active</></>
                        <li><a href="#" @click={|_| MainMsg::FilterCompleted}>Completed</></>
                    </>
                    <button class={cls_clear_completed}  @click={|_| MainMsg::RemoveCompleted}>
                        {"Clear completed"}
                    </>
                </>
            </>
        );

        html!(
            <div>
                {app}
                {MainApp::footer()}
            </>
        ).into()
    }
}
