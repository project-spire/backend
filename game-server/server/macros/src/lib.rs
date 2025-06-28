mod status_effect;

use proc_macro::TokenStream;
use syn::parse_macro_input;

#[proc_macro_derive(StatusEffect, attributes(buff, debuff, passive, curse))]
pub fn derive_status_effect(input: TokenStream) -> TokenStream {
    let input =  parse_macro_input!(input);
    status_effect::expand_derive_status_effect(input)
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}