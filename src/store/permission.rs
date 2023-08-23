use super::Store;

use crate::Error;

impl Store {

    pub async fn permissions_by_user(&self, user_id: i64) -> Result<Vec<String>, Error> {
          let row = sqlx::query!(
            r#"
                select permissions.code
                from permissions
                inner join users_permissions on users_permissions.permission_id = permissions.id
                inner join users on users_permissions.user_id = users.id
                where users.id = $1
            "#,
              user_id,
          )
          .map(|ret|{ 
              ret.code
          })
          .fetch_all(&self.db)
          .await
          .map_err(|e| {
               tracing::error!("{:?}, user_id = {}", e, user_id);
               Error::DatabaseQuery(e)
          })?;

          Ok(row)
    }

    pub async fn grant_permissions_to_user(&self, user_id: i64, codes: &Vec<String>) -> Result<(), Error> {
         let _count =  sqlx::query!(
             r#"
              insert into users_permissions
              select $1, permissions.id from permissions where permissions.code = ANY($2)
             "#,
             user_id, codes,
         )
        .execute(&self.db)
        .await
        .map_err(|e| {
            tracing::error!("{:?}", e);
            Error::DatabaseQuery(e)
        })?
        .rows_affected();

        println!("grant count = {}", _count);

        Ok(())
    }

}
