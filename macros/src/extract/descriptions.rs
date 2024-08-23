use crate::Container;
use syn::{Attribute, Expr, Meta};

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

pub(crate) fn descriptions(container: &Container) -> Descriptions {
    match container {
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
