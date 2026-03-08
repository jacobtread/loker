use std::fmt::{Display, Write};

/// Joins the [Display] output of all the values produced by as a string
/// separated by the `sep` separator
pub fn join_iter_string<I: Display>(mut iterator: impl Iterator<Item = I>, sep: &str) -> String {
    match iterator.next() {
        None => String::new(),
        Some(first_elt) => {
            // estimate lower bound of capacity needed
            let (lower, _) = iterator.size_hint();
            let mut result = String::with_capacity(sep.len() * lower);
            write!(&mut result, "{}", first_elt).unwrap();
            iterator.for_each(|elt| {
                result.push_str(sep);
                write!(&mut result, "{}", elt).unwrap();
            });
            result
        }
    }
}

#[cfg(test)]
mod test {
    use crate::utils::string::join_iter_string;

    /// Tests an empty iterator produces an empty string
    #[test]
    fn test_join_empty_string() {
        let empty: Option<String> = None;
        let expected = "".to_string();
        let output = join_iter_string(empty.into_iter(), ",");
        assert_eq!(output, expected);
    }

    /// Tests an iterator with a single item does not produce the the separator
    #[test]
    fn test_join_string_parts_one() {
        let one: Option<String> = Some("test".to_string());
        let expected = "test".to_string();
        let output = join_iter_string(one.into_iter(), ",");
        assert_eq!(output, expected);
    }

    /// Tests an iterator with multiple items produces a single separators
    #[test]
    fn test_join_string_parts_two() {
        let parts: Vec<String> = vec!["test".to_string(), "test2".to_string()];
        let expected = "test,test2".to_string();
        let output = join_iter_string(parts.into_iter(), ",");
        assert_eq!(output, expected);
    }

    /// Tests an iterator with multiple items produces multiple separators
    #[test]
    fn test_join_string_parts_multiple() {
        let parts: Vec<String> = vec!["test".to_string(), "test2".to_string(), "test3".to_string()];
        let expected = "test,test2,test3".to_string();
        let output = join_iter_string(parts.into_iter(), ",");
        assert_eq!(output, expected);
    }
}
