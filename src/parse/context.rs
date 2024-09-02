use std::vec;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum Segment {
    Identifier(&'static str),
    Value(Vec<u8>),
    Context(Context),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Context {
    pub(crate) segments: Vec<Segment>,
}

impl IntoIterator for Context {
    type IntoIter = ContextIter;
    type Item = Segment;

    fn into_iter(self) -> Self::IntoIter {
        ContextIter {
            segments: self.segments.into_iter(),
        }
    }
}

#[derive(Debug)]
pub(crate) struct ContextIter {
    segments: vec::IntoIter<Segment>,
}

impl Iterator for ContextIter {
    type Item = Segment;

    fn next(&mut self) -> Option<Self::Item> {
        self.segments.next()
    }
}

#[cfg(test)]
mod tests {
    use super::{
        Context,
        Segment,
    };

    #[test]
    fn iter_empty() {
        assert_eq!(
            Context { segments: vec![] }.into_iter().collect::<Vec<_>>(),
            vec![]
        );
    }

    #[test]
    fn iter_non_empty() {
        assert_eq!(
            Context {
                segments: vec![
                    Segment::Identifier("foo"),
                    Segment::Value("bar".into()),
                    Segment::Context(Context {
                        segments: vec![Segment::Value("baz".into())],
                    })
                ],
            }
            .into_iter()
            .collect::<Vec<_>>(),
            vec![
                Segment::Identifier("foo"),
                Segment::Value("bar".into()),
                Segment::Context(Context {
                    segments: vec![Segment::Value("baz".into())],
                })
            ]
        );
    }
}
