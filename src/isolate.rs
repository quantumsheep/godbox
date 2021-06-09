use rand::{thread_rng, Rng};
use serde::Serialize;
use std::fs::{self, File};
use std::io;
use std::io::prelude::*;
use std::os::unix::prelude::ExitStatusExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::process::ExitStatus;
use std::{collections::HashMap, process::Stdio};

#[derive(Debug)]
pub struct ExecutedCommandResult {
    pub status: ExitStatus,
    pub stdout: String,
    pub stderr: String,
}

fn exec_command<I, S>(
    args: I,
    stdout: Option<Stdio>,
    stderr: Option<Stdio>,
) -> io::Result<ExecutedCommandResult>
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    let mut args_string: Vec<String> = args.into_iter().map(Into::into).collect();

    let program = args_string.remove(0);

    println!(
        "Executing command: {} {}",
        program,
        args_string.join(" ").to_string()
    );

    let output = Command::new(program)
        .args(args_string)
        .stdout(stdout.unwrap_or(Stdio::piped()))
        .stderr(stderr.unwrap_or(Stdio::piped()))
        .spawn()?
        .wait_with_output()?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    Ok(ExecutedCommandResult {
        status: output.status,
        stdout,
        stderr,
    })
}

#[derive(Default, Debug, Builder, Clone, Serialize)]
#[builder(default)]
pub struct IsolateMetadata {
    pub time: Option<f64>,
    pub time_wall: Option<f64>,
    pub max_rss: Option<u64>,
    pub csw_voluntary: Option<u64>,
    pub csw_forced: Option<u64>,
    pub cg_mem: Option<u64>,
    pub exit_code: Option<i32>,
    pub status: Option<String>,
}

impl From<String> for IsolateMetadata {
    fn from(string: String) -> Self {
        let mut builder = IsolateMetadataBuilder::default();

        for metadata in string.lines() {
            let values = metadata.split(':').collect::<Vec<_>>();

            if values.len() < 2 {
                continue;
            }

            let key = values[0];
            let value = values[1];

            match key {
                "time" => builder.time(Some(value.parse().unwrap())),
                "time-wall" => builder.time_wall(Some(value.parse().unwrap())),
                "max-rss" => builder.max_rss(Some(value.parse().unwrap())),
                "csw-voluntary" => builder.csw_voluntary(Some(value.parse().unwrap())),
                "csw-forced" => builder.csw_forced(Some(value.parse().unwrap())),
                "cg-mem" => builder.cg_mem(Some(value.parse().unwrap())),
                "exitcode" => builder.exit_code(Some(value.parse().unwrap())),
                "status" => builder.status(Some(value.to_string())),
                _ => &mut builder,
            };
        }

        builder.build().unwrap()
    }
}

#[derive(Debug, Clone)]
pub struct IsolatedExecutedCommandResult {
    pub status: ExitStatus,
    pub stdout: String,
    pub stderr: String,
    pub metadata: IsolateMetadata,
}

#[derive(Debug, Clone)]
pub struct IsolatedBox {
    pub box_id: u32,
    pub workdir: String,

    stdout_file: String,
    stderr_file: String,
    metadata_file: String,
}

#[derive(Default, Debug, Builder, Clone)]
#[builder(setter(into))]
pub struct IsolatedBoxOptions {
    #[builder(default)]
    pub environment: Option<HashMap<String, String>>,

    #[builder(default = "false")]
    pub profiling: bool,

    #[builder(default = "5")]
    pub run_time_limit: u64,

    #[builder(default = "0")]
    pub extra_time_limit: u64,

    #[builder(default = "10")]
    pub wall_time_limit: u64,

    #[builder(default = "128000")]
    pub stack_size_limit: u64,

    #[builder(default = "120")]
    pub process_count_limit: u64,

    #[builder(default = "512000")]
    pub memory_limit: u64,

    #[builder(default = "10240")]
    pub storage_limit: u64,
}

impl IsolatedBox {
    pub fn new(box_id: u32) -> io::Result<IsolatedBox> {
        let output = exec_command(
            vec!["isolate", "--cg", &format!("-b {}", box_id), "--init"],
            None,
            None,
        )?;

        let workdir = output.stdout.trim().to_string();

        let stdout_file = Self::create_file(workdir.clone(), "stdout")?;
        let stderr_file = Self::create_file(workdir.clone(), "stderr")?;
        let metadata_file = Self::create_file(workdir.clone(), "metadata")?;

        Ok(IsolatedBox {
            box_id,
            workdir,
            stdout_file,
            stderr_file,
            metadata_file,
        })
    }

    fn create_file<S1, S2>(workdir: S1, filename: S2) -> io::Result<String>
    where
        S1: Into<String>,
        S2: Into<String>,
    {
        let filepath = format!("{}/{}", workdir.into(), filename.into());

        exec_command(vec!["touch", &filepath], None, None)?;
        exec_command(vec!["chown", "$(whoami):", &filepath], None, None)?;

        Ok(filepath)
    }

