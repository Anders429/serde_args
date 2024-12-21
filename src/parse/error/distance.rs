use std::cmp::min;
use unicode_segmentation::UnicodeSegmentation;

pub(crate) fn levenshtein(a: &str, b: &str) -> usize {
    let b_count = b.graphemes(true).count();
    let mut cache = (1..(b_count + 1)).collect::<Vec<_>>();

    a.graphemes(true).enumerate().fold(
        b_count,
        |#[allow(unused_assignments)] mut result, (i, a_char)| {
            result = i + 1;
            let mut distance = i;
            b.graphemes(true).enumerate().for_each(|(j, b_char)| {
                let cached_distance = cache[j];
                result = min(
                    result + 1,
                    min(
                        distance + if a_char == b_char { 0 } else { 1 },
                        cached_distance + 1,
                    ),
                );
                distance = cached_distance;
                cache[j] = result;
            });
            result
        },
    )
}

#[cfg(test)]
mod tests {
    use super::levenshtein;

    #[test]
    fn levenshtein_empty() {
        assert_eq!(levenshtein("", ""), 0);
    }

    #[test]
    fn levenshtein_left_string_empty() {
        assert_eq!(levenshtein("", "foo"), 3);
    }

    #[test]
    fn levenshtein_right_string_empty() {
        assert_eq!(levenshtein("foo", ""), 3);
    }

    #[test]
    fn levenshtein_both_strings_same_length() {
        assert_eq!(levenshtein("foo", "bar"), 3);
    }

    #[test]
    fn levenshtein_substitutions_and_insertions() {
        assert_eq!(levenshtein("kitten", "sitting"), 3);
    }

    #[test]
    fn levenshtein_deletion() {
        assert_eq!(levenshtein("uniformed", "uninformed"), 1);
    }

    #[test]
    fn levenshtein_similar_start_and_end() {
        assert_eq!(levenshtein("saturday", "sunday"), 3);
    }

    #[test]
    fn levenshtein_graphemes() {
        assert_eq!(levenshtein("foo", "bãr"), 3);
    }
}
