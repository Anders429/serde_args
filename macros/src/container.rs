use proc_macro2::Span;
use syn::{
    parse,
    parse::{Parse, ParseStream},
    Item, ItemEnum, ItemStruct,
};

pub(crate) enum Container {
    Struct(ItemStruct),
    Enum(ItemEnum),
}

impl Parse for Container {
    fn parse(input: ParseStream) -> parse::Result<Self> {
        match Item::parse(input)? {
            Item::Struct(r#struct) => Ok(Self::Struct(r#struct)),
            Item::Enum(r#enum) => Ok(Self::Enum(r#enum)),
            item => Err(parse::Error::new(
                Span::call_site(),
                format!("cannot use `serde_args::help` macro on {:?} item", item),
            )),
        }
    }
}
