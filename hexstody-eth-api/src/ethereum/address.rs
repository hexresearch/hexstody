use ethereum::Account;
use bitcoin::Address;
use rocket::data::ToByteUnit;
use rocket::form::error::ErrorKind;
use rocket::form::{self, DataField, FromFormField, ValueField};
use rocket_okapi::okapi::schemars::{
    gen::SchemaGenerator,
    schema::{InstanceType, Metadata, Schema, SchemaObject},
    JsonSchema,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::str::FromStr;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BtcAddress(pub Address);

impl From<Address> for BtcAddress {
    fn from(value: Address) -> Self {
        BtcAddress(value)
    }
}

impl From<BtcAddress> for Address {
    fn from(value: BtcAddress) -> Self {
        value.0
    }
}

#[rocket::async_trait]
impl<'r> FromFormField<'r> for BtcAddress {
    fn from_value(field: ValueField<'r>) -> form::Result<'r, Self> {
        Address::from_str(field.value)
            .map_err(|e| ErrorKind::Custom(Box::new(e)).into())
            .map(BtcAddress)
    }

    async fn from_data(field: DataField<'r, '_>) -> form::Result<'r, Self> {
        // Retrieve the configured data limit or use `256KiB` as default.
        let limit = field.request.limits().get("address").unwrap_or(64.bytes());

        // Read the capped data stream, returning a limit error as needed.
        let bytes = field.data.open(limit).into_bytes().await?;
        if !bytes.is_complete() {
            Err((None, Some(limit)))?;
        }

        // Store the bytes in request-local cache
        let bytes = bytes.into_inner();
        let bytes = rocket::request::local_cache!(field.request, bytes);

        // Try to parse the name as UTF-8 or return an error if it fails.
        let hash_str = std::str::from_utf8(bytes)?;
        Address::from_str(hash_str)
            .map_err(|e| ErrorKind::Custom(Box::new(e)).into())
            .map(BtcAddress)
    }
}

impl JsonSchema for BtcAddress {
    fn schema_name() -> String {
        "bitcoin-address".to_owned()
    }

    fn json_schema(_: &mut SchemaGenerator) -> Schema {
        SchemaObject {
            instance_type: Some(InstanceType::String.into()),
            format: Some("bitcoin address".to_owned()),
            metadata: Some(Box::new(Metadata {
                examples: vec![
                    json!("3FZbgi29cpjq2GjdwV8eyHuJJnkLtktZc5"),
                    json!("bc1qar0srrr7xfkvy5l643lydnw9re59gtzzwf5mdq"),
                    json!("bc1pmzfrwwndsqmk5yh69yjr5lfgfg4ev8c0tsc06e"),
                ],
                ..Metadata::default()
            })),
            ..Default::default()
        }
        .into()
    }
}


#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EthAccount(pub Account);

impl From<Account> for EthAccount {
    fn from(value: Account) -> Self {
        EthAccount(value)
    }
}

impl From<EthAccount> for Account {
    fn from(value: EthAccount) -> Self {
        value.0
    }
}

#[rocket::async_trait]
impl<'r> FromFormField<'r> for EthAccount {
    fn from_value(field: ValueField<'r>) -> form::Result<'r, Self> {
        Address::from_str(field.value)
            .map_err(|e| ErrorKind::Custom(Box::new(e)).into())
            .map(EthAccount)
    }

    async fn from_data(field: DataField<'r, '_>) -> form::Result<'r, Self> {
        // Retrieve the configured data limit or use `256KiB` as default.
        let limit = field.request.limits().get("address").unwrap_or(64.bytes());

        // Read the capped data stream, returning a limit error as needed.
        let bytes = field.data.open(limit).into_bytes().await?;
        if !bytes.is_complete() {
            Err((None, Some(limit)))?;
        }

        // Store the bytes in request-local cache
        let bytes = bytes.into_inner();
        let bytes = rocket::request::local_cache!(field.request, bytes);

        // Try to parse the name as UTF-8 or return an error if it fails.
        let hash_str = std::str::from_utf8(bytes)?;
        Address::from_str(hash_str)
            .map_err(|e| ErrorKind::Custom(Box::new(e)).into())
            .map(EthAccount)
    }
}

impl JsonSchema for EthAccount {
    fn schema_name() -> String {
        "bitcoin-address".to_owned()
    }

    fn json_schema(_: &mut SchemaGenerator) -> Schema {
        SchemaObject {
            instance_type: Some(InstanceType::String.into()),
            format: Some("bitcoin address".to_owned()),
            metadata: Some(Box::new(Metadata {
                examples: vec![
                    json!("3FZbgi29cpjq2GjdwV8eyHuJJnkLtktZc5"),
                    json!("bc1qar0srrr7xfkvy5l643lydnw9re59gtzzwf5mdq"),
                    json!("bc1pmzfrwwndsqmk5yh69yjr5lfgfg4ev8c0tsc06e"),
                ],
                ..Metadata::default()
            })),
            ..Default::default()
        }
        .into()
    }
}
