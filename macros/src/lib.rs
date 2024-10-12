mod attributes;
mod container;
mod generate;
mod help;
#[cfg(test)]
mod test;
mod version;

use container::Container;
use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn help(_attr: TokenStream, item: TokenStream) -> TokenStream {
    help::process(item.into()).into()
}

#[proc_macro_attribute]
pub fn version(_attr: TokenStream, item: TokenStream) -> TokenStream {
    version::process(item.into()).into()
}
