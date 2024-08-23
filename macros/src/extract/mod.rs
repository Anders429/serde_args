mod descriptions;

pub(crate) use descriptions::{descriptions, Descriptions};

use proc_macro2::{Span, TokenStream};
use syn::{parse, Ident, Item, Visibility};

pub(crate) fn identifier(item: &Item) -> Result<&Ident, TokenStream> {
    match item {
        Item::Enum(item) => Ok(&item.ident),
        Item::Struct(item) => Ok(&item.ident),
        item => Err(parse::Error::new(
            Span::call_site(),
            format!("cannot use `serde_args::help` macro on {:?} item", item),
        )
        .into_compile_error()),
    }
}

pub(crate) fn visibility(item: &Item) -> Result<&Visibility, TokenStream> {
    match item {
        Item::Enum(item) => Ok(&item.vis),
        Item::Struct(item) => Ok(&item.vis),
        item => Err(parse::Error::new(
            Span::call_site(),
            format!("cannot use `serde_args::help` macro on {:?} item", item),
        )
        .into_compile_error()),
    }
}
