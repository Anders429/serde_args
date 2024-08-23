use proc_macro2::{Span, TokenStream};
use syn::{parse, Attribute, Expr, Fields, Item, Meta};

#[derive(Debug)]
pub(crate) struct Documentation<'a> {
    pub(crate) exprs: Vec<&'a Expr>,
}

impl<'a> From<&'a Vec<Attribute>> for Documentation<'a> {
    fn from(attrs: &'a Vec<Attribute>) -> Self {
        let mut exprs = Vec::new();
        for attr in attrs {
            if let Meta::NameValue(name_value) = &attr.meta {
                if let Some(ident) = name_value.path.get_ident() {
                    if *ident == "doc" {
                        exprs.push(&name_value.value);
                    }
                }
            }
        }
        Self { exprs }
    }
}

#[derive(Debug)]
pub(crate) struct Descriptions<'a> {
    pub(crate) container: Documentation<'a>,
    pub(crate) keys: Vec<Documentation<'a>>,
}

pub(crate) fn descriptions(item: &Item) -> Result<Descriptions, TokenStream> {
    match item {
        Item::Enum(item) => {
            let container = Documentation::from(&item.attrs);

            // Extract variant information.
            let mut keys = vec![];
            for variant in &item.variants {
                keys.push(Documentation::from(&variant.attrs));
            }

            Ok(Descriptions { container, keys })
        }
        Item::Struct(item) => {
            // Extract the container description from the struct's documentation.
            let container = Documentation::from(&item.attrs);

            // Extract field information.
            if let fields @ Fields::Named(_) = &item.fields {
                let mut keys = vec![];
                for field in fields {
                    keys.push(Documentation::from(&field.attrs));
                }

                Ok(Descriptions { container, keys })
            } else {
                Err(parse::Error::new(
                    Span::call_site(),
                    "cannot use `serde_args::help` on struct with non-named fields",
                )
                .into_compile_error())
            }
        }
        item => Err(parse::Error::new(
            Span::call_site(),
            format!("cannot use `serde_args::help` macro on {:?} item", item),
        )
        .into_compile_error()),
    }
}
