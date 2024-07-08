use mongodb::{bson::Uuid, Collection, Database};
use serde::{Deserialize, Serialize};

use crate::core::db::Model;

pub static DB_NAME: &str = "tokens";

#[derive(Serialize, Deserialize)]
pub struct Token {
    user_id: Uuid,
}

impl Model for Token {
    fn collection(db: &Database) -> Collection<Self> {
        db.collection::<Token>(&DB_NAME)
    }
}
