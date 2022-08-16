pub mod currency;

pub use currency::*;
use schemars::JsonSchema;
use serde::{Serialize, Deserialize};


/// Languages used for the frontend
#[derive(
    Debug, Serialize, Deserialize, JsonSchema, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Copy
)]
pub enum Language {
    English,
    Russian
}

impl Language {
    pub fn to_alpha(&self) -> &str {
        match self {
            Language::English => "en",
            Language::Russian => "ru",
        }
    }
}