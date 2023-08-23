use chrono::{DateTime, Utc};
use secrecy::{ExposeSecret, Secret};
use std::collections::HashMap;

use super::email::Email;
use super::user_name::UserName;
use super::user_pass::UserPass;
use super::token::Token;
use crate::validator::Validator;

#[derive(serde::Serialize)]
pub struct User {
    pub id: i64,
    pub created_at: DateTime<Utc>,
    pub name: UserName,
    pub email: Email,
    #[serde(skip)]
    pub password: UserPass,
    #[serde(skip)]
    pub password_hash: Secret<String>,
    pub activated: bool,
    #[serde(skip)]
    pub version: i32,
}

#[derive(serde::Deserialize, Debug)]
pub struct SignupJson {
    pub name: String,
    pub email: String,
    pub password: String,
}

impl TryFrom<SignupJson> for User {
    type Error = HashMap<&'static str, &'static str>;

    fn try_from(value: SignupJson) -> Result<Self, Self::Error> {
        let mut v = Validator::new();

        let name = UserName::parse(value.name);
        let email = Email::parse(value.email);
        let pass = UserPass::parse(value.password);

        if let Err(name_err) = name {
            v.add_err("name", name_err);
        }
        if let Err(email_err) = email {
            v.add_err("email", email_err);
        }
        if let Err(pass_err) = pass {
            v.add_err("password", pass_err);
        }

        if !v.valid() {
            Err(v.get_err())
        } else {
            Ok(Self {
                name: name.unwrap(),
                email: email.unwrap(),
                password: pass.unwrap(),
                id: 0,
                created_at: DateTime::<Utc>::default(),
                activated: false,
                version: 0,
                password_hash: Secret::new(String::default()),
            })
        }
    }
}

#[derive(serde::Deserialize)]
pub struct LoginJson {
    pub email: String,
    pub password: String,
}

pub struct LoginUser {
    pub email: Email,
    pub password: UserPass,
}

impl TryFrom<LoginJson> for LoginUser {
    type Error = HashMap<&'static str, &'static str>;

    fn try_from(value: LoginJson) -> Result<Self, Self::Error> {
        let mut v = Validator::new();

        let email = Email::parse(value.email);
        let pass = UserPass::parse(value.password);

        if let Err(email_err) = email {
            v.add_err("email", email_err);
        }
        if let Err(pass_err) = pass {
            v.add_err("password", pass_err);
        }
        if !v.valid() {
            Err(v.get_err())
        } else {
            Ok(Self {
                email: email.unwrap(),
                password: pass.unwrap(),
            })
        }
    }
}

#[derive(serde::Deserialize)]
pub struct EmailJson {
    pub email: String,
}

impl TryFrom<EmailJson> for Email {
    type Error = HashMap<&'static str, &'static str>;

    fn try_from(value: EmailJson) -> Result<Self, Self::Error> {
        let mut v = Validator::new();
        let ret = Email::parse(value.email);
        if let Err(email_err) = ret {
            v.add_err("email", email_err);
        }
        if !v.valid() {
            Err(v.get_err())
        } else {
            Ok(ret.unwrap())
        }
    }
}

#[derive(serde::Deserialize)]
pub struct ResetPass {
    pub password: UserPass,
    pub token: String,
}

impl ResetPass {
    pub fn validate(&self, v: &mut Validator) {
        let pass = UserPass::parse(self.password.0.expose_secret().clone());
        if let Err(pass_err) = pass {
            v.add_err("password", pass_err);
        }
        Token::validate(v,  &self.token);
    }
}
