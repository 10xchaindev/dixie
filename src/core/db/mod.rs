pub  mod error;

use rocket_db_pools::Connection;
use serde::{de::DeserializeOwned, Serialize};

use mongodb::{
    bson::{doc, oid::ObjectId, Document},
    Collection,
};

use rocket::{async_trait, http::Status};
use rocket::{futures::TryStreamExt, State};

use super::config::Config;

pub trait Doc: Sized + Serialize + Sync + Unpin + Send + DeserializeOwned {}
impl<T> Doc for T where T: Sized + Serialize + Sync + Unpin + Send + DeserializeOwned {}

#[async_trait]
pub trait Model
where
    Self: Serialize,
{
    fn collection(db: &mongodb::Database) -> Collection<Self>
    where
        Self: Sized;

    /// todo replace to find_one
    async fn get_by_id(
        db: &mongodb::Database,
        id: &ObjectId,
        filter: Option<Document>,
    ) -> Result<Self, mongodb::error::Error>
    where
        Self: Doc,
    {
        let mut id_filter = doc! {"_id": id};

        let filter = if let Some(filter) = filter {
            id_filter.extend(filter);
            id_filter
        } else {
            id_filter
        };

        let result = Self::collection(db).find_one(filter, None).await;
        match result {
            Ok(result) => {
                if let Some(result) = result {
                    Ok(result)
                } else {
                    Err(mongodb::error::Error::custom(Status::NotFound))
                }
            }

            Err(error) => Err(error),
        }
    }
}

#[async_trait]
pub trait Rel<BaseModel: Model + Doc> {
    async fn resolve_rel(
        db: &mongodb::Database,
        document: &BaseModel,
    ) -> Result<Self, mongodb::error::Error>
    where
        Self: Sized;

    async fn find(
        db: &mongodb::Database,
        filter: Document,
    ) -> Result<Vec<Self>, mongodb::error::Error>
    where
        Self: Sized,
    {
        let mut collections = BaseModel::collection(&db).find(filter, None).await.unwrap();
        let mut results = vec![];

        while let Some(document) = collections.try_next().await.unwrap() {
            let rel = Self::resolve_rel(&db, &document).await.unwrap();
            results.push(rel);
        }

        Ok(results)
    }

    async fn find_one(
        db: &mongodb::Database,
        filter: Document,
    ) -> Result<Self, mongodb::error::Error>
    where
        Self: Sized,
    {
        let document = BaseModel::collection(&db)
            .find_one(filter, None)
            .await
            .unwrap();

        if let Some(document) = document {
            Ok(Self::resolve_rel(&db, &document).await.unwrap())
        } else {
            Err(mongodb::error::Error::custom(Status::NotFound))
        }
    }
}

#[derive(rocket_db_pools::Database)]
#[database("mongodb")]
pub struct Db(pub mongodb::Client);

impl Db {
    pub fn get(connection: &Connection<Self>, config: &State<Config>) -> mongodb::Database {
        connection.database(&config.db_name)
    }
}
