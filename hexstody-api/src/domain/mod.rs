pub mod currency;

use std::str::FromStr;
pub use currency::*;
use schemars::JsonSchema;
use serde::{Serialize, Deserialize};
use regex::Regex;
use thiserror::Error;

/// Languages used for the frontend
#[derive(
    Debug, Default, Serialize, Deserialize, JsonSchema, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Copy
)]
pub enum Language {
    #[default]
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

#[derive(Error, Debug)]
pub enum LanguageError {
    #[error("Invalid language `{0}`")]
    ParseLanguageError(String),
}

impl FromStr for Language {
    type Err = LanguageError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "en" => Ok(Language::English),
            "ru" => Ok(Language::Russian),
            lang => Err(LanguageError::ParseLanguageError(lang.to_owned()))
        }
    }
}

#[derive(
    Debug, Serialize, Deserialize, JsonSchema, Clone, PartialEq, Eq, PartialOrd, Ord, Hash
)]
pub struct Email { pub email: String }

impl Email {
    pub fn validate(value: &str) -> bool {
        let email_regex = Regex::new(r"^([a-z0-9_+]([a-z0-9_+.]*[a-z0-9_+])?)@([a-z0-9]+([\-\.]{1}[a-z0-9]+)*\.[a-z]{2,6})").unwrap();
        email_regex.is_match(value)
    }

    pub fn from_str(value: &str) -> Option<Email>{
        if Email::validate(value){
            Some(Email {email: value.to_string()})
        } else { None }
    }
}

#[derive(
    Debug, Serialize, Deserialize, JsonSchema, Clone, PartialEq, Eq, PartialOrd, Ord, Hash
)]
pub struct PhoneNumber {pub number: String}

impl PhoneNumber {
    pub fn validate(value: &str) -> bool {
        let phone_regex = Regex::new(r"^(\+\d{1,2}\s?)?1?\-?\.?\s?\(?\d{3}\)?[\s.-]?\d{3}[\s.-]?\d{4}$|^(\+\d{1,2}\s?)?1?\-?\.?\s?\(?\d{3}\)?[\s.-]?\d{3}[\s.-]?\d{2}[\s.-]?\d{2}$").unwrap();
        phone_regex.is_match(value)
    }

    pub fn from_str(value: &str) -> Option<PhoneNumber>{
        if PhoneNumber::validate(value){
            Some(PhoneNumber {number: value.to_string()})
        } else { None }
    }
}

#[derive(
    Debug, Serialize, Deserialize, JsonSchema, Clone, PartialEq, Eq, PartialOrd, Ord, Hash
)]
pub struct TgName {pub tg_name: String}

#[derive(
    Debug, Serialize, Deserialize, JsonSchema, Clone, PartialEq, Eq, PartialOrd, Ord, Hash
)]
pub struct ChallengeResponse {
    pub user: String,
    pub challenge: String
}
