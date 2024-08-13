use std::cmp::min;

pub(crate) fn distance(a: &str, b: &str) -> usize {
    let b_count = b.chars().count();
    let mut cache = (1..(b_count + 1)).collect::<Vec<_>>();

    a.chars().enumerate().fold(
        b_count,
        |#[allow(unused_assignments)] mut result, (i, a_char)| {
            result = i + 1;
            let mut distance = i;
            b.chars().enumerate().for_each(|(j, b_char)| {
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
    use super::distance;

    #[test]
    fn distance_empty() {
        assert_eq!(distance("", ""), 0);
    }

    #[test]
    fn distance_left_string_empty() {
        assert_eq!(distance("", "foo"), 3);
    }

    #[test]
    fn distance_right_string_empty() {
        assert_eq!(distance("foo", ""), 3);
    }

    #[test]
    fn distance_both_strings_same_length() {
        assert_eq!(distance("foo", "bar"), 3);
    }

    #[test]
    fn distance_substitutions_and_insertions() {
        assert_eq!(distance("kitten", "sitting"), 3);
    }

    #[test]
    fn distance_deletion() {
        assert_eq!(distance("uniformed", "uninformed"), 1);
    }

    #[test]
    fn distance_similar_start_and_end() {
        assert_eq!(distance("saturday", "sunday"), 3);
    }
}
