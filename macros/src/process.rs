use crate::{extract, generate};
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{parse2 as parse, Ident};

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
    let parsed_item = match parse(item.clone()) {
        Ok(item) => item,
        Err(error) => return error.into_compile_error(),
    };
    let descriptions = return_error!(extract::descriptions(&parsed_item));
    let visibility = return_error!(extract::visibility(&parsed_item));
    let ident = return_error!(extract::identifier(&parsed_item));

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
