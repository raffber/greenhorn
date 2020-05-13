# Introduction

Greenhorn is a rust library for building desktop applications with web technologies in (almost)
pure rust.

This is accomplished by separating the application into a server-side process
(the backend) and web view implemented in javascript (the frontend).
While most HTML-based desktop applications leave state synchronization
between frontend and backend up to the application logic, this library synchronizes its state at DOM-level.
Thus, the user may implement the application logic purely in the backend using rust and the DOM is automatically synchronized with the frontend.
This facilitates the integration of a desktop GUI with system services and simplifies application development considerably.

## Deploying

The greenhorn library is not opinionated on how to deploy applications. However, the projects ships additional facilities to deploy applications to common targets.

 * Desktop applications may be deployed using [Boscop's WebView](https://github.com/Boscop/web-view).
The `greenhorn_web_view` crates serves as an adapter for it. This is the easiest deployment method for desktop operating systems.
Refer to this [page for details](./deploy_webview.md).

 * greenhorn application may be deployed in a webbrowser. Refer to [this page for details](./deploy_wasm.m)
 and check out [this example](https://github.com/raffber/greenhorn/tree/master/examples/todomvc/wasm).

 * Desktop applications may also be deployed with electron. An example is still pending.

## Examples

A `todomvc` example is presented in the repository at `examples/todomvc`:

 * The core applications logic is located in the `lib` crate.
 * A deployment example for [WebView](https://github.com/Boscop/web-view) is given in the `webview` crate.
 * 


