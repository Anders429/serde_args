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
pub fn generate(attr: TokenStream, item: TokenStream) -> TokenStream {
    generate::process(attr.into(), item.into()).into()
}
