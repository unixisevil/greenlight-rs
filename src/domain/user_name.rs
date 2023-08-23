use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug, Default, serde::Serialize, sqlx::Type)]
#[sqlx(transparent)]
pub struct UserName(String);

impl UserName {
    pub fn parse(s: String) -> Result<UserName, &'static str> {
        let is_empty_or_whitespace = s.trim().is_empty();
        let is_too_long = s.graphemes(true).count() > 128;
        let forbidden_characters = ['/', '(', ')', '"', '<', '>', '\\', '{', '}'];
        let contains_forbidden_characters = s.chars().any(|g| forbidden_characters.contains(&g));

        if is_empty_or_whitespace {
            Err("name cannot be blank")
        }else if is_too_long{
            Err("name cannot be too long")
        }else if contains_forbidden_characters{
            Err("name contains forbidden character")
        } else {
            Ok(Self(s))
        }
    }
}

impl AsRef<str> for UserName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for UserName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
         self.0.fmt(f)
    }
}

#[cfg(test)]
mod tests {
    use super::UserName;
    use claims::{assert_err, assert_ok};

    #[test]
    fn a_256_grapheme_long_name_is_valid() {
        let name = "a̐".repeat(128);
        assert_ok!(UserName::parse(name));
    }

    #[test]
    fn a_name_longer_than_128_graphemes_is_rejected() {
        let name = "a".repeat(129);
        assert_err!(UserName::parse(name));
    }

    #[test]
    fn whitespace_only_names_are_rejected() {
        let name = " ".to_string();
        assert_err!(UserName::parse(name));
    }

    #[test]
    fn empty_string_is_rejected() {
        let name = "".to_string();
        assert_err!(UserName::parse(name));
    }

    #[test]
    fn names_containing_an_invalid_character_are_rejected() {
        for name in &['/', '(', ')', '"', '<', '>', '\\', '{', '}'] {
            let name = name.to_string();
            assert_err!(UserName::parse(name));
        }
    }

    #[test]
    fn a_valid_name_is_parsed_successfully() {
        let name = "Alan turing".to_string();
        assert_ok!(UserName::parse(name));
    }
}
