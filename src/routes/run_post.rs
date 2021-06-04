use crate::api_helpers::{ApiError, ApiResult};
use crate::runner::phase_settings::{PhaseSandboxSettings, PhaseSettings};
use crate::runner::runner::Runner;
use crate::runner::runner::RunnerPhaseResult;
use actix_web::{post, web::Json};
use merge::Merge;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use validator::Validate;

#[derive(Deserialize, Debug, Validate)]
pub struct RunBodyDTO {
    phases: Vec<PhaseSettings>,

    environment: Option<HashMap<String, String>>,
    sandbox_settings: Option<PhaseSandboxSettings>,

    files: String,
}

#[derive(Serialize, Debug, Default, Builder)]
#[builder(setter(into, strip_option), default)]
pub struct RunResponseDTO {
    phases: Vec<RunnerPhaseResult>,
}

#[post("/run")]
pub async fn route(body: actix_web_validator::Json<RunBodyDTO>) -> ApiResult<RunResponseDTO> {
    let mut runner = match Runner::new() {
        Ok(v) => v,
        Err(e) => {
            return ApiError::internal_server_error(format!(
                "Failed to initialize the isolated environment: {}",
                e
            ))
            .into();
        }
    };

    let mut results = vec![];

    let isolated_box_id = match runner.setup(&body.files) {
        Ok(v) => v,
        Err(e) => return e.into(),
    };

    for i in 0..body.phases.len() {
        let mut phase_settings = body.phases[i].clone();

        phase_settings.name = phase_settings.name.or(Some(i.to_string()));

        if let Some(environment) = body.environment.clone() {
            if let Some(phase_environment) = &mut phase_settings.environment {
                phase_environment.extend(environment.into_iter());
            } else {
                phase_settings.environment = Some(environment);
            }
        }

        if let Some(sandbox_settings) = body.sandbox_settings.clone() {
            if let Some(phase_sandbox_settings) = &mut phase_settings.sandbox_settings {
                phase_sandbox_settings.merge(sandbox_settings);
            } else {
                phase_settings.sandbox_settings = Some(sandbox_settings);
            }
        }

        let result = match runner.run_phase(isolated_box_id, &phase_settings) {
            Ok(v) => v,
            Err(e) => return e.into(),
        };

        results.push(result);
    }

    Ok(Json(RunResponseDTO { phases: results }))
}
