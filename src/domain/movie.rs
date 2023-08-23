/*
type Movie struct {
    ID        int64     `json:"id"`
    CreatedAt time.Time `json:"-"`
    Title     string    `json:"title"`
    Year      int32     `json:"year,omitempty"`
    Runtime   Runtime   `json:"runtime,omitempty"`
    Genres    []string  `json:"genres,omitempty"`
    Version   int32     `json:"version"`
}
*/

use super::runtime::RunTime;
use crate::validator::Validator;
use chrono::prelude::*;
use chrono::{DateTime, Utc};
use std::collections::HashMap;

#[derive(Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct Movie {
    #[serde(skip_deserializing)]
    pub id: i64,

    #[serde(skip)]
    pub created_at: DateTime<Utc>,

    pub title: String,

    #[serde(skip_serializing_if = "is_zero")]
    pub year: i32,

    #[serde(skip_serializing_if = "RunTime::is_zero")]
    pub runtime: RunTime,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub genres: Vec<String>,

    #[serde(skip_deserializing)]
    pub version: i32,
}

fn is_zero(num: &i32) -> bool {
    *num == 0
}

#[derive(serde::Deserialize, Debug)]
pub struct NewMovie {
    #[serde(default)]
    pub title: Option<String>,

    #[serde(default)]
    pub year: Option<i32>,

    #[serde(default)]
    pub runtime: Option<RunTime>,

    #[serde(default)]
    pub genres: Option<Vec<String>>,
}

impl NewMovie {
    pub fn validate(&self) -> Result<(), HashMap<&'static str, &'static str>> {
        let mut v = Validator::new();

        if let Some(ref title) = self.title {
            v.check(!title.is_empty(), "title", "must be provided");
            v.check(
                title.len() <= 500,
                "title",
                "must not be more than 500 bytes long",
            );
        }

        if let Some(year) = self.year {
            v.check(year >= 1888, "year", "must be greater than 1888");
            v.check(
                year <= Utc::now().year(),
                "year",
                "must not be in the future",
            );
        }

        if let Some(runtime) = self.runtime {
            v.check(
                runtime.as_ref() > &0,
                "runtime",
                "must be a positive integer",
            );
        }

        if let Some(ref genres) = self.genres {
            v.check(
                !genres.is_empty(),
                "genres",
                "must contain at least 1 genre",
            );
            v.check(
                genres.len() <= 5,
                "genres",
                "must not contain more than 5 genres",
            );
            v.check(
                !(1..genres.len()).any(|i| genres[i..].contains(&genres[i - 1])),
                "genres",
                "must not contain duplicate values",
            );
        }

        if !v.valid() {
            Err(v.get_err())
        } else {
            Ok(())
        }
    }
}

impl TryFrom<NewMovie> for Movie {
    type Error = HashMap<&'static str, &'static str>;

    fn try_from(value: NewMovie) -> Result<Self, Self::Error> {
        let mut v = if let Err(err_map) = value.validate() {
            err_map.into()
        } else {
            Validator::new()
        };

        v.check(value.title.is_some(),   "title",     "must be provided");
        v.check(value.year.is_some(),    "year",       "must be provided");
        v.check(value.runtime.is_some(), "runtime", "must be provided");
        v.check(value.genres.is_some(),  "genres",   "must be provided");
        
        if !v.valid() {
            Err(v.get_err())
        } else {
            Ok(Self {
                title: value.title.unwrap(),
                year: value.year.unwrap(),
                runtime: value.runtime.unwrap(),
                genres: value.genres.unwrap(),
                ..Self::default()
            })
        }
    }
}

/*
        v.check(value.title.as_ref().is_some_and(|x| x.len() > 0),"title", "must be provided");
        v.check(value.title.as_ref().is_some_and(|x| x.len() <= 500), "title", "must not be more than 500 bytes long");

        v.check(value.year.is_some(), "year", "must be provided");
        v.check(value.year.as_ref().is_some_and(|&x| x >= 1888), "year", "must be greater than 1888");
        v.check(value.year.as_ref().is_some_and(|&x| x <= Utc::now().year()) , "year", "must not be in the future");

        v.check(value.runtime.is_some(), "runtime",  "must be provided");
        v.check(value.runtime.as_ref().is_some_and(|x| x.as_ref() > &0),  "runtime", "must be a positive integer");

        v.check(value.genres.is_some(), "genres",   "must be provided");
        v.check(value.genres.as_ref().is_some_and(|x| x.len() >= 1), "genres",  "must contain at least 1 genre");
        v.check(value.genres.as_ref().is_some_and(|x| x.len() <= 5), "genres",  "must not contain more than 5 genres");
        v.check(
            !value.genres.as_ref().is_some_and(|vs| (1..vs.len()).any(|i| vs[i..].contains(&vs[i - 1]))),
            "genres",  "must not contain duplicate values"
        );
        */

