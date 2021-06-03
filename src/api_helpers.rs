use rocket::http::ContentType;
use rocket::http::Status;
use rocket::response::status::Custom;
use rocket::response::{self, Responder, Response};
use rocket::{Request, Catcher};
use rocket_contrib::json::Json;
use serde::Serialize;
use serde::Serializer;
use serde_json::json;
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
                        "name": "Internal Server Error",
                        "description": format!("Failed to encode error to JSON: {}", e),
                    })
                    .to_string(),
                ))
                .ok(),
        }
    }
}

pub type ApiResult<T> = Result<Json<T>, ApiError>;

#[derive(Serialize)]
struct ErrorResponseDTO<'r> {
    status: u32,
    name: &'r str,
    description: &'r str,
}

macro_rules! catchers {
    ($($code:expr, $name:expr, $description:expr, $fn_name:ident),+) => (
        let mut funcs = Vec::new();

        $(
            fn $fn_name<'r>(req: &'r Request) -> response::Result<'r> {
                Custom(Status::from_code($code).unwrap(),
                    response::content::Json(json!(ErrorResponseDTO {
                        status: $code,
                        name: $name,
                        description: $description,
                    }).to_string())
                ).respond_to(req)
            }

            funcs.push(Catcher::new($code, $fn_name));
        )+

        funcs
    )
}

pub fn generate_catchers() -> Vec<Catcher> {
    catchers! {
        400, "Bad Request", "The request could not be understood by the server due to malformed syntax.", handle_400,
        401, "Unauthorized", "The request requires user authentication.", handle_401,
        402, "Payment Required", "The request could not be processed due to lack of payment.", handle_402,
        403, "Forbidden", "The server refused to authorize the request.", handle_403,
        404, "Not Found", "The requested resource could not be found.", handle_404,
        405, "Method Not Allowed", "The request method is not supported for the requested resource.", handle_405,
        406, "Not Acceptable", "The requested resource is capable of generating only content not acceptable according to the Accept headers sent in the request.", handle_406,
        407, "Proxy Authentication Required", "Authentication with the proxy is required.", handle_407,
        408, "Request Timeout", "The server timed out waiting for the request.", handle_408,
        409, "Conflict", "The request could not be processed because of a conflict in the request.", handle_409,
        410, "Gone", "The resource requested is no longer available and will not be available again.", handle_410,
        411, "Length Required", "The request did not specify the length of its content, which is required by the requested resource.", handle_411,
        412, "Precondition Failed", "The server does not meet one of the preconditions specified in the request.", handle_412,
        413, "Payload Too Large", "The request is larger than the server is willing or able to process.", handle_413,
        414, "URI Too Long", "The URI provided was too long for the server to process.", handle_414,
        415, "Unsupported Media Type", "The request entity has a media type which the server or resource does not support.", handle_415,
        416, "Range Not Satisfiable", "The portion of the requested file cannot be supplied by the server.", handle_416,
        417, "Expectation Failed", "The server cannot meet the requirements of the Expect request-header field.", handle_417,
        418, "I'm a teapot", "I was requested to brew coffee, and I am a teapot.", handle_418,
        421, "Misdirected Request", "The server cannot produce a response for this request.", handle_421,
        422, "Unprocessable Entity", "The request was well-formed but was unable to be followed due to semantic errors.", handle_422,
        426, "Upgrade Required", "Switching to the protocol in the Upgrade header field is required.", handle_426,
        428, "Precondition Required", "The server requires the request to be conditional.", handle_428,
        429, "Too Many Requests", "Too many requests have been received recently.", handle_429,
        431, "Request Header Fields Too Large", "The server is unwilling to process the request because either an individual header field, or all the header fields collectively, are too large.", handle_431,
        451, "Unavailable For Legal Reasons", "The requested resource is unavailable due to a legal demand to deny access to this resource.", handle_451,
        500, "Internal Server Error", "The server encountered an internal error while processing this request.", handle_500,
        501, "Not Implemented", "The server either does not recognize the request method, or it lacks the ability to fulfill the request.", handle_501,
        503, "Service Unavailable", "The server is currently unavailable.", handle_503,
        504, "Gateway Timeout", "The server did not receive a timely response from an upstream server.", handle_504,
        510, "Not Extended", "Further extensions to the request are required for the server to fulfill it.", handle_510
    }
}
