use mongodb::{bson::oid::ObjectId, Collection, Database};
use rocket::serde::{Deserialize, Serialize};

use crate::core::db::Model;

pub static DB_NAME: &str = "users";

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize)]
pub struct User {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub firstName: String,
    pub lastName: String,
}

impl Model for User {
    fn collection(db: &Database) -> Collection<User> {
        db.collection::<User>(&self::DB_NAME)
    }
}
