use proc_macro::{self, TokenStream};
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(BotState)]
pub fn mobot_derive_botstate(input: TokenStream) -> TokenStream {
    let DeriveInput { ident, .. } = parse_macro_input!(input);
    let output = quote! {
        impl mobot::handler::BotState for #ident {}
    };
    output.into()
}

#[proc_macro_derive(BotRequest)]
pub fn mobot_derive_request(input: TokenStream) -> TokenStream {
    let DeriveInput { ident, .. } = parse_macro_input!(input);
    let output = quote! {
        impl super::Request for #ident {}
    };
    output.into()
}
