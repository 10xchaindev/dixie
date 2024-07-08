use std::borrow::Borrow;

use mongodb::{
    bson::Document,
    error,
    options::UpdateModifications,
    results::{InsertOneResult, UpdateResult},
};
use rocket::{async_trait, http::Status};

use crate::core::{
    db::{error::DexieError, Doc, Model},
    header_map::context::Context,
};

#[async_trait]
pub trait Write<T: Model + Doc> {
    async fn create_one<'a>(
        context: &Context<'a>,
        doc: impl Borrow<T> + Send,
    ) -> Result<T, (Status, error::Error)> {
        match T::collection(&context.db).insert_one(doc, None).await {
            Ok(InsertOneResult { inserted_id, .. }) => {
                let id = inserted_id.as_object_id();
                if let Some(id) = &id {
                    match T::get_by_id(&context.db, id, None).await {
                        Ok(document) => Ok(document),
                        Err(error) => Err((Status::NotFound, error)),
                    }
                } else {
                    Err((
                        Status::NotModified,
                        mongodb::error::Error::custom(DexieError::NoContent),
                    ))
                }
            }
            Err(error) => Err((Status::InternalServerError, error)),
        }
    }

    async fn update_one<'a>(
        context: &Context<'a>,
        query: Document,
        update: impl Into<UpdateModifications> + Send,
    ) -> Result<T, (Status, error::Error)> {
        match T::collection(&context.db)
            .update_one(query, update, None)
            .await
        {
            Ok(UpdateResult { upserted_id, .. }) => {
                if let Some(id) = upserted_id {
                    if let Some(id) = id.as_object_id() {
                        match T::get_by_id(&context.db, &id, None).await {
                            Ok(result) => Ok(result),
                            Err(error) => Err((Status::NotFound, error)),
                        }
                    } else {
                        Err((
                            Status::NotModified,
                            mongodb::error::Error::custom(DexieError::NoContent),
                        ))
                    }
                } else {
                    Err((
                        Status::NotModified,
                        mongodb::error::Error::custom(DexieError::NoContent),
                    ))
                }
            }
            Err(error) => Err((Status::InternalServerError, error)),
        }
    }
}
