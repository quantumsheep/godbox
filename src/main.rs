#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;

#[macro_use]
extern crate serde_json;

#[macro_use]
extern crate serde_derive;

extern crate base64;

mod api_helpers;
mod isolate;

use crate::api_helpers::{ApiError, ApiResult};
use crate::isolate::Isolate;
use rocket_contrib::json::Json;
use serde_derive::{Deserialize, Serialize};

#[derive(Deserialize, Debug)]
struct RunBodyDTO {
    language: String,
    files: String,
}

#[derive(Serialize, Debug)]
struct RunResponseDTO {
    output: String,
}

#[post("/run", data = "<body>")]
fn run(body: Json<RunBodyDTO>) -> ApiResult<RunResponseDTO> {
    let files_buffer = match base64::decode(&body.files) {
        Ok(buf) => buf,
        Err(e) => return ApiError::bad_request(format!("Error while reading files: {}", e)).into(),
    };

    let mut isolate = Isolate::new();

    let isolated_box = match isolate.init_box() {
        Ok(isolated_box) => isolated_box,
        Err(e) => {
            return ApiError::internal_server_error(format!(
                "Failed to initialize the isolated environment: {}",
                e
            ))
            .into()
        }
    };

    if let Err(e) = isolated_box.upload_file("/box/files.zip", &files_buffer) {
        return ApiError::internal_server_error(format!(
            "Failed to upload the files into the isolated environment: {}",
            e,
        ))
        .into();
    }

    let output = match isolated_box.exec(vec!["/usr/bin/unzip", "-n", "-qq", "/box/files.zip"]) {
        Ok(output) => String::from_utf8_lossy(&output.stdout).to_string(),
        Err(e) => {
            return ApiError::internal_server_error(format!("An error occured: {}", e)).into()
        }
    };

    return Ok(Json(RunResponseDTO { output }));
}

fn main() {
    rocket::ignite().mount("/", routes![run]).launch();
}
