use proc_macro2::{Span, TokenStream};
use quote::ToTokens;
use syn::{
    parse,
    parse::{Parse, ParseStream},
    Item, ItemEnum, ItemStruct,
};

#[derive(Clone, Debug)]
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

impl ToTokens for Container {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::Struct(r#struct) => r#struct.to_tokens(tokens),
            Self::Enum(r#enum) => r#enum.to_tokens(tokens),
        }
    }
}
