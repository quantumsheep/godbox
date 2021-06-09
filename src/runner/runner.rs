use crate::api_helpers::ApiError;
use crate::isolate::{
    Isolate, IsolateMetadata, IsolateMetadataBuilder, IsolatedBox, IsolatedBoxOptions,
    IsolatedBoxOptionsBuilder, IsolatedExecutedCommandResult,
};
use serde::Serialize;
use std::io;
use std::os::unix::prelude::ExitStatusExt;
use std::process::ExitStatus;

use super::phase_settings::PhaseSettings;

#[derive(Serialize, Debug, Clone)]
pub struct RunnerPhaseResult {
    pub name: Option<String>,
    pub status: i32,
    pub stdout: String,
    pub stderr: String,
    pub metadata: IsolateMetadata,
}

pub struct Runner {
    isolate: Isolate,
}

impl Runner {
    pub fn new() -> io::Result<Runner> {
        let runner = Runner {
            isolate: Isolate::new(),
        };

        Ok(runner)
    }

    fn get_isolated_box(&self, isolated_box_id: u32) -> Result<&IsolatedBox, ApiError> {
        match self.isolate.boxes.get(&isolated_box_id) {
            Some(v) => Ok(v),
            None => {
                return ApiError::internal_server_error(format!(
                    "Unknown isolated box ID: {}",
                    isolated_box_id
                ))
                .into()
            }
        }
    }

    fn cleanup_isolated_box(&mut self, isolated_box_id: u32) -> Result<(), ApiError> {
        if let Err(e) = self.isolate.destroy_box(isolated_box_id) {
            println!("Failed to cleanup the box {}: {}", isolated_box_id, e);
        }

        Ok(())
    }

    pub fn setup(&mut self, files: &String) -> Result<u32, ApiError> {
        let isolated_box = match self.isolate.init_box() {
            Ok(v) => v,
            Err(e) => {
                return ApiError::internal_server_error(format!(
                    "Failed to initialize a new box: {}",
                    e
                ))
                .into()
            }
        };

        let files_buffer = match base64::decode(&files) {
            Ok(buf) => buf,
            Err(e) => {
                return ApiError::bad_request(format!("Error while reading files: {}", e)).into()
            }
        };

        if let Err(e) = isolated_box.upload_file("/box/files.zip", &files_buffer) {
            return ApiError::internal_server_error(format!(
                "Failed to upload files into the isolated environment: {}",
                e,
            ))
            .into();
        }

        let unzip_result = self.exec_isolated_box(
            &isolated_box,
            "/usr/bin/unzip -n -qq /box/files.zip && /bin/rm /box/files.zip",
            IsolatedBoxOptionsBuilder::default().build().unwrap(),
        );

        if !unzip_result.status.success() {
            self.cleanup_isolated_box(isolated_box.box_id)?;

            return ApiError::bad_request(format!(
                "Error while unzipping files: {}",
                unzip_result.stderr
            ))
            .into();
        }

        Ok(isolated_box.box_id)
    }

    fn exec_isolated_box<S>(
        &self,
        isolated_box: &IsolatedBox,
        script: S,
        options: IsolatedBoxOptions,
    ) -> IsolatedExecutedCommandResult
    where
        S: Into<String>,
    {
        match isolated_box.exec(script, options) {
            Ok(result) => result,
            Err(e) => IsolatedExecutedCommandResult {
                status: ExitStatus::from_raw(1),
                stderr: e.to_string(),
                stdout: "".to_string(),
                metadata: IsolateMetadataBuilder::default().build().unwrap(),
            },
        }
    }

    fn exec<S>(
        &self,
        isolated_box_id: u32,
        script: S,
        options: IsolatedBoxOptions,
    ) -> Result<IsolatedExecutedCommandResult, ApiError>
    where
        S: Into<String>,
    {
        let isolated_box = self.get_isolated_box(isolated_box_id)?;

        Ok(self.exec_isolated_box(&isolated_box, script, options))
    }

    pub fn run_phase(
        &mut self,
        isolated_box_id: u32,
        settings: &PhaseSettings,
    ) -> Result<RunnerPhaseResult, ApiError> {
        let result = self.exec(isolated_box_id, &settings.script, settings.clone().into())?;

        if !result.status.success() {
            self.cleanup_isolated_box(isolated_box_id)?;
        }

        Ok(RunnerPhaseResult {
            name: settings.name.clone(),
            status: result.status.code().unwrap_or(1),
            stderr: result.stderr,
            stdout: result.stdout,
            metadata: result.metadata,
        })
    }
}
