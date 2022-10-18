use std::fmt::Display;

use okapi::openapi3::Responses;
use rocket::Response;
use rocket::{http::Status, response::Responder};
use rocket_okapi::okapi::schemars::JsonSchema;
use rocket_okapi::response::OpenApiResponderInner;
use rocket_okapi::util::add_schema_response;
use serde::Serialize;
use serde_json::json;

pub type Result<T> = std::result::Result<T, ErrorMessage>;

pub trait HexstodyError {
    /// Error subtype, defines concrete error enum: hexstody_api, invoice_api etc
    fn subtype() -> &'static str; 
    /// Internal error code. Paired with subtype uniquely defines the error
    fn code(&self) -> u16;
    /// Server status code: 400, 403, 500 etc
    fn status(&self) -> u16;
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct ErrorMessage {
    /// Internal code of the error. Paired with subtype uniquely defines the error
    pub code: u16,
    /// Subtype
    pub subtype: String,
    /// Server code of the error: 400, 403, 500 etc
    pub status: u16,
    /// Error message
    pub message: String,
}

impl <E: HexstodyError + Display> From<E> for ErrorMessage {
    fn from(err: E) -> ErrorMessage {
        ErrorMessage { 
            code: err.code(),
            status: err.status(), 
            message: format!("{err}"), 
            subtype: E::subtype().to_string()
        }
    }
}

impl<'r, 'o: 'r> Responder<'r, 'o>  for ErrorMessage {
    fn respond_to(self, _: &'r rocket::Request<'_>) -> rocket::response::Result<'o> {
        rocket::error_!("[{}:{}]: {}", self.subtype, self.code, self.message);
        let resp = json!({
            "id": format!("{}:{}", self.subtype, self.code),
            "message": self.message
        });
        let resp = serde_json::to_string(&resp).unwrap_or_default();
        Response::build()
            .status(Status::from_code(self.status).unwrap_or_default())
            .header(rocket::http::ContentType::JSON)
            .sized_body(resp.len(), std::io::Cursor::new(resp))
            .ok()
    }
}

impl OpenApiResponderInner for ErrorMessage{
    fn responses(gen: &mut rocket_okapi::gen::OpenApiGenerator) -> rocket_okapi::Result<okapi::openapi3::Responses> {
        let mut responses = Responses::default();
        let schema = gen.json_schema::<ErrorMessage>();
        add_schema_response(&mut responses, 200, "application/json", schema.into())?;
        Ok(responses)
    }
}