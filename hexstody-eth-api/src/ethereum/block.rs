use web3::types::H256;
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
pub struct EthBlockHash(pub H256);

impl From<H256> for EthBlockHash {
    fn from(value: H256) -> Self {
        EthBlockHash(value)
    }
}

impl From<EthBlockHash> for H256{
    fn from(value: EthBlockHash) -> Self {
        value.0
    }
}

#[rocket::async_trait]
impl<'r> FromFormField<'r> for EthBlockHash {
    fn from_value(field: ValueField<'r>) -> form::Result<'r, Self> {
        H256::from_str(field.value)
            .map_err(|e| ErrorKind::Custom(Box::new(e)).into())
            .map(EthBlockHash)
    }
    async fn from_data(field: DataField<'r, '_>) -> form::Result<'r, Self> {
        // Retrieve the configured data limit or use `256KiB` as default.
        let limit = field
            .request
            .limits()
            .get("blockhash")
            .unwrap_or(64.bytes());

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
        H256::from_str(hash_str)
            .map_err(|e| ErrorKind::Custom(Box::new(e)).into())
            .map(EthBlockHash)
    }
}

impl JsonSchema for EthBlockHash {
    fn schema_name() -> String {
        "ethereum-block-hash".to_owned()
    }

    fn json_schema(_: &mut SchemaGenerator) -> Schema {
        SchemaObject {
            instance_type: Some(InstanceType::String.into()),
            format: Some("ethereum block hash".to_owned()),
            metadata: Some(Box::new(Metadata {
                examples: vec![json!(
                    "000000000019d6689c085ae165831e934ff763ae46a2a6c172b3f1b60a8ce26f"
                )],
                ..Metadata::default()
            })),
            ..Default::default()
        }
        .into()
    }
}
