use serde::Deserialize;

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct Config {
    pub address: String,
    pub page_size: i64,
    pub db_name: String,
    pub shared_secret: String
}
