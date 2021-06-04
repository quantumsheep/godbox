use actix_web::dev::Body;
use actix_web::http::header::CONTENT_TYPE;
use actix_web::http::HeaderValue;
use actix_web::http::StatusCode;
use actix_web::HttpResponse;
use actix_web::ResponseError;
use actix_web::Result as ActixResult;
use actix_web::web::Json;
use derive_more::{Display, Error};
use serde::Serialize;

pub type ApiResult<T> = ActixResult<Json<T>>;

#[derive(Debug, Serialize, Display, Error)]
#[display(fmt = "API Error {}: {}", status, message)]
pub struct ApiError {
    pub status: u16,
    pub message: String,
}

impl ApiError {
    pub fn new<S: Into<String>>(status: StatusCode, message: S) -> ApiError {
        ApiError {
            status: status.as_u16(),
            message: message.into(),
        }
    }

    pub fn not_found<S: Into<String>>(message: S) -> ApiError {
        ApiError::new(StatusCode::NOT_FOUND, message)
    }

    pub fn bad_request<S: Into<String>>(message: S) -> ApiError {
        ApiError::new(StatusCode::BAD_REQUEST, message)
    }

    pub fn internal_server_error<S: Into<String>>(message: S) -> ApiError {
        ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, message)
    }
}

impl ResponseError for ApiError {
    fn status_code(&self) -> StatusCode {
        StatusCode::from_u16(self.status).unwrap()
    }

    fn error_response(&self) -> HttpResponse {
        let mut resp = HttpResponse::new(self.status_code());
        resp.headers_mut().insert(
            CONTENT_TYPE,
            HeaderValue::from_static("application/json; charset=utf-8"),
        );

        resp.set_body(Body::from(serde_json::to_string(&self).unwrap()))
    }
}

impl<T> From<ApiError> for Result<T, ApiError> {
    fn from(error: ApiError) -> Result<T, ApiError> {
        Err(error)
    }
}

impl<T> From<ApiError> for ActixResult<T> {
    fn from(error: ApiError) -> ActixResult<T> {
        Err(error.into())
    }
}
