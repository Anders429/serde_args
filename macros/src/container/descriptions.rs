use syn::{Attribute, Expr, Meta};

#[derive(Debug, Eq, PartialEq)]
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

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct Descriptions<'a> {
    pub(crate) container: Documentation<'a>,
    pub(crate) keys: Vec<Documentation<'a>>,
}
