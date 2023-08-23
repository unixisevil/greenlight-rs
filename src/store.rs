mod movie;
mod user;
mod token;
mod permission;

use sqlx::postgres::{PgPool, PgPoolOptions};

use crate::config;
use crate::Error;

#[derive(Debug, Clone)]
pub struct Store {
    pub db: PgPool,
}

impl Store {
    pub fn new(dbc: &config::DbConfig) -> Result<Self, Error> {
        tracing::warn!("{}", dbc.db_dsn);
        let db_pool = PgPoolOptions::new()
            .max_connections(dbc.db_max_conn)
            .acquire_timeout(dbc.db_connect_timeout)
            .connect_lazy(&dbc.db_dsn)
            .map_err(Error::InitDatabase)?;

        Ok(Store { db: db_pool })
    }
}
