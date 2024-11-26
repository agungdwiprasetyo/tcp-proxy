use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Address {
    pub source: String,
    pub target: String,
}
