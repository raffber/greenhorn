#![recursion_limit="256"]

use greenhorn::prelude::*;
use greenhorn::html;

struct Todo {
    title: String,
}

pub struct MainApp {
    todos: Vec<Todo>,
    remaining: u32,
    visibility: Visibility,
}

pub enum MainMsg {
    NewTodoKeyUp(DomEvent),
    TodoDblClick(usize),
    RemoveTodo(usize),
    DoneEdit(usize),
    TodoInputKeyUp(usize, DomEvent),
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
    fn update(&mut self, msg: Self::Message, _ctx: Context<Self::Message>) -> Updated {
        match msg {
            MainMsg::NewTodoKeyUp(_) => {},
            MainMsg::TodoDblClick(_) => {},
            MainMsg::RemoveTodo(_) => {},
            MainMsg::DoneEdit(_) => {},
            MainMsg::TodoInputKeyUp(_, _) => {},
            MainMsg::FilterAll => {},
            MainMsg::FilterActive => {},
            MainMsg::FilterCompleted => {},
            MainMsg::RemoveCompleted => {},
        }
        Updated::no()
    }
}

impl MainApp {
    pub fn new() -> Self {
        Self {
            todos: vec![],
            remaining: 0,
            visibility: Visibility::All
        }
    }

    fn render_todo(&self, idx: usize) -> Node<MainMsg> {
        let todo = &self.todos[idx];
        html!(
            <li .todo>
                <div .view>
                    <input .toggle type="checkbox" />
                    <label @dblclick={move |_| MainMsg::TodoDblClick(idx)}>{&todo.title}</>
                    <button .destroy @click={move |_| MainMsg::RemoveTodo(idx)} />
                </>
                <input .edit type="text" @blur={move |_| MainMsg::DoneEdit(idx)} @keyup={move |evt| MainMsg::TodoInputKeyUp(idx, evt)} />
            </>
        ).into()
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
        let todos: Vec<_> = (0..self.todos.len()).map(|x| self.render_todo(x)).collect();

        let app = html!(
            <section .todoapp>
                <header .header>
                    <h1>{"todos"}</>
                    <input .new-todo autofocus="" autocomplete="off" placeholder="What needs to be done?" @keyup={MainMsg::NewTodoKeyUp} />
                </>
                <section .main>
                    <input #toggle-all .toggle-all type="checkbox"/>
                    <label>{"Mark all as complete"}</>
                    <ul .todo-list>
                        {todos}
                    </>
                </>
                <footer .footer>
                    <span .todo-count>
                        <strong>{format!("Remaining: {} ", self.remaining)}</>
                        {if self.remaining > 1 { "items left" } else {"item left"} }
                    </>
                    <ul .filters>
                        <li><a @click={|_| MainMsg::FilterAll}>All</></>
                        <li><a @click={|_| MainMsg::FilterActive}>Active</></>
                        <li><a @click={|_| MainMsg::FilterCompleted}>Completed</></>
                    </>
                    <button class={cls_clear_completed}  @click={|_| MainMsg::RemoveCompleted}>
                        Clear completed
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
