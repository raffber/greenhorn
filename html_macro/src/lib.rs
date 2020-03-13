#![allow(dead_code)]

///
/// This crates complements greenhorn by adding a html!{} macro
/// to simplify creating view functions:
///
/// TODOs
/// ======
///
/// [ ] Parse -123 in macro attributes - lower priority
/// [ ] Error Handling
/// ```
/// ```
/// struct Main {
/// }
///
/// enum Msg {
///     MouseDown(DomEvent),
/// }
///
/// impl Main {
///     fn nested_view_function(&self) -> Node<Msg> {
///         html!{ <p>Example paragraph</p> }
///     }
/// }
///
///
/// impl Render for Main {
///     type Message = Msg;
///
///    fn render(&self) -> Node<Self::Message> {
///         let mouse_move_js_handler = "function(evt) { console.log(evt); }";
///         html!{
///             <div .class_name #my_id other_attribute="attribute_value" @mousemove=mouse_move_js_handler @mousedown={Msg::MouseDown}>
///                 Some text {mount(self.component, Msg::ComponentMsg)} and more text... and {"some rust code"}
///                 {self.nested_view_function()}
///             </div>
///         }
///     }
/// }
///
///


mod element;
mod matches;
mod primitives;
mod attributes;

extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro_hack::proc_macro_hack;
use quote::quote;
use element::Element;
use proc_macro_error::proc_macro_error;


#[proc_macro_error]
#[proc_macro_hack]
pub fn html(input: TokenStream) -> TokenStream {
    match syn::parse::<Element>(input) {
        Ok(root) => TokenStream::from(quote! {
            use greenhorn::prelude::NodeBuilder;
            #root
        }),
        Err(err) => {
            err.to_compile_error().into()
        }
    }
}

#[proc_macro_error]
#[proc_macro_hack]
pub fn svg(input: TokenStream) -> TokenStream {
    match syn::parse::<Element>(input) {
        Ok(mut root) => {
            root.setup_namespace("http://www.w3.org/2000/svg");
            TokenStream::from(quote! {
                use greenhorn::prelude::NodeBuilder;
                #root
            })
        },
        Err(err) => {
            err.to_compile_error().into()
        }
    }
}

