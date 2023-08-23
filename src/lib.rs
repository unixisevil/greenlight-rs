mod errors;
mod config;
mod store;
mod domain;
mod validator;
mod route;
mod handlers;
mod mailer;

pub use errors::Error;
pub use config::Config;
pub use mailer::run_mail_worker;
use domain::*;
use route::build_routes;
use store::Store;
use mailer::Mailer;


use anyhow::Context;
use warp::{Filter, Reply};

pub fn build(config: config::Config) -> Result<(Mailer, warp::Server<impl Filter<Extract = impl Reply> + Clone>) , Error> {
    let store = Store::new(&config.pg)?;
   
    let redis = redis::Client::open(config.redis_url)
        .context("failed to parse redis_url")
        .map_err(Error::UnexpectedError)?;

    let mailer =  Mailer::new(config.mail, redis.clone());
    let routes =  build_routes(store, redis);     //.await;
    Ok((mailer, warp::serve(routes)))            
}

