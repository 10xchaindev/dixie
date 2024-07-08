use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum DexieError {
    ParseError,
    NoContent,
}
