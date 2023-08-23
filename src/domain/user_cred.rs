use crate::domain::UserName;
use crate::domain::{Email, UserPass};
use secrecy::ExposeSecret;

use std::collections::HashMap;
use crate::validator::Validator;

#[derive(serde::Deserialize, Default)]
pub struct SignupJson {
    pub name: String,
    pub email: String,
    pub password: String,
}

impl TryFrom<SignupJson> for User {
    type Error = HashMap<&'static str, &'static str>;

    fn try_from(value: SignupJson) -> Result<Self, Self::Error> {
        //let mut error_map: Self::Error = HashMap::new();
        let mut v = Validator::new();

        let name = UserName::parse(value.name);
        let email = Email::parse(value.email);
        let pass = UserPass::parse(value.password);

        if let Err(name_err) = name {
            v.add_err("name", name_err);
        }
        if let Err(email_err) = email {
            error_map.insert("email", email_err);
        }
        if let Err(pass_err) = pass {
            error_map.insert("password", pass_err);
        }

        if !error_map.is_empty() {
            Err(error_map)
        } else {
            Ok(Self {
                name: name.unwrap(),
                email: email.unwrap(),
                password: pass.unwrap(),
            })
        }
    }
}

#[derive(serde::Deserialize, Default, Clone)]
pub struct LoginForm {
    pub email: String,
    pub password: String,
}

pub struct UserLoginForm {
    pub email: Email,
    pub password: UserPass,
}

impl TryFrom<LoginForm> for UserLoginForm {
    type Error = HashMap<&'static str, &'static str>;

    fn try_from(value: LoginForm) -> Result<Self, Self::Error> {
        let mut error_map: Self::Error = HashMap::new();

        let email = Email::parse(value.email);
        let pass = UserPass::parse(value.password);

        if let Err(email_err) = email {
            error_map.insert("email", email_err);
        }
        if let Err(pass_err) = pass {
            error_map.insert("password", pass_err);
        }
        if !error_map.is_empty() {
            Err(error_map)
        } else {
            Ok(Self {
                email: email.unwrap(),
                password: pass.unwrap(),
            })
        }
    }
}

#[derive(serde::Deserialize, Debug)]
pub struct PasswordUpdateForm {
    pub current_password: String,
    pub new_password: String,
    pub new_password_confirmation: String,
}

pub struct UserPasswordUpdate {
    pub current_password: UserPass,
    pub new_password: UserPass,
    pub new_password_confirmation: UserPass,
}

impl TryFrom<PasswordUpdateForm> for UserPasswordUpdate {
    type Error = HashMap<&'static str, &'static str>;

    fn try_from(value: PasswordUpdateForm) -> Result<Self, Self::Error> {
        let mut error_map: Self::Error = HashMap::new();
        let PasswordUpdateForm { current_password, new_password, new_password_confirmation } = value;

        let current = UserPass::parse(current_password);
        if let Err(pass_err) = current {
            error_map.insert("current_password", pass_err);
        }
        let new = UserPass::parse(new_password);
        if let Err(pass_err) = new {
            error_map.insert("new_password", pass_err);
        }
        let confirmation = UserPass::parse(new_password_confirmation);
        if let Err(pass_err) = confirmation {
            error_map.insert("new_password_confirmation", pass_err);
        }

        if matches!((&new, &confirmation), 
                    (Ok(pass), Ok(confirm_pass)) if pass.0.expose_secret() != confirm_pass.0.expose_secret()) {
                    error_map.insert("new_password_confirmation", "Passwords do not match");
        }
        
        if !error_map.is_empty() {
            Err(error_map)
        } else {
            Ok(Self {
                current_password: current.unwrap(),
                new_password: new.unwrap(),
                new_password_confirmation: confirmation.unwrap(),
            })
        }
    }
}