    pub fn upload_file<S: Into<String>>(&self, path_string: S, buf: &[u8]) -> io::Result<PathBuf> {
        let path = Path::new(&path_string.into()).to_owned();

        let separator = match path.is_absolute() {
            true => "",
            false => "/",
        };

        if let Some(parent) = path.parent() {
            let directory = format!("{}{}{}", self.workdir, separator, parent.to_string_lossy());
            fs::create_dir_all(directory)?;
        }

        let file_absolute_path = format!("{}{}{}", self.workdir, separator, path.to_string_lossy());

        let mut file = File::create(&file_absolute_path)?;
        file.write_all(buf)?;

        Ok(Path::new(&file_absolute_path).to_owned())
    }

    pub fn exec<S>(
        &self,
        script: S,
        options: IsolatedBoxOptions,
    ) -> io::Result<IsolatedExecutedCommandResult>
    where
        S: Into<String>,
    {
        let box_id_arg = format!("-b {}", self.box_id);
        let metadata_arg = format!("-M{}", self.metadata_file);
        let run_time_limit_arg = format!("-t {}", options.run_time_limit);
        let extra_time_limit_arg = format!("-x {}", options.extra_time_limit);
        let wall_time_limit_arg = format!("-w {}", options.wall_time_limit);
        let stack_size_limit_arg = format!("-k {}", options.stack_size_limit);
        let process_count_limit_arg = format!("-p{}", options.process_count_limit);
        let memory_limit_arg = format!("--cg-mem={}", options.memory_limit);
        let storage_limit_arg = format!("-f {}", options.storage_limit);

        let isolate_args = vec![
            "isolate",
            "--cg",
            // Silent mode - Disable status messages printed to stderr, except for fatal errors of the sandbox itself
            "-s",
            // Box ID
            &box_id_arg,
            // Metadata file
            &metadata_arg,
            // Run time limit
            &run_time_limit_arg,
            // Extra time limit
            &extra_time_limit_arg,
            // Wall Time limit
            &wall_time_limit_arg,
            // Stack size limit
            &stack_size_limit_arg,
            // Process count limit
            &process_count_limit_arg,
            // Enable per process/thread time limit
            "--cg-timing",
            // Memory limit in KB
            &memory_limit_arg,
            // Storage size limit in KB
            &storage_limit_arg,
        ];

        let mut environment_variables = vec![
            "-EHOME=/tmp".into(),
            "-EPATH=/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin".into(),
        ];

        if let Some(environment) = options.environment.clone() {
            for (key, value) in environment.iter() {
                environment_variables.push(format!(
                    "-E{}={}",
                    key.replace("\\", "\\\\").replace("\"", "\\\""),
                    value.replace("\\", "\\\\").replace("\"", "\\\"")
                ));
            }
        }

        let mut args: Vec<String> = vec![];
        args.append(&mut isolate_args.iter().map(|&v| v.into()).collect());

        args.append(&mut environment_variables);

        args.append(&mut vec![
            // Run a command
            "--run".into(),
            "--".into(),
        ]);

        let script_name = format!("/box/.script-{}.sh", thread_rng().gen::<u64>());

        self.upload_file(
            script_name.clone(),
            format!("{}\n", script.into()).as_bytes(),
        )?;

        if options.profiling {
            args.append(&mut vec![
                "/usr/bin/perf_5.10".into(),
                "record".into(),
                "-g".into(),
            ]);
        }

        args.append(&mut vec!["/bin/bash".into(), script_name.clone()]);

        let stdout_stream = File::create(self.stdout_file.clone())?;
        let stderr_stream = File::create(self.stderr_file.clone())?;

        let result = exec_command(
            args,
            Some(Stdio::from(stdout_stream)),
            Some(Stdio::from(stderr_stream)),
        )?;

        let stdout = fs::read_to_string(self.stdout_file.clone())?;
        let stderr = fs::read_to_string(self.stderr_file.clone())?;
        let metadata_string = fs::read_to_string(self.metadata_file.clone())?;

        let metadata = IsolateMetadata::from(metadata_string);

        println!("{:?}", metadata);

        Ok(IsolatedExecutedCommandResult {
            status: match metadata.exit_code {
                Some(exit_code) => ExitStatus::from_raw(exit_code),
                None => result.status,
            },
            stdout,
            stderr,
            metadata,
        })
    }
}

#[derive(Debug)]
pub struct Isolate {
    pub boxes: HashMap<u32, IsolatedBox>,
}

impl Isolate {
    pub fn new() -> Isolate {
        Isolate {
            boxes: HashMap::new(),
        }
    }

    pub fn init_box(&mut self) -> io::Result<IsolatedBox> {
        let box_id = thread_rng().gen_range(0..=(i32::MAX as u32));
        let isolated_box = IsolatedBox::new(box_id)?;

        self.boxes.insert(box_id, isolated_box.clone());

        Ok(isolated_box)
    }

    fn cleanup(&self, isolated_box_id: u32) -> io::Result<ExecutedCommandResult> {
        let box_id_arg = format!("-b {}", isolated_box_id);

        let isolate_args = vec!["isolate", "--cg", &box_id_arg, "--cleanup"];

        exec_command(isolate_args, None, None)
    }

    pub fn destroy_box(&mut self, isolated_box_id: u32) -> io::Result<()> {
        self.cleanup(isolated_box_id)?;

        self.boxes.remove(&isolated_box_id);

        Ok(())
    }
}
