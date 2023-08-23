use tracing::instrument;
use warp::http::StatusCode;
use serde_json::json;
use std::collections::HashMap;

use crate::store::Store;
use crate::domain::movie::NewMovie;
use crate::validator::Validator;
use crate::errors::Error;
use crate::filter::Filter;


#[instrument]
pub async fn add_movie(
    store: Store,
    input: NewMovie,
) -> Result<impl warp::Reply, warp::Rejection> {
    let mut movie = input.try_into().map_err(Error::Validation)?;

    store.add_movie(&mut movie).await?;  

    let loc = format!("/v1/movies/{}", movie.id);
    let body = json!({"movie": &movie});
    Ok(
        warp::reply::with_status(
            warp::reply::with_header(warp::reply::json(&body), "Location", loc), 
            StatusCode::CREATED,
        )
    )
}

#[instrument]
pub async fn get_movie(
    id: i64,
    store: Store,
) -> Result<impl warp::Reply, warp::Rejection> {
    let movie = store.get_movie(id).await?;
    Ok( 
        warp::reply::with_status(warp::reply::json(&movie), StatusCode::OK)
    )
}

#[instrument]
pub async fn update_movie(
    id: i64,
    input: NewMovie,
    store: Store,
) -> Result<impl warp::Reply, warp::Rejection> {
    let mut movie = store.get_movie(id).await?;

    input.validate().map_err(Error::Validation)?;

    if let Some(title) = input.title {
        movie.title = title;
    }
    if let Some(year) = input.year {
        movie.year = year;
    }
    if let Some(runtime) = input.runtime {
        movie.runtime = runtime;
    }
    if let Some(genres) = input.genres {
        movie.genres = genres;
    }

    store.update_movie(&mut movie).await?;  
    Ok( 
        warp::reply::with_status(warp::reply::json(&movie), StatusCode::OK)
    )
}

#[instrument]
pub async fn remove_movie(
    id: i64,
    store: Store,
) -> Result<impl warp::Reply, warp::Rejection> {
    let count = store.delete_movie(id).await?;
    if count == 0  {
        return Err(Error::RecordNotFound.into());
    }
    let msg = json!({"message": "movie successfully deleted"});
    Ok( 
        warp::reply::with_status(warp::reply::json(&msg), StatusCode::OK)
    )
}

#[instrument]
pub async fn search_movie(
    qs: HashMap<String, String>,
    store: Store,
)-> Result<impl warp::Reply, warp::Rejection> {
    let title =  qs.get("title").map(String::as_str).unwrap_or_default();
    let genres = qs.get("genres").map_or(vec![], |gs| gs.split(',').collect());

    let mut v = Validator::new(); 
    let page = match qs.get("page") {
        None =>  1i64,
        Some(e) =>  {
            e.parse().map_err(|_|v.add_err("page",  "must be an integer value")).unwrap_or(1i64)
        },
    };
    let page_size = match qs.get("page_size") {
        None =>  20i64,
        Some(e) =>  {
            e.parse().map_err(|_| v.add_err("page_size",  "must be an integer value")).unwrap_or(20i64)
        },
    };

    let sort =  qs.get("sort").map(String::as_str).unwrap_or("id");

    let filter = Filter{
        page, 
        page_size,
        sort,
        sort_list: &["id", "title", "year", "runtime", "-id", "-title", "-year", "-runtime"],
    };

    filter.validate(&mut v);
    if !v.valid() {
        return Err(Error::Validation(v.get_err()).into());
    }

    let (meta, movies) = store.search_movie(title, genres, &filter).await?;

    Ok( 
        warp::reply::with_status(
            warp::reply::json(&json!({"metadata": meta, "movies": movies})), 
            StatusCode::OK
        )
    )
}


