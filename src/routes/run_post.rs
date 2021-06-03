use crate::api_helpers::{ApiError, ApiResult};
use crate::runner::phase_settings::{PhaseRunSettings, PhaseSettings};
use crate::runner::runner::RunnerPhaseResult;
use crate::runner::runner::Runner;
use merge::Merge;
use rocket_contrib::json::Json;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Deserialize, Debug)]
pub struct RunBodyDTO {
    phases: Vec<PhaseSettings>,

    environment: Option<HashMap<String, String>>,
    isolation_settings: Option<PhaseRunSettings>,

    files: String,
}

#[derive(Serialize, Debug, Default, Builder)]
#[builder(setter(into, strip_option), default)]
pub struct RunResponseDTO {
    phases: Vec<RunnerPhaseResult>,
}

#[post("/run", data = "<body>")]
pub fn route(body: Json<RunBodyDTO>) -> ApiResult<RunResponseDTO> {
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

        if let Some(isolation_settings) = body.isolation_settings.clone() {
            if let Some(phase_isolation_settings) = &mut phase_settings.isolation_settings {
                phase_isolation_settings.merge(isolation_settings);
            } else {
                phase_settings.isolation_settings = Some(isolation_settings);
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
