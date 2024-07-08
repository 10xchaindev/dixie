use mongodb::options::FindOptions;
use rocket::{FromForm, State};
use serde::{Deserialize, Serialize};

use crate::core::{config::Config, header_map::uri::Uri};

#[derive(FromForm)]
pub struct Query {
    limit: i64,
    offset: u64,
}

impl Query {
    pub fn new(limit: i64, offset: u64) -> Self {
        Self { limit, offset }
    }

    pub fn default(config: &State<Config>, query: Option<Self>) -> Self {
        if let Some(query) = query {
            query
        } else {
            Self {
                limit: config.page_size,
                offset: 0,
            }
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct LimitOffsetPagination<T> {
    limit: i64,
    offset: u64,
    count: u64,
    next: Option<String>,
    previous: Option<String>,
    results: Vec<T>,
}

impl<T> LimitOffsetPagination<T> {
    pub fn get_next_uri(&self, uri: &Uri) -> Option<String> {
        if self.offset < self.count {
            let offset = self.offset + self.limit as u64;
            Some(format!(
                "{}?offset={}&limit={}",
                uri.to_string(),
                offset,
                self.limit
            ))
        } else {
            None
        }
    }

    pub fn get_previous_uri(&self, uri: &Uri) -> Option<String> {
        if self.offset > self.limit as u64 {
            Some(format!(
                "{}?offset={}&limit={}",
                uri.to_string(),
                self.offset,
                self.limit
            ))
        } else {
            None
        }
    }

    pub fn new(Query { limit, offset }: &Query, uri: &Uri, count: u64, results: Vec<T>) -> Self {
        let mut result = Self {
            limit: limit.clone(),
            offset: offset.clone(),
            count,
            next: None,
            previous: None,
            results,
        };

        result.next = result.get_next_uri(&uri);
        result.previous = result.get_previous_uri(&uri);

        result
    }
}

pub fn build_find_option(query: &Query) -> FindOptions {
    FindOptions::builder()
        .limit(query.limit)
        .skip(query.offset)
        .build()
}
