use actix_web::{error::InternalError, App, HttpResponse, HttpServer};
use actix_web_validator::{Error, JsonConfig};
use serde::Serialize;
use std::{env, io};
use validator::ValidationErrors;

extern crate derive_more;

#[macro_use]
extern crate derive_builder;

mod api_helpers;
mod isolate;
mod routes;
mod runner;

#[derive(Serialize)]
pub struct ValidationErrorDTO {
    pub message: String,
    pub fields: Vec<String>,
}

impl From<&ValidationErrors> for ValidationErrorDTO {
    fn from(error: &ValidationErrors) -> Self {
        ValidationErrorDTO {
            message: "Validation error".to_owned(),
            fields: error
                .field_errors()
                .iter()
                .map(|(field, _)| field.to_string())
                .collect(),
        }
    }
}

#[actix_web::main]
async fn main() -> io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .app_data(
                JsonConfig::default()
                    .limit(
                        match env::var("API_MAX_PAYLOAD_SIZE")
                            .ok()
                            .and_then(|value| value.parse().ok())
                        {
                            Some(value) => value,
                            None => 32768,
                        },
                    )
                    .error_handler(|err, _| {
                        let json_error = match &err {
                            Error::Validate(error) => ValidationErrorDTO::from(error),
                            _ => ValidationErrorDTO {
                                message: err.to_string(),
                                fields: Vec::new(),
                            },
                        };

                        InternalError::from_response(err, HttpResponse::Conflict().json(json_error))
                            .into()
                    }),
            )
            .service(routes::run_post::route)
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}
