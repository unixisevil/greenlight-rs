use zxcvbn::zxcvbn;
use secrecy::Secret;
use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug, Clone, serde::Deserialize)]
pub struct UserPass(pub Secret<String>);

impl UserPass {
    pub fn parse(s: String) -> Result<Self, &'static str> {
        let is_empty_or_whitespace = s.trim().is_empty();
        if is_empty_or_whitespace{
            return Err("password cannot be blank")
        }
        let is_too_short = s.graphemes(true).count() < 8;
        let est = zxcvbn(&s, &[]).unwrap();
        let is_too_weak =  est.score() < 3;

        if is_too_short {
            Err("password must be at least 8 characters long")
        }else if is_too_weak  {
            Err("password is too weak")
        } else {
            Ok(Self(Secret::new(s)))
        }
    }
}


#[cfg(test)]
mod tests {
    use super::UserPass;
    use claims::{assert_err, assert_ok};

    #[test]
    fn a_pass_with_low_score_is_rejected() {
        let pass = "123456".to_string();
        assert_err!(UserPass::parse(pass));
    }

    #[test]
    fn whitespace_only_pass_are_rejected() {
        let pass = "  ".to_string();
        assert_err!(UserPass::parse(pass));
    }

    #[test]
    fn empty_pass_is_rejected() {
        let pass  = "".to_string();
        assert_err!(UserPass::parse(pass));
    }
    
    #[test]
    fn a_valid_pass_is_parsed_successfully() {
        let pass = "|correcthorsebattery$staple".to_string();
        assert_ok!(UserPass::parse(pass));
    }
}

