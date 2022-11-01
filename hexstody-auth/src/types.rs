use async_trait::async_trait;
use rocket::{http::Status, request::FromRequest, outcome::Outcome};
use rocket_okapi::{okapi::openapi3::{Parameter, ParameterValue, Object}, request::{RequestHeaderInput, OpenApiFromRequest}};
use serde::{Serialize, Deserialize};

use crate::error as error;

#[derive(Debug, PartialEq, PartialOrd, Eq, Ord, Hash, Clone, Serialize, Deserialize)]
pub struct ApiKey(String);

#[async_trait]
impl<'r> FromRequest<'r> for ApiKey {
    type Error = error::Error;

    async fn from_request(request: &'r rocket::Request<'_>) -> rocket::request::Outcome<Self, Self::Error> {
        let api_key = request.headers().get_one("ApiKey");
        match api_key {
          Some(api_key) => {
            // check validity
            Outcome::Success(ApiKey(api_key.to_string()))
          },
          // token does not exist
          None => Outcome::Failure((Status::Unauthorized, error::Error::AuthRequired))
        }
    }
}

impl<'r> OpenApiFromRequest<'r> for ApiKey {
    fn from_request_input(
        gen: &mut rocket_okapi::gen::OpenApiGenerator,
        _name: String,
        required: bool,
    ) -> rocket_okapi::Result<rocket_okapi::request::RequestHeaderInput> {
        let schema = gen.json_schema::<String>();
        let description = Some("Contains API key for authorization".to_owned(),
        );
        let example = Some(serde_json::json!("123e4567-e89b-12d3-a456-426614174000"));
        Ok(RequestHeaderInput::Parameter(Parameter {
            name: "Signature-Data".to_owned(),
            location: "header".to_owned(),
            description: description,
            required,
            deprecated: false,
            allow_empty_value: false,
            value: ParameterValue::Schema {
                style: None,
                explode: None,
                allow_reserved: false,
                schema,
                example: example,
                examples: None,
            },
            extensions: Object::default(),
        }))
    }
}