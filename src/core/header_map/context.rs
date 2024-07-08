use mongodb::Database;
use rocket::{
    async_trait,
    request::{FromRequest, Outcome},
    Request, State,
};
use rocket_db_pools::Connection;


use crate::core::{config::Config, db::Db};

use super::uri::Uri;

pub struct Context<'a> {
    pub uri: Uri<'a>,
    pub db: Database,
    pub config: &'a State<Config>,
}

#[async_trait]
impl<'r> FromRequest<'r> for Context<'r> {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let uri = request.guard::<Uri>().await.unwrap();
        let connection = request.guard::<Connection<Db>>().await.unwrap();
        let config = request.guard::<&State<Config>>().await.unwrap();

        let db = Db::get(&connection, &config);

        Outcome::Success(Self {
            uri,
            db,
            config,
        })
    }
}
