use std::{
    ffi::OsString,
    str,
};
use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug, Eq, PartialEq)]
pub(super) enum Token {
    Positional(Vec<u8>),
    Optional(Vec<u8>),
    EndOfOptions,
}

pub(super) struct ParsedArgs<Args> {
    args: Args,
    pub(super) revisit: Option<Vec<u8>>,
    pub(super) consumed_token: bool,
}

impl<Args> ParsedArgs<Args> {
    pub(super) fn new(args: Args) -> Self {
        Self {
            args,
            revisit: None,
            consumed_token: false,
        }
    }
}

impl<Args> ParsedArgs<Args>
where
    Args: Iterator<Item = OsString>,
{
    pub(super) fn next_token(&mut self) -> Option<Token> {
        if let Some(token) = self.next() {
            if let Some(short_token) = token.strip_prefix(b"-") {
                if short_token.is_empty() {
                    // A single `-` is an empty optional token.
                    Some(Token::Optional(Vec::new()))
                } else if let Some(long_token) = short_token.strip_prefix(b"-") {
                    if long_token.is_empty() {
                        Some(Token::EndOfOptions)
                    } else {
                        Some(Token::Optional(long_token.to_vec()))
                    }
                } else {
                    // This is only an option if there is a single character.
                    if short_token.len() > 4 {
                        Some(Token::Positional(token))
                    } else if let Ok(short_token_str) = str::from_utf8(short_token) {
                        if short_token_str.graphemes(true).count() == 1 {
                            Some(Token::Optional(short_token.to_vec()))
                        } else {
                            Some(Token::Positional(token))
                        }
                    } else {
                        Some(Token::Positional(token))
                    }
                }
            } else {
                Some(Token::Positional(token))
            }
        } else {
            None
        }
    }

    pub(super) fn next_positional(&mut self) -> Option<Vec<u8>> {
        self.next()
    }

    pub(super) fn next_optional(&mut self) -> Option<Vec<u8>> {
        if let Some(token) = self.next_token() {
            match token {
                Token::Optional(token) => Some(token),
                Token::EndOfOptions => None,
                Token::Positional(token) => {
                    self.revisit = Some(token);
                    None
                }
            }
        } else {
            None
        }
    }
}

impl<Args> Iterator for ParsedArgs<Args>
where
    Args: Iterator<Item = OsString>,
{
    type Item = Vec<u8>;

    fn next(&mut self) -> Option<Self::Item> {
        let value = self
            .revisit
            .take()
            .or_else(|| self.args.next().map(|os_str| os_str.into_encoded_bytes()));
        if value.is_some() {
            self.consumed_token = true;
        }
        value
    }
}

#[cfg(test)]
mod tests {
    use super::{
        ParsedArgs,
        Token,
    };
    use claims::{
        assert_none,
        assert_some,
        assert_some_eq,
    };
    use std::ffi::OsString;

    #[test]
    fn next_token_none() {
        let mut args = ParsedArgs::new([].into_iter());

        assert_none!(args.next_token());
    }

    #[test]
    fn next_token_short_option() {
        let mut args = ParsedArgs::new([OsString::from("-h")].into_iter());

        assert_some_eq!(args.next_token(), Token::Optional("h".into()));
    }

    #[test]
    fn next_token_short_option_grapheme() {
        let mut args = ParsedArgs::new([OsString::from("-ã")].into_iter());

        assert_some_eq!(args.next_token(), Token::Optional("ã".into()));
    }

    #[test]
    fn next_token_long_option() {
        let mut args = ParsedArgs::new([OsString::from("--help")].into_iter());

        assert_some_eq!(args.next_token(), Token::Optional("help".into()));
    }

    #[test]
    fn next_token_positional() {
        let mut args = ParsedArgs::new([OsString::from("foo")].into_iter());

        assert_some_eq!(args.next_token(), Token::Positional("foo".into()));
    }

    #[test]
    fn next_token_positional_leading_dash() {
        let mut args = ParsedArgs::new([OsString::from("-foo")].into_iter());

        assert_some_eq!(args.next_token(), Token::Positional("-foo".into()));
    }

    #[test]
    fn next_token_end_of_options() {
        let mut args = ParsedArgs::new([OsString::from("--")].into_iter());

        assert_some_eq!(args.next_token(), Token::EndOfOptions);
    }

    #[test]
    fn next_positional() {
        let mut args = ParsedArgs::new([OsString::from("foo")].into_iter());

        assert_some_eq!(args.next_positional(), b"foo");
    }

    #[test]
    fn next_positional_none() {
        let mut args = ParsedArgs::new([].into_iter());

        assert_none!(args.next_positional());
    }

    #[test]
    fn next_positional_leading_dash() {
        let mut args = ParsedArgs::new([OsString::from("-h")].into_iter());

        assert_some_eq!(args.next_positional(), b"-h");
    }

    #[test]
    fn next_positional_leading_dashes() {
        let mut args = ParsedArgs::new([OsString::from("--help")].into_iter());

        assert_some_eq!(args.next_positional(), b"--help");
    }

    #[test]
    fn next_positional_double_dash() {
        let mut args = ParsedArgs::new([OsString::from("--")].into_iter());

        assert_some_eq!(args.next_positional(), b"--");
    }

    #[test]
    fn next_optional_none() {
        let mut args = ParsedArgs::new([].into_iter());

        assert_none!(args.next_optional());
    }

    #[test]
    fn next_optional_short() {
        let mut args = ParsedArgs::new([OsString::from("-h")].into_iter());

        assert_some_eq!(args.next_optional(), b"h");
    }

    #[test]
    fn next_optional_long() {
        let mut args = ParsedArgs::new([OsString::from("--help")].into_iter());

        assert_some_eq!(args.next_optional(), b"help");
    }

    #[test]
    fn next_optional_end_of_options() {
        let mut args = ParsedArgs::new([OsString::from("--")].into_iter());

        assert_none!(args.next_optional());
    }

    #[test]
    fn next_optional_positional() {
        let mut args = ParsedArgs::new([OsString::from("foo")].into_iter());

        assert_none!(args.next_optional());
        // Ensure we revisit the positional argument.
        assert_some_eq!(args.next_positional(), b"foo");
    }

    #[test]
    fn next_none() {
        let mut args = ParsedArgs::new([].into_iter());

        assert_none!(args.next());
    }

    #[test]
    fn next() {
        let mut args = ParsedArgs::new(["foo".into()].into_iter());

        assert_some_eq!(args.next(), b"foo");
    }

    #[test]
    fn next_revisit() {
        let mut args = ParsedArgs::new(["foo".into()].into_iter());
        args.revisit = Some("bar".into());

        assert_some_eq!(args.next(), b"bar");
    }

    #[test]
    fn next_consumed_token() {
        let mut args = ParsedArgs::new(["foo".into()].into_iter());

        assert_some!(args.next());
        assert!(args.consumed_token);
    }
}
