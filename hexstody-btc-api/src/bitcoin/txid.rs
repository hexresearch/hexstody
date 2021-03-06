use bitcoin::Txid;
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
pub struct BtcTxid(pub Txid);

impl From<Txid> for BtcTxid {
    fn from(value: Txid) -> Self {
        BtcTxid(value)
    }
}

impl From<BtcTxid> for Txid {
    fn from(value: BtcTxid) -> Self {
        value.0
    }
}

#[rocket::async_trait]
impl<'r> FromFormField<'r> for BtcTxid {
    fn from_value(field: ValueField<'r>) -> form::Result<'r, Self> {
        Txid::from_str(field.value)
            .map_err(|e| ErrorKind::Custom(Box::new(e)).into())
            .map(BtcTxid)
    }

    async fn from_data(field: DataField<'r, '_>) -> form::Result<'r, Self> {
        // Retrieve the configured data limit or use `256KiB` as default.
        let limit = field.request.limits().get("txid").unwrap_or(64.bytes());

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
        Txid::from_str(hash_str)
            .map_err(|e| ErrorKind::Custom(Box::new(e)).into())
            .map(BtcTxid)
    }
}

impl JsonSchema for BtcTxid {
    fn schema_name() -> String {
        "bitcoin-txid".to_owned()
    }

    fn json_schema(_: &mut SchemaGenerator) -> Schema {
        SchemaObject {
            instance_type: Some(InstanceType::String.into()),
            format: Some("bitcoin transaction id".to_owned()),
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
