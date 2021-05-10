#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;

#[macro_use]
extern crate serde_json;

#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate derive_builder;

extern crate base64;

mod api_helpers;
mod isolate;

use crate::api_helpers::{ApiError, ApiResult};
use crate::isolate::{
    ExecutedCommandResult, Isolate, IsolatedBox, IsolatedBoxOptions, IsolatedBoxOptionsBuilder,
};
use rocket_contrib::json::Json;
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Deserialize, Debug)]
struct RunBodyDTO {
    compile_script: Option<String>,
    run_script: String,

    shared_environment: Option<HashMap<String, String>>,
    compile_environment: Option<HashMap<String, String>>,
    run_environment: Option<HashMap<String, String>>,

    profiling: Option<bool>,

    files: String,
}

#[derive(Serialize, Debug, Default, Builder)]
#[builder(setter(into, strip_option), default)]
struct RunResponseDTO {
    compile_status: Option<i64>,
    compile_stdout: Option<String>,
    compile_stderr: Option<String>,

    run_status: Option<i64>,
    run_stdout: Option<String>,
    run_stderr: Option<String>,

    profiling_result: Option<String>,
}

fn exec_command<S>(
    isolated_box: &IsolatedBox,
    command: S,
    options: IsolatedBoxOptions,
    error_if_not_success: bool,
) -> Result<ExecutedCommandResult, String>
where
    S: Into<String>,
{
    match isolated_box.exec(command, options) {
        Ok(result) => {
            if error_if_not_success && !result.status.success() {
                return Err(result.stdout);
            }

            Ok(result)
        }
        Err(e) => {
            return Err(e.to_string());
        }
    }
}

fn cleanup_isolated_box(isolate: &mut Isolate, isolated_box: &IsolatedBox) {
    if let Err(e) = isolate.destroy_box(&isolated_box) {
        println!("Failed to cleanup the box {}: {}", isolated_box.box_id, e);
    }
}

#[post("/run", data = "<body>")]
fn run(mut body: Json<RunBodyDTO>) -> ApiResult<RunResponseDTO> {
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
        cleanup_isolated_box(&mut isolate, &isolated_box);

        return ApiError::internal_server_error(format!(
            "Failed to upload files into the isolated environment: {}",
            e,
        ))
        .into();
    }

    if let Err(e) = exec_command(
        &isolated_box,
        "/usr/bin/unzip -n -qq /box/files.zip && /bin/rm /box/files.zip",
        IsolatedBoxOptionsBuilder::default().build().unwrap(),
        true,
    ) {
        cleanup_isolated_box(&mut isolate, &isolated_box);

        return ApiError::bad_request(format!("Error while unzipping files: {}", e)).into();
    }

    let mut compile_result_option: Option<ExecutedCommandResult> = None;

    if let Some(compile_script) = body.compile_script.clone() {
        if let Some(shared_environment) = body.shared_environment.clone() {
            if let Some(compile_environment) = &mut body.compile_environment {
                compile_environment.extend(shared_environment.into_iter());
            } else {
                body.compile_environment = Some(shared_environment);
            }
        }

        compile_result_option = match exec_command(
            &isolated_box,
            compile_script,
            IsolatedBoxOptionsBuilder::default()
                .environment(body.compile_environment.clone())
                .build()
                .unwrap(),
            false,
        ) {
            Ok(result) => {
                if !result.status.success() {
                    return Ok(Json(
                        RunResponseDTOBuilder::default()
                            .compile_status(result.status.code().unwrap())
                            .compile_stdout(result.stdout)
                            .compile_stderr(result.stderr)
                            .build()
                            .unwrap(),
                    ));
                }

                Some(result)
            }
            Err(e) => {
                cleanup_isolated_box(&mut isolate, &isolated_box);

                return ApiError::internal_server_error(format!("An error occured: {}", e)).into();
            }
        };
    }

    if let Some(shared_environment) = body.shared_environment.clone() {
        if let Some(run_environment) = &mut body.run_environment {
            run_environment.extend(shared_environment.into_iter());
        } else {
            body.run_environment = Some(shared_environment);
        }
    }

    let run_result = match exec_command(
        &isolated_box,
        body.run_script.clone(),
        IsolatedBoxOptionsBuilder::default()
            .environment(body.run_environment.clone())
            .profiling(body.profiling.unwrap_or(false))
            .build()
            .unwrap(),
        false,
    ) {
        Ok(result) => result,
        Err(e) => {
            cleanup_isolated_box(&mut isolate, &isolated_box);

            return ApiError::internal_server_error(format!("An error occured: {}", e)).into();
        }
    };

    let mut profiling_result_option = None;

    if body.profiling.unwrap_or(false) {
        match exec_command(
            &isolated_box,
            "/usr/bin/perf_5.10 script -i perf.data > out.txt && /bin/cat out.txt",
            IsolatedBoxOptionsBuilder::default().build().unwrap(),
            false,
        ) {
            Ok(result) => profiling_result_option = Some(result.stdout),
            Err(e) => {
                cleanup_isolated_box(&mut isolate, &isolated_box);

                return ApiError::internal_server_error(format!("An error occured: {}", e)).into();
            }
        };
    }

    cleanup_isolated_box(&mut isolate, &isolated_box);

    let mut result = RunResponseDTOBuilder::default();

    result
        .run_status(run_result.status.code().unwrap())
        .run_stdout(run_result.stdout)
        .run_stderr(run_result.stderr);

    if let Some(compile_result) = compile_result_option {
        result
            .compile_status(compile_result.status.code().unwrap())
            .compile_stdout(compile_result.stdout)
            .compile_stderr(compile_result.stderr);
    }

    if let Some(profiling_result) = profiling_result_option {
        result.profiling_result(profiling_result);
    }

    Ok(Json(result.build().unwrap()))
}

fn main() {
    rocket::ignite().mount("/", routes![run]).launch();
}
