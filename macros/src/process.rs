use crate::{extract, generate};
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{parse2 as parse, Ident, Item};

macro_rules! return_error {
    ($result: expr) => {
        match $result {
            Ok(value) => value,
            Err(error) => return error,
        }
    };
}

pub(super) fn process(item: TokenStream) -> TokenStream {
    // Parse the descriptions from the container.
    let container = match parse(item.clone()) {
        Ok(container) => container,
        Err(error) => return error.into_compile_error(),
    };
    let parsed_item: Item = match parse(item.clone()) {
        Ok(item) => item,
        Err(error) => return error.into_compile_error(),
    };
    let descriptions = extract::descriptions(&container);
    let visibility = extract::visibility(&container);
    let ident = extract::identifier(&container);

    // Extract the container.
    let phase_1 = return_error!(generate::phase_1(parsed_item.clone(), ident));
    let phase_2 = return_error!(generate::phase_2(parsed_item.clone(), descriptions, ident));
    let phase_3 = return_error!(generate::phase_3(parsed_item.clone()));

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
