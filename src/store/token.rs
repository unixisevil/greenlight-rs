use super::Store;

use crate::token::Token;
use crate::Error;

impl Store {
    pub async fn save_token(&self, token: &Token) -> Result<(), Error> {
        sqlx::query!(
            r#"
              insert into tokens (hash, user_id, expiry, scope) 
              values ($1, $2, $3, $4)   
             "#,
            token.hash,
            token.user_id,
            token.expiry,
            token.scope,
        )
        .execute(&self.db)
        .await
        .map_err(|e| {
            tracing::error!("{:?}", e);
            Error::DatabaseQuery(e)
        })?;

        Ok(())
    }

    pub async fn delete_token(&self, scope: impl AsRef<str>, user_id: i64) -> Result<(), Error> {
        sqlx::query!(
            r#"
               delete from tokens where scope = $1 and user_id = $2
            "#,
            scope.as_ref(),
            user_id,
        )
        .execute(&self.db)
        .await
        .map_err(|e| {
            tracing::error!("{:?}", e);
            Error::DatabaseQuery(e)
        })?;

        Ok(())
    }
}
