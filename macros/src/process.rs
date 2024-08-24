use crate::{extract, generate};
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{parse2 as parse, Ident};

pub(super) fn process(item: TokenStream) -> TokenStream {
    // Parse the descriptions from the container.
    let container = match parse(item) {
        Ok(container) => container,
        Err(error) => return error.into_compile_error(),
    };
    let descriptions = extract::descriptions(&container);
    let visibility = extract::visibility(&container);
    let ident = extract::identifier(&container);

    // Extract the container.
    let phase_1 = generate::phase_1(container.clone(), ident);
    let phase_2 = generate::phase_2(container.clone(), descriptions, ident);
    let phase_3 = generate::phase_3(container.clone());

    // Create a module name from the identifier name.
    let module = Ident::new(&format!("__{}", ident), Span::call_site());

    // Put everything in a contained module.
    quote! {
        mod #module {
            use super::*;

            #phase_1
            #phase_2
            #phase_3
        }
        #visibility use #module::#ident;
    }
}
