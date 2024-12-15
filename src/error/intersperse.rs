pub(super) struct Intersperse<I: Iterator> {
    iter: I,
    next_item: Option<I::Item>,
    separator: I::Item,
}

impl<I> Intersperse<I>
where
    I: Iterator,
{
    pub(super) fn new(mut iter: I, separator: I::Item) -> Self {
        Self {
            next_item: iter.next(),
            iter,
            separator,
        }
    }
}

impl<I> Iterator for Intersperse<I>
where
    I: Iterator,
    I::Item: Clone,
{
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(next) = self.next_item.take() {
            Some(next)
        } else {
            self.next_item = self.iter.next();
            if self.next_item.is_some() {
                Some(self.separator.clone())
            } else {
                None
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Intersperse;
    use std::iter;

    #[test]
    fn empty_iterator() {
        assert_eq!(
            Intersperse::new(iter::empty(), 'a').collect::<Vec<_>>(),
            vec![]
        );
    }

    #[test]
    fn single_element_iterator() {
        assert_eq!(
            Intersperse::new(iter::once('b'), 'a').collect::<Vec<_>>(),
            vec!['b']
        );
    }

    #[test]
    fn multiple_element_iterator() {
        assert_eq!(
            Intersperse::new(['b', 'c', 'd'].into_iter(), 'a').collect::<Vec<_>>(),
            vec!['b', 'a', 'c', 'a', 'd']
        );
    }
}
