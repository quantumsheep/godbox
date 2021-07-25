use crate::api_helpers::{ApiError, ApiResult};
use crate::runner::phase_settings::{PhaseSandboxSettings, PhaseSettings};
use crate::runner::runner::Runner;
use crate::runner::runner::RunnerPhaseResult;
use crate::utils;
use actix_web::{post, web::Json};
use merge::Merge;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
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

fn is_over_cap_limit_env(current_option: Option<u64>, name: &str) -> bool {
    match current_option {
        Some(current) => current > utils::parsed_env::get(name, u64::MAX),
        None => false,
    }
}

fn check_body(body: &actix_web_validator::Json<RunBodyDTO>) -> Result<(), ApiError> {
    fn setting_max_value_error(origin: &str, env_name: &str) -> ApiError {
        ApiError::bad_request(format!(
            "{}: maximum allowed value is {}",
            origin,
            env::var(env_name).unwrap_or(u64::MAX.to_string())
        ))
    }

    macro_rules! check_cap_limit {
        ($origin_str:expr, $origin_expr:expr, $env_name:expr) => {
            if is_over_cap_limit_env($origin_expr, $env_name) {
                return setting_max_value_error($origin_str, $env_name).into();
            }
        };
    }

    if let Some(sandbox_settings) = &body.sandbox_settings {
        #[cfg_attr(rustfmt, rustfmt_skip)]
        {
            check_cap_limit!("sandbox_settings.run_time_limit", sandbox_settings.run_time_limit, "MAX_RUN_TIME_LIMIT");
            check_cap_limit!("sandbox_settings.extra_time_limit", sandbox_settings.extra_time_limit, "MAX_EXTRA_TIME_LIMIT");
            check_cap_limit!("sandbox_settings.wall_time_limit", sandbox_settings.wall_time_limit, "MAX_WALL_TIME_LIMIT");
            check_cap_limit!("sandbox_settings.stack_size_limit", sandbox_settings.stack_size_limit, "MAX_STACK_SIZE_LIMIT");
            check_cap_limit!("sandbox_settings.process_count_limit", sandbox_settings.process_count_limit, "MAX_PROCESS_COUNT_LIMIT");
            check_cap_limit!("sandbox_settings.memory_limit", sandbox_settings.memory_limit, "MAX_MEMORY_LIMIT");
            check_cap_limit!("sandbox_settings.storage_limit", sandbox_settings.storage_limit, "MAX_STORAGE_LIMIT");
        }
    }

    let allow_profiling = match env::var("ALLOW_PROFILING") {
        Ok(value) => {
            let lowercase_value = value.to_lowercase();
            ["true", "yes"].iter().any(|&s| s == lowercase_value)
        },
        Err(_) => true,
    };

    for i in 0..body.phases.len() {
        let phase_settings = body.phases[i].clone();

        if let Some(profiling) = phase_settings.profiling {
            if profiling && !allow_profiling {
                return ApiError::bad_request("Profiling is not allowed").into();
            }
        }

        if let Some(sandbox_settings) = phase_settings.sandbox_settings {
            #[cfg_attr(rustfmt, rustfmt_skip)]
            {
                check_cap_limit!(&format!("phases[{}].sandbox_settings.run_time_limit", i), sandbox_settings.run_time_limit, "MAX_RUN_TIME_LIMIT");
                check_cap_limit!(&format!("phases[{}].sandbox_settings.extra_time_limit", i), sandbox_settings.extra_time_limit, "MAX_EXTRA_TIME_LIMIT");
                check_cap_limit!(&format!("phases[{}].sandbox_settings.wall_time_limit", i), sandbox_settings.wall_time_limit, "MAX_WALL_TIME_LIMIT");
                check_cap_limit!(&format!("phases[{}].sandbox_settings.stack_size_limit", i), sandbox_settings.stack_size_limit, "MAX_STACK_SIZE_LIMIT");
                check_cap_limit!(&format!("phases[{}].sandbox_settings.process_count_limit", i), sandbox_settings.process_count_limit, "MAX_PROCESS_COUNT_LIMIT");
                check_cap_limit!(&format!("phases[{}].sandbox_settings.memory_limit", i), sandbox_settings.memory_limit, "MAX_MEMORY_LIMIT");
                check_cap_limit!(&format!("phases[{}].sandbox_settings.storage_limit", i), sandbox_settings.storage_limit, "MAX_STORAGE_LIMIT");
            }
        }
    }

    Ok(())
}

#[post("/run")]
pub async fn route(body: actix_web_validator::Json<RunBodyDTO>) -> ApiResult<RunResponseDTO> {
    if let Err(e) = check_body(&body) {
        return e.into();
    }

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

        let status = result.status;

        results.push(result);

        if status != 0 {
            break;
        }
    }

    Ok(Json(RunResponseDTO { phases: results }))
}
