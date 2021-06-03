use crate::isolate::{IsolatedBoxOptions, IsolatedBoxOptionsBuilder};
use merge::Merge;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize, Debug, Clone, Default, Merge)]
pub struct PhaseSandboxSettings {
    pub run_time_limit: Option<u64>,
    pub extra_time_limit: Option<u64>,
    pub wall_time_limit: Option<u64>,
    pub stack_size_limit: Option<u64>,
    pub process_count_limit: Option<u64>,
    pub memory_limit: Option<u64>,
    pub storage_limit: Option<u64>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct PhaseSettings {
    pub name: Option<String>,

    pub script: String,
    pub environment: Option<HashMap<String, String>>,

    pub sandbox_settings: Option<PhaseSandboxSettings>,
    pub profiling: Option<bool>,
}

impl From<PhaseSettings> for IsolatedBoxOptions {
    fn from(settings: PhaseSettings) -> Self {
        let mut options = IsolatedBoxOptionsBuilder::default();

        if let Some(sandbox_settings) = settings.sandbox_settings {
            if let Some(run_time_limit) = sandbox_settings.run_time_limit {
                options.run_time_limit(run_time_limit);
            }

            if let Some(extra_time_limit) = sandbox_settings.extra_time_limit {
                options.extra_time_limit(extra_time_limit);
            }

            if let Some(wall_time_limit) = sandbox_settings.wall_time_limit {
                options.wall_time_limit(wall_time_limit);
            }

            if let Some(stack_size_limit) = sandbox_settings.stack_size_limit {
                options.stack_size_limit(stack_size_limit);
            }

            if let Some(process_count_limit) = sandbox_settings.process_count_limit {
                options.process_count_limit(process_count_limit);
            }

            if let Some(memory_limit) = sandbox_settings.memory_limit {
                options.memory_limit(memory_limit);
            }

            if let Some(storage_limit) = sandbox_settings.storage_limit {
                options.storage_limit(storage_limit);
            }
        }

        options.environment(settings.environment.clone());

        if let Some(profiling) = settings.profiling {
            options.profiling(profiling);
        }

        options.build().unwrap()
    }
}
