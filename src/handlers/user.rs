use tracing::instrument;
use warp::http::StatusCode;
use serde_json::json;
use redis::Client;

use crate::{errors::Error, user::ResetPass};
use crate::validator::Validator;
use crate::store::Store;
use crate::mailer::{push_task, Welcome};
use crate::user::{SignupJson, User};
use crate::token::{SCOPE_ACTIVATION, SCOPE_PASSWORDRESET, Token};
use super::password::gen_passwordhash;
use super::token::gen_token_and_save;


#[instrument]
pub async fn  register(
    input: SignupJson,
    store: Store,
    redis: Client,
) -> Result<impl warp::Reply, warp::Rejection> {
    let mut user: User  = input.try_into().map_err(Error::Validation)?;
    user.password_hash =  gen_passwordhash(user.password.clone().0).await?;

    let ret = store.add_user(&mut user).await;
    if let Err(Error::DuplicateEmail) = ret {
            let mut v = Validator::new(); 
            v.add_err("email", "a user with this email address already exists");
            return Err(Error::Validation(v.get_err()).into());
    }else if let Err(e)  =  ret  {
            return Err(e.into());
    }
    store.grant_permissions_to_user(user.id, &vec!["movies:read".to_owned()]).await?; 
    let tok = gen_token_and_save(store, user.id, chrono::Duration::days(3),  SCOPE_ACTIVATION).await?;

    let task =  Welcome::new(user.id, tok.plain_text)
                .gen_task(user.email.clone().into())
                .map_err(Error::Render)?;

    push_task(&redis, &task).await.map_err(Error::UnexpectedError)?; 

    Ok(warp::reply::with_status(
            warp::reply::json(&json!({"user": &user})), StatusCode::ACCEPTED,
        )
    )
}

pub async fn  activate(
    input: Token,
    store: Store,
) -> Result<impl warp::Reply, warp::Rejection> {
    let mut v = Validator::new(); 
    Token::validate(&mut v, &input.plain_text);
    if !v.valid() {
            return Err(Error::Validation(v.get_err()).into());
    }

    let  user = store.get_user_by_token(SCOPE_ACTIVATION, input.plain_text).await;
    if let Err(Error::RecordNotFound) = user {
            v.add_err("token", "invalid or expired activation token");
            return Err(Error::Validation(v.get_err()).into());
    }else if let Err(e)  =  user {
            return Err(e.into());
    }
    let mut user = user.unwrap();
    user.activated = true;
    store.update_user(&mut user).await?;
    store.delete_token(SCOPE_ACTIVATION, user.id).await?;

    Ok(warp::reply::with_status(
            warp::reply::json(&json!({"user": &user})), StatusCode::OK,
        )
    )
}


pub async fn  password_update(
    input: ResetPass,
    store: Store,
) -> Result<impl warp::Reply, warp::Rejection> {
    let mut v = Validator::new(); 
    input.validate(&mut v);
    if !v.valid() {
            return Err(Error::Validation(v.get_err()).into());
    }
    let user = store.get_user_by_token(SCOPE_PASSWORDRESET, &input.token).await;
    if let Err(Error::RecordNotFound) = user {
            v.add_err("token", "invalid or expired password reset token");
            return Err(Error::Validation(v.get_err()).into());
    }else if let Err(e)  =  user {
            return Err(e.into());
    }
    let mut user = user.unwrap();
    user.password_hash =  gen_passwordhash(user.password.clone().0).await?;
    store.update_user(&mut user).await?;
    store.delete_token(SCOPE_PASSWORDRESET, user.id).await?;

    Ok(warp::reply::with_status(
            warp::reply::json(&json!({"message": "your password was successfully reset"})), StatusCode::OK,
        )
    )
}


