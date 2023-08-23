use redis::Client;
use std::collections::HashMap;
use tracing::instrument;
use warp::{http::Method, Filter, Reply};

use crate::errors::{return_error, Error};
use crate::handlers::movie;
use crate::handlers::token;
use crate::handlers::user;
use crate::store::Store;
use crate::token::{Token, SCOPE_AUTHENTICATION};
use crate::validator::Validator;

fn with_perm(
    perm_code: &'static str,
) -> impl Filter<Extract = (&'static str,), Error = std::convert::Infallible> + Copy {
    warp::any().map(move || perm_code)
}

#[instrument]
async fn require_permission(
    perm_code: &'static str,
    tok_str: Option<String>,
    store: Store,
) -> Result<(), warp::Rejection> {
    let Some(tok_str) = tok_str  else {
        return Err(Error::AuthenticationRequired.into());
    };

    let vs = tok_str.split(' ').collect::<Vec<&str>>();
    if vs.len() != 2 || vs[0] != "Bearer" {
        return Err(Error::InvalidAuthenticationToken.into());
    }

    let mut v = Validator::new();
    Token::validate(&mut v, vs[1]);
    if !v.valid() {
        return Err(Error::InvalidAuthenticationToken.into());
    }

    let user = store
        .get_user_by_token(SCOPE_AUTHENTICATION, vs[1])
        .await
        .map_err(|e| match e {
            Error::RecordNotFound => Error::InvalidAuthenticationToken,
            _ => e,
        })?;

    if !user.activated {
        return Err(Error::InactiveAccount.into());
    }

    let perms = store.permissions_by_user(user.id).await?;
    tracing::debug!(request_perm= ?perm_code, have_perms= ?perms, "before search perm list");

    if !perms.iter().any(|e| e == perm_code) {
        tracing::warn!("request will be rejected");
        return Err(Error::Unauthorized.into());
    }

    tracing::debug!("permission check pass");

    Ok(())
}

pub fn build_routes(store: Store, redis: Client) -> impl Filter<Extract = impl Reply> + Clone {
    let store_filter = warp::any().map(move || store.clone());
    let redis_filter = warp::any().map(move || redis.clone());

    let cors = warp::cors()
        .allow_any_origin()
        .allow_header("content-type")
        .allow_methods(&[Method::PATCH, Method::DELETE, Method::GET, Method::POST]);

    let write_perm = with_perm("movies:write")
        .and(warp::header::optional::<String>("Authorization"))
        .and(store_filter.clone())
        .and_then(require_permission)
        .untuple_one();

    let read_perm = with_perm("movies:read")
        .and(warp::header::optional::<String>("Authorization"))
        .and(store_filter.clone())
        .and_then(require_permission)
        .untuple_one();

    let prefix = warp::path!("v1" / ..);

    let get_movie = warp::get()
        .and(warp::path("movies"))
        .and(warp::path::param::<i64>())
        .and(warp::path::end())
        .and(store_filter.clone())
        .and(read_perm.clone())
        .and_then(movie::get_movie);

    let add_movie = warp::post()
        .and(warp::path("movies"))
        .and(warp::path::end())
        .and(store_filter.clone())
        .and(warp::body::json())
        .and(write_perm.clone())
        .and_then(movie::add_movie);

    let update_movie = warp::patch()
        .and(warp::path("movies"))
        .and(warp::path::param::<i64>())
        .and(warp::path::end())
        .and(warp::body::json())
        .and(store_filter.clone())
        .and(write_perm.clone())
        .and_then(movie::update_movie);

    let remove_movie = warp::delete()
        .and(warp::path("movies"))
        .and(warp::path::param::<i64>())
        .and(warp::path::end())
        .and(store_filter.clone())
        .and(write_perm)
        .and_then(movie::remove_movie);

    let search_movie = warp::get()
        .and(warp::path("movies"))
        .and(warp::path::end())
        .and(warp::query::<HashMap<String, String>>())
        .and(store_filter.clone())
        .and(read_perm)
        .and_then(movie::search_movie);

    let reg_user = warp::post()
        .and(warp::path("users"))
        .and(warp::path::end())
        .and(warp::body::json())
        .and(store_filter.clone())
        .and(redis_filter.clone())
        .and_then(user::register);

    let activate = warp::put()
        .and(warp::path!("users" / "activated"))
        .and(warp::path::end())
        .and(warp::body::json())
        .and(store_filter.clone())
        .and_then(user::activate);

    let password_update = warp::put()
        .and(warp::path!("users" / "password"))
        .and(warp::path::end())
        .and(warp::body::json())
        .and(store_filter.clone())
        .and_then(user::password_update);

    let auth_token = warp::post()
        .and(warp::path!("tokens" / "authentication"))
        .and(warp::path::end())
        .and(warp::body::json())
        .and(store_filter.clone())
        .and_then(token::gen_auth_token);

    let activate_token = warp::post()
        .and(warp::path!("tokens" / "activation"))
        .and(warp::path::end())
        .and(warp::body::json())
        .and(store_filter.clone())
        .and(redis_filter.clone())
        .and_then(token::gen_activation_token);

    let reset_token = warp::post()
        .and(warp::path!("tokens" / "password-reset"))
        .and(warp::path::end())
        .and(warp::body::json())
        .and(store_filter)
        .and(redis_filter)
        .and_then(token::gen_reset_token);

    prefix
        .and(
            get_movie
            .or(add_movie)
            .or(update_movie)
            .or(remove_movie)
            .or(search_movie)
            .or(reg_user)
            .or(activate)
            .or(password_update)
            .or(auth_token)
            .or(activate_token)
            .or(reset_token),
    )
    .with(cors)
    .with(warp::trace::request())
    .recover(return_error)
}
