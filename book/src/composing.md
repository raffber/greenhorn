# Composing Applications with Components 


## Composing by Calling Functions in `render()`


## Composing by Nesting `Render` Types


## Composing by Nesting `Render + App` Types


## Composing with `Component<>`

All composition patterns presented above enable effective code re-use.
However, the runtime is not able to distinguish which part of the DOM has been updated.
As a consequence, the runtime still has to re-build the whole DOM and run the DOM diff algorithm to format a patch for the frontend to apply.

As the application and the DOM grow, the runtime has to do a lot of work for very few DOM changes.
Also, most updates are contained deep within the composition hierarchy. Thus, if the runtime knows more about our composition structure it could only re-build the part of the DOM where the update as occured.

`greenhorn` ships with a special wrapper type which facilitates this - `Component<T: App + Render>`:

```rust

```


## Choosing a composition pattern

