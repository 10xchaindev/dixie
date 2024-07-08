use std::error::Error;

use mongodb::bson::Document;
use rocket::{async_trait, futures::TryStreamExt, http::Status};

use crate::core::{
    db::{Doc, Model},
    header_map::context::Context,
    pagination::limit_offset::{build_find_option, LimitOffsetPagination, Query},
};

#[async_trait]
pub trait Read<T: Model + Doc> {
    async fn get_all<'a>(
        context: &Context<'a>,
        query: &Query,
        filter: Option<Document>,
    ) -> Result<LimitOffsetPagination<T>, (Status, mongodb::error::Error)> {
        let options = build_find_option(&query);
        let count = T::collection(&context.db)
            .count_documents(None, None)
            .await
            .unwrap();
        let collections = T::collection(&context.db).find(filter, options).await;
        let mut results = vec![];

        match collections {
            Ok(mut collections) => {
                while let Some(document) = collections.try_next().await.unwrap() {
                    results.push(document);
                }

                Ok(LimitOffsetPagination::new(
                    query,
                    &context.uri,
                    count,
                    results,
                ))
            }

            Err(error) => Err((Status::InternalServerError, error)),
        }
    }

    async fn get_one<'a>(
        context: &Context<'a>,
        filter: Document,
    ) -> Result<T, (Status, Box<dyn Error>)> {
        match T::collection(&context.db).find_one(filter, None).await {
            Ok(document) => match document {
                Some(document) => Ok(document),
                None => Err((
                    Status::NotFound,
                    Box::new(mongodb::error::Error::custom(Status::NotFound)),
                )),
            },
            Err(error) => Err((Status::InternalServerError, Box::new(error))),
        }
    }
}
