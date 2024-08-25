mod descriptions;

pub(crate) use descriptions::Descriptions;

use descriptions::Documentation;
use proc_macro2::{Span, TokenStream};
use quote::ToTokens;
use syn::{
    parse,
    parse::{Parse, ParseStream},
    Ident, Item, ItemEnum, ItemStruct, Visibility,
};

#[derive(Clone, Debug)]
pub(crate) enum Container {
    Struct(ItemStruct),
    Enum(ItemEnum),
}

impl Container {
    pub(crate) fn descriptions(&self) -> Descriptions {
        match self {
            Container::Enum(item) => {
                let container = Documentation::from(&item.attrs);

                // Extract variant information.
                let mut keys = vec![];
                for variant in &item.variants {
                    keys.push(Documentation::from(&variant.attrs));
                }

                Descriptions { container, keys }
            }
            Container::Struct(item) => {
                // Extract the container description from the struct's documentation.
                let container = Documentation::from(&item.attrs);

                // Extract field information.
                let mut keys = vec![];
                for field in &item.fields {
                    keys.push(Documentation::from(&field.attrs));
                }

                Descriptions { container, keys }
            }
        }
    }

    pub(crate) fn identifier(&self) -> &Ident {
        match self {
            Container::Enum(item) => &item.ident,
            Container::Struct(item) => &item.ident,
        }
    }

    pub(crate) fn visibility(&self) -> &Visibility {
        match self {
            Container::Enum(item) => &item.vis,
            Container::Struct(item) => &item.vis,
        }
    }
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
