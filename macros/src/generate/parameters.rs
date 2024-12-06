use proc_macro2::Span;
use syn::{
    parse,
    parse::{
        Parse,
        ParseStream,
    },
    punctuated::Punctuated,
    Ident,
    Path,
    Token,
};

#[derive(Debug, Eq, Hash, PartialEq)]
pub(super) enum Parameter {
    DocHelp,
    Version,
}

#[derive(Debug, Eq, PartialEq)]
pub(super) struct Parameters(u8);

impl Parameters {
    #[cfg(test)]
    const EMPTY: u8 = 0;
    const VERSION: u8 = 1;
    // DocHelp must be the last one returned in iteration.
    const DOC_HELP: u8 = 2;
}

impl Parse for Parameters {
    fn parse(input: ParseStream) -> parse::Result<Self> {
        let mut parameters = 0;
        for path in Punctuated::<Path, Token![,]>::parse_terminated(input)? {
            let ident = path.require_ident()?;
            if *ident == Ident::new("doc_help", Span::call_site()) {
                parameters |= Parameters::DOC_HELP;
            } else if *ident == Ident::new("version", Span::call_site()) {
                parameters |= Parameters::VERSION;
            } else {
                return Err(syn::Error::new_spanned(
                    ident,
                    "invalid parameter; expected one of `doc_help` or `version`",
                ));
            }
        }
        Ok(Self(parameters))
    }
}

impl IntoIterator for Parameters {
    type Item = Parameter;
    type IntoIter = Iter;

    fn into_iter(self) -> Self::IntoIter {
        Iter { parameters: self.0 }
    }
}

pub(super) struct Iter {
    parameters: u8,
}

impl Iterator for Iter {
    type Item = Parameter;

    fn next(&mut self) -> Option<Self::Item> {
        if self.parameters & Parameters::VERSION != 0 {
            self.parameters ^= Parameters::VERSION;
            Some(Parameter::Version)
        } else if self.parameters & Parameters::DOC_HELP != 0 {
            self.parameters ^= Parameters::DOC_HELP;
            Some(Parameter::DocHelp)
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let size = self.parameters.count_ones() as usize;
        (size, Some(size))
    }
}

impl ExactSizeIterator for Iter {}

#[cfg(test)]
mod tests {
    use super::{
        Parameter,
        Parameters,
    };
    use claims::{
        assert_err,
        assert_ok_eq,
    };
    use syn::parse_str;

    #[test]
    fn parse_empty() {
        assert_ok_eq!(parse_str::<Parameters>(""), Parameters(Parameters::EMPTY));
    }

    #[test]
    fn parse_doc_help() {
        assert_ok_eq!(
            parse_str::<Parameters>("doc_help"),
            Parameters(Parameters::DOC_HELP)
        );
    }

    #[test]
    fn parse_version() {
        assert_ok_eq!(
            parse_str::<Parameters>("version"),
            Parameters(Parameters::VERSION)
        );
    }

    #[test]
    fn parse_all() {
        assert_ok_eq!(
            parse_str::<Parameters>("doc_help, version"),
            Parameters(Parameters::DOC_HELP | Parameters::VERSION)
        );
    }

    #[test]
    fn parse_unknown() {
        assert_eq!(
            format!("{}", assert_err!(parse_str::<Parameters>("unknown"))),
            "invalid parameter; expected one of `doc_help` or `version`"
        );
    }

    #[test]
    fn iter_none() {
        assert_eq!(
            Parameters(Parameters::EMPTY)
                .into_iter()
                .collect::<Vec<_>>(),
            &[]
        );
    }

    #[test]
    fn iter_doc_help() {
        assert_eq!(
            Parameters(Parameters::DOC_HELP)
                .into_iter()
                .collect::<Vec<_>>(),
            &[Parameter::DocHelp]
        );
    }

    #[test]
    fn iter_version() {
        assert_eq!(
            Parameters(Parameters::VERSION)
                .into_iter()
                .collect::<Vec<_>>(),
            &[Parameter::Version]
        );
    }

    #[test]
    fn iter_version_doc_help() {
        // `DocHelp` should always come last.
        // This is because the `DocHelp` `expecting()` function will never return `false`.
        assert_eq!(
            Parameters(Parameters::DOC_HELP | Parameters::VERSION)
                .into_iter()
                .collect::<Vec<_>>(),
            &[Parameter::Version, Parameter::DocHelp]
        );
    }
}
