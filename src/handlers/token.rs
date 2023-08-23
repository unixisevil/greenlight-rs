use chrono::Duration;
use redis::Client;
use serde_json::json;
use warp::http::StatusCode;

use crate::domain::token::Token;
use crate::domain::user::{EmailJson, LoginJson, LoginUser};
use crate::errors::Error;
use crate::store::Store;
use crate::token::{SCOPE_ACTIVATION, SCOPE_AUTHENTICATION, SCOPE_PASSWORDRESET};
use crate::validator::Validator;
use crate::Email;
use crate::mailer::{push_task, PasswordReset, TokenActivation};

use super::password::verify_passwordhash;

pub(super) async fn gen_token_and_save(
    store: Store,
    user_id: i64,
    ttl: Duration,
    scope: &'static str,
) -> Result<Token, Error> {
    let tok = Token::new(user_id, ttl, scope);
    store.save_token(&tok).await?;
    Ok(tok)
}

pub async fn gen_auth_token(
    input: LoginJson,
    store: Store,
) -> Result<impl warp::Reply, warp::Rejection> {
    let login_user: LoginUser = input.try_into().map_err(Error::Validation)?;
    let user = store.get_user_by_email(&login_user.email).await;
    if let Err(Error::RecordNotFound) = user {
        return Err(Error::InvalidCredentials.into());
    } else if let Err(e) = user {
        return Err(e.into());
    }
    let user = user.unwrap();
    let user_id = user.id;

    verify_passwordhash(user.password_hash, login_user.password.0).await?;
    let tok = gen_token_and_save(
        store,
        user_id,
        chrono::Duration::hours(24),
        SCOPE_AUTHENTICATION,
    )
    .await?;

    Ok(warp::reply::with_status(
        warp::reply::json(&json!({"authentication_token": tok})),
        StatusCode::CREATED,
    ))
}

pub async fn gen_reset_token(
    input: EmailJson,
    store: Store,
    redis: Client,
) -> Result<impl warp::Reply, warp::Rejection> {
    let email: Email = input.try_into().map_err(Error::Validation)?;
    let user = store.get_user_by_email(&email).await;

    let mut v = Validator::new();
    if let Err(Error::RecordNotFound) = user {
        v.add_err("email", "no matching email address found");
        return Err(Error::Validation(v.get_err()).into());
    } else if let Err(e) = user {
        return Err(e.into());
    }
    let user = user.unwrap();
    if !user.activated {
        v.add_err("email", "user account must be activated");
        return Err(Error::Validation(v.get_err()).into());
    }

    let tok = gen_token_and_save(
        store,
        user.id,
        chrono::Duration::minutes(45),
        SCOPE_PASSWORDRESET,
    )
    .await?;

    let task =  PasswordReset::new(tok.plain_text)
                .gen_task(user.email.into())
                .map_err(Error::Render)?;

    push_task(&redis, &task).await.map_err(Error::UnexpectedError)?; 

    let msg = json!({"message": "an email will be sent to you containing password reset instructions"});
    Ok(warp::reply::with_status(
            warp::reply::json(&msg), StatusCode::ACCEPTED,
        )
    )
}


pub async fn gen_activation_token(
    input: EmailJson,
    store: Store,
    redis: Client,
) -> Result<impl warp::Reply, warp::Rejection> {
    let email: Email = input.try_into().map_err(Error::Validation)?;
    let user = store.get_user_by_email(&email).await;

    let mut v = Validator::new();
    if let Err(Error::RecordNotFound) = user {
        v.add_err("email", "no matching email address found");
        return Err(Error::Validation(v.get_err()).into());
    } else if let Err(e) = user {
        return Err(e.into());
    }
    let user = user.unwrap();
    if user.activated {
        v.add_err("email", "user has already been activated");
        return Err(Error::Validation(v.get_err()).into());
    }

    let tok = gen_token_and_save(
        store,
        user.id,
        chrono::Duration::days(3),
        SCOPE_ACTIVATION,
    )
    .await?;

    let task =  TokenActivation::new(tok.plain_text)
                .gen_task(user.email.into())
                .map_err(Error::Render)?;

    push_task(&redis, &task).await.map_err(Error::UnexpectedError)?; 

    let msg = json!({"message": "an email will be sent to you containing activation instructions"});
    Ok(warp::reply::with_status(
            warp::reply::json(&msg), StatusCode::ACCEPTED,
        )
    )
}
