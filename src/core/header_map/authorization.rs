use jsonwebtoken::{DecodingKey, TokenData, Validation};
use mongodb::{bson::oid::ObjectId, Database};
use rocket::{
    async_trait,
    http::Status,
    request::{FromRequest, Outcome},
    serde::json::Json,
    State,
};
use rocket_db_pools::Connection;
use serde::{Deserialize, Serialize};

use crate::{
    core::{
        config::Config,
        db::{Db, Model},
        error::{Error, ErrorKind},
    },
    models::user::User,
};

struct TokenAuthentication;

#[derive(Serialize, Deserialize)]
struct Claim {
    user_id: ObjectId,
}

impl TokenAuthentication {
    fn decode_token(
        shared_secret: &str,
        key: &str,
    ) -> Result<TokenData<Claim>, jsonwebtoken::errors::Error> {
        let decoded = jsonwebtoken::decode::<Claim>(
            &key,
            &DecodingKey::from_secret(shared_secret.as_ref()),
            &Validation::new(jsonwebtoken::Algorithm::HS256),
        );

        match decoded {
            Ok(token_data) => Ok(token_data),
            Err(error) => Err(error),
        }
    }

    pub async fn authenticate(
        shared_secret: &str,
        key: &str,
        db: &Database,
    ) -> Result<User, ErrorKind> {
        match TokenAuthentication::decode_token(shared_secret, &key) {
            Ok(token_data) => {
                let user = User::get_by_id(db, &token_data.claims.user_id, None).await;

                match user {
                    Ok(user) => Ok(user),
                    Err(error) => Err(ErrorKind::Database(error)),
                }
            }

            Err(error) => Err(ErrorKind::JWToken(error)),
        }
    }
}

pub struct Authorization(pub Box<User>);

#[derive(Debug)]
pub enum AuthorizationError {
    MissingKey,
    InvalidKey,
    NoUserFound,
}

#[async_trait]
impl<'r> FromRequest<'r> for Authorization {
    type Error = Json<Error>;

    async fn from_request(request: &'r rocket::Request<'_>) -> Outcome<Self, Self::Error> {
        let connection = request.guard::<Connection<Db>>().await.unwrap();
        let config = request.guard::<&State<Config>>().await.unwrap();

        let db = Db::get(&connection, config);

        match request.headers().get_one("Authorization") {
            Some(value) => {
                match TokenAuthentication::authenticate(&config.shared_secret, value, &db).await {
                    Ok(user) => Outcome::Success(Authorization(Box::new(user))),
                    Err(error) => Outcome::Error(error.to_response()),
                }
            }
            None => Outcome::Error((Status::BadRequest, Json(Error::custom()))),
        }
    }
}
