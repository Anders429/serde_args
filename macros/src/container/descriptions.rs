use syn::{
    Attribute,
    Expr,
    Lit,
    Meta,
};

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct Documentation {
    pub(crate) lines: Vec<String>,
}

impl From<&Vec<Attribute>> for Documentation {
    fn from(attrs: &Vec<Attribute>) -> Self {
        let mut lines = Vec::new();
        for attr in attrs {
            if let Meta::NameValue(name_value) = &attr.meta {
                if let Some(ident) = name_value.path.get_ident() {
                    if *ident == "doc" {
                        if let Expr::Lit(literal) = &name_value.value {
                            if let Lit::Str(string) = &literal.lit {
                                lines.push(string.value().trim().to_owned());
                            }
                        }
                    }
                }
            }
        }
        Self { lines }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct Descriptions {
    pub(crate) container: Documentation,
    pub(crate) keys: Vec<Documentation>,
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
            Documentation { lines: vec![] }
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
            Documentation { lines: vec![] }
        );
    }

    #[test]
    fn documentation_from_attributes_single_doc() {
        assert_eq!(
            Documentation::from(
                &assert_ok!(parse_str::<OuterAttributes>("#[doc = \"foo bar baz\"]")).0
            ),
            Documentation {
                lines: vec!["foo bar baz".into(),]
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
                lines: vec!["foo bar baz".into(), "qux quux".into(),]
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
                lines: vec![
                    "foo bar baz".into(),
                    "qux quux".into(),
                ]
            }
        );
    }
}
