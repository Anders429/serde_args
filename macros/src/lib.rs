mod extract;
mod generate;
mod process;

use proc_macro::TokenStream;
use process::process;

#[proc_macro_attribute]
pub fn help(_attr: TokenStream, item: TokenStream) -> TokenStream {
    process(item.into()).into()
}
