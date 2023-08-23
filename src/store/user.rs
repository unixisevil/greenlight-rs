use super::Store;

use sqlx::{postgres::PgRow, Row};
use chrono::Utc;
use secrecy::{ExposeSecret, Secret};

use crate::token::Token;
use crate::user::User;
use crate::user_pass::UserPass;
use crate::Error;


impl Store {
    pub async fn add_user(&self, user: &mut User) -> Result<(), Error> {
        match sqlx::query!(
            r#"
               insert into users (name, email, password_hash, activated) 
               values ($1, $2::TEXT::CITEXT, $3, $4)
               returning id, created_at, version
            "#,
            user.name.as_ref(),
            user.email.as_ref(),
            user.password_hash.expose_secret(),
            user.activated,
        )
        .map(|ret| {
            user.version = ret.version;
            user.id = ret.id;
            user.created_at = ret.created_at;
        })
        .fetch_one(&self.db)
        .await
        {
            Ok(_) => Ok(()),
            Err(e) => {
                tracing::error!("{:?}", e);
                if let Some(de) = e.as_database_error() {
                    if de.is_unique_violation() {
                        Err(Error::DuplicateEmail)
                    } else {
                        Err(Error::DatabaseQuery(e))
                    }
                } else {
                    Err(Error::DatabaseQuery(e))
                }
            }
        }
    }

    pub async fn get_user_by_token(
        &self,
        scope: impl AsRef<str>,
        token: impl AsRef<str>,
    ) -> Result<User, Error> {
        let user = sqlx::query(
            r#"
            select users.id, users.created_at, users.name, users.email::text, 
                   users.password_hash, users.activated, users.version
            from users
            inner join tokens
            on users.id = tokens.user_id
            where tokens.hash = $1
            and tokens.scope = $2 
            and tokens.expiry > $3
            "#,
        )
        .bind(Token::gen_hash(token))
        .bind(scope.as_ref())
        .bind(Utc::now())
        .map(|row: PgRow| User {
            id: row.get("id"),
            created_at: row.get("created_at"),
            name: row.get("name"),
            email: row.get("email"),
            password: UserPass(Secret::new(String::default())),
            password_hash: Secret::new(row.get("password_hash")),
            activated: row.get("activated"),
            version: row.get("version"),
        })
        .fetch_one(&self.db)
        .await
        .map_err(|e| {
            tracing::error!("{:?}", e);
            match e {
                sqlx::Error::RowNotFound => Error::RecordNotFound,
                _ => Error::DatabaseQuery(e),
            }
        })?;

        Ok(user)
    }


    pub async fn get_user_by_email(&self, email: impl AsRef<str>) -> Result<User, Error>  {
        let user = sqlx::query(
            r#"
                 select id, created_at, name, email::TEXT, password_hash, activated, version
                 from users
                 where email = $1::TEXT::CITEXT          
            "#
        )
        .bind(email.as_ref())
        .map(|row: PgRow| User {
            id: row.get("id"),
            created_at: row.get("created_at"),
            name: row.get("name"),
            email: row.get("email"),
            password: UserPass(Secret::new(String::default())),
            password_hash: Secret::new(row.get("password_hash")),
            activated: row.get("activated"),
            version: row.get("version"),
        })
        .fetch_one(&self.db)
        .await
        .map_err(|e| {
            tracing::error!("{:?}", e);
            match e {
                sqlx::Error::RowNotFound => Error::RecordNotFound,
                _ => Error::DatabaseQuery(e),
            }
        })?;

        Ok(user)
    }

    pub async fn update_user(&self, user: &mut User) -> Result<(), Error> {
        match sqlx::query!(
            r#"
               update users 
               set name = $1, email = $2::TEXT::CITEXT, password_hash = $3, activated = $4, version = version + 1
               where id = $5 and version = $6
               returning version              
            "#,
            user.name.as_ref(),
            user.email.as_ref(),
            AsRef::<str>::as_ref(user.password_hash.expose_secret()),
            user.activated,
            user.id,
            user.version,
        )
        .map(|ret| {
            user.version = ret.version;
        })
        .fetch_one(&self.db)
        .await
        {
            Ok(_) => Ok(()),
            Err(e) => {
                tracing::error!("{:?}", e);
                match e {
                    sqlx::Error::RowNotFound => Err(Error::EditConflict),
                    sqlx::Error::Database(ref de) => {
                        if de.is_unique_violation() {
                            Err(Error::DuplicateEmail)
                        }else {
                            Err(Error::DatabaseQuery(e))
                        }
                    }
                    _ => Err(Error::DatabaseQuery(e)),
                }
            }
        }

    }
}
