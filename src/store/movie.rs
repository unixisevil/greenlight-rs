use super::Store;

use sqlx::{
    postgres::PgRow,
    Row,
};

use crate::filter::{Filter, MetaData};
use crate::movie::Movie;
use crate::Error;

impl Store {
    pub async fn add_movie(&self, movie: &mut Movie) -> Result<(), Error> {
        match sqlx::query(
            r#"
                insert into movies (title, year, runtime, genres) 
                values ($1, $2, $3, $4)
                returning id, created_at, version
            "#,
        )
        .bind(&movie.title)
        .bind(movie.year)
        .bind(movie.runtime)
        .bind(&movie.genres)
        .map(|row: PgRow| {
            movie.id = row.get("id");
            movie.created_at = row.get("created_at");
            movie.version = row.get("version");
        })
        .fetch_one(&self.db)
        .await
        {
            Ok(_) => Ok(()),
            Err(e) => {
                tracing::error!("{:?}", e);
                Err(Error::DatabaseQuery(e))
            }
        }
    }

    pub async fn get_movie(&self, id: i64) -> Result<Movie, Error> {
        let row = sqlx::query_as!(
            Movie,
            r#"
                        select id, created_at, title, year, runtime, genres, version
                        from movies
                        where id = $1
                    "#,
            id,
        )
        .fetch_one(&self.db)
        .await
        .map_err(|e| {
            tracing::error!("{:?}", e);
            match e {
                sqlx::Error::RowNotFound => Error::RecordNotFound,
                _ => Error::DatabaseQuery(e),
            }
        })?;

        Ok(row)
    }

    pub async fn update_movie(&self, movie: &mut Movie) -> Result<(), Error> {
        match sqlx::query!(
            r#"
                update movies 
                set title = $1, year = $2, runtime = $3, genres = $4, version = version + 1
                where id = $5 AND version = $6
                returning version
            "#,
            &movie.title,
            &movie.year,
            &movie.runtime.as_ref(),
            &movie.genres,
            &movie.id,
            &movie.version,
        )
        .map(|ret| {
            movie.version = ret.version;
        })
        .fetch_one(&self.db)
        .await
        {
            Ok(_) => Ok(()),
            Err(e) => {
                tracing::error!("{:?}", e);
                match e {
                    sqlx::Error::RowNotFound => Err(Error::EditConflict),
                    _ => Err(Error::DatabaseQuery(e)),
                }
            }
        }
    }

    pub async fn delete_movie(&self, id: i64) -> Result<u64, Error> {
        let remove_count = sqlx::query!(
            r#"
               delete from  movies where  id = $1
            "#,
            id,
        )
        .execute(&self.db)
        .await
        .map_err(|e| {
            tracing::error!("{:?}", e);
            Error::DatabaseQuery(e)
        })?
        .rows_affected();

        Ok(remove_count)
    }

    pub async fn search_movie(
        &self,
        title: &str,
        genres: Vec<&str>,
        filter: &Filter<'_>,
    ) -> Result<(MetaData, Vec<Movie>), Error> {
        let mut count = 0i64;
        match sqlx::query(
            &format!(
            r#"
                select count(*) over(), id, created_at, title, year, runtime, genres, version
                from movies
                where (to_tsvector('simple', title) @@ plainto_tsquery('simple', $1) or $1 = '') 
                and (genres @> $2 or $2 = '{{}}')     
                order by {} {}, id asc
                limit $3 offset $4      
            "#,
            filter.sort_column().unwrap(), filter.sort_direction(),
            )
        )
        .bind(title)
        .bind(genres)
        .bind(filter.limit())
        .bind(filter.offset())
        .map(|row: PgRow| {
            count = row.get(0);
            Movie {
                id: row.get("id"),
                created_at: row.get("created_at"),
                title: row.get("title"),
                year: row.get("year"),
                runtime: row.get("runtime"),
                genres: row.get("genres"),
                version: row.get("version"),
            }
        })
        .fetch_all(&self.db)
        .await
        {
            Ok(movies) => Ok((MetaData::calc(count, filter.page, filter.page_size), movies)),
            Err(e) => {
                tracing::error!("{:?}", e);
                Err(Error::DatabaseQuery(e))
            }
        }
    }

}

