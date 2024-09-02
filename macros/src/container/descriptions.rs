use syn::{
    Attribute,
    Expr,
    Meta,
};

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

#[cfg(test)]
mod tests {
    use super::Documentation;
    use crate::test::OuterAttributes;
    use claims::assert_ok;
    use syn::parse_str;

    #[test]
    fn documentation_from_attributes_none() {
        assert_eq!(
            Documentation::from(&vec![]),
            Documentation { exprs: vec![] }
        );
    }

    #[test]
    fn documentation_from_attributes_no_doc() {
        assert_eq!(
            Documentation::from(
                &assert_ok!(parse_str::<OuterAttributes>(
                    "#[serde(rename_all = \"kebab-case\")]"
                ))
                .0
            ),
            Documentation { exprs: vec![] }
        );
    }

    #[test]
    fn documentation_from_attributes_single_doc() {
        assert_eq!(
            Documentation::from(
                &assert_ok!(parse_str::<OuterAttributes>("#[doc = \"foo bar baz\"]")).0
            ),
            Documentation {
                exprs: vec![&assert_ok!(parse_str("\"foo bar baz\""))]
            }
        );
    }

    #[test]
    fn documentation_from_attributes_multiple_docs() {
        assert_eq!(
            Documentation::from(
                &assert_ok!(parse_str::<OuterAttributes>(
                    "#[doc = \"foo bar baz\"] #[doc = \"qux quux\"]"
                ))
                .0
            ),
            Documentation {
                exprs: vec![
                    &assert_ok!(parse_str("\"foo bar baz\"")),
                    &assert_ok!(parse_str("\"qux quux\""))
                ]
            }
        );
    }

    #[test]
    fn documentation_from_attributes_multiple_docs_non_doc_interleaved() {
        assert_eq!(
            Documentation::from(
                &assert_ok!(parse_str::<OuterAttributes>(
                    "#[doc = \"foo bar baz\"] #[serde(rename_all = \"kebab-case\")] #[doc = \"qux quux\"]"
                ))
                .0
            ),
            Documentation {
                exprs: vec![
                    &assert_ok!(parse_str("\"foo bar baz\"")),
                    &assert_ok!(parse_str("\"qux quux\""))
                ]
            }
        );
    }
}
