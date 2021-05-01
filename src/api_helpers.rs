use rocket::http::ContentType;
use rocket::http::Status;
use rocket::response::{self, Responder, Response};
use rocket::Request;
use rocket_contrib::json::Json;
use serde::Serialize;
use serde::Serializer;
use std::io::Cursor;

#[derive(Debug)]
pub struct ApiStatus(Status);

impl ApiStatus {
    pub fn http_status(&self) -> Status {
        return self.0;
    }
}

impl Serialize for ApiStatus {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u16(self.http_status().code)
    }
}

#[derive(Debug, Serialize)]
pub struct ApiError {
    pub status: ApiStatus,
    pub message: String,
}

impl ApiError {
    pub fn new<S: Into<String>>(status: Status, message: S) -> ApiError {
        ApiError {
            status: ApiStatus(status),
            message: message.into(),
        }
    }

    pub fn not_found<S: Into<String>>(message: S) -> ApiError {
        ApiError::new(Status::NotFound, message)
    }

    pub fn bad_request<S: Into<String>>(message: S) -> ApiError {
        ApiError::new(Status::BadRequest, message)
    }

    pub fn internal_server_error<S: Into<String>>(message: S) -> ApiError {
        ApiError::new(Status::InternalServerError, message)
    }
}

impl<T> From<ApiError> for Result<T, ApiError> {
    fn from(error: ApiError) -> Result<T, ApiError> {
        Err(error)
    }
}

impl<'r> Responder<'r> for ApiError {
    fn respond_to(self, _: &Request) -> response::Result<'r> {
        match serde_json::to_string(&self) {
            Ok(body) => Response::build()
                .status(self.status.http_status())
                .header(ContentType::JSON)
                .sized_body(Cursor::new(body))
                .ok(),
            Err(e) => Response::build()
                .status(Status::InternalServerError)
                .header(ContentType::JSON)
                .sized_body(Cursor::new(
                    json!({
                        "status": 500,
                        "message": format!("Failed to encode error to JSON: {}", e),
                    })
                    .to_string(),
                ))
                .ok(),
        }
    }
}

pub type ApiResult<T> = Result<Json<T>, ApiError>;
