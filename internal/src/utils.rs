/// capitalize
/// Capitalize the first character in s.
/// Take borrowed str of s
/// then return capitalized String
pub fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}

/// remove_whitespace
/// Take borrow of str and remove whitespace
/// return cleaned String
pub fn remove_whitespace(s: &str) -> String {
    s.chars().filter(|c| !c.is_whitespace()).collect()
}

#[cfg(test)]
mod test {
    use super::*;
    use test_log::test;

    #[test]
    fn test_capitalize() {
        let test = "lorem ipsum dolor sit amet".to_string();
        let expected = "Lorem ipsum dolor sit amet".to_string();
        let result = capitalize(test.as_str());
        assert_eq!(result, expected);
    }
}
