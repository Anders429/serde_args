mod process;

use proc_macro::TokenStream;
use process::process;

#[proc_macro_attribute]
pub fn help(_attr: TokenStream, item: TokenStream) -> TokenStream {
    match process(item.into()) {
        Ok(tokens) => tokens,
        Err(error) => error.into_compile_error(),
    }
    .into()
}
