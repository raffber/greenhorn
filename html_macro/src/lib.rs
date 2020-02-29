///
/// This crates complements greenhorn by adding a html!{} macro
/// to simplify creating view functions:
///
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
/// ```
///


mod tree;
mod matches;

extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro_hack::proc_macro_hack;
use syn::parse_macro_input;
use quote::quote;
use tree::Element;
use proc_macro_error::proc_macro_error;


#[proc_macro_error]
#[proc_macro_hack]
pub fn html(input: TokenStream) -> TokenStream {
    let root = syn::parse::<Element>(input);
    println!("html - macro input parsed");
    println!("---------------------------");
    TokenStream::from(quote! { 1 + 1 })
}

