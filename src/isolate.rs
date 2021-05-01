use rand::{thread_rng, Rng};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io;
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::process::ExitStatus;

#[derive(Debug)]
pub struct ExecutedCommandResult {
    pub status: ExitStatus,
    pub stdout: String,
    pub stderr: String,
}

fn exec_command<I, S>(args: I) -> io::Result<ExecutedCommandResult>
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

    let output = Command::new(program).args(args_string).output()?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    Ok(ExecutedCommandResult {
        status: output.status,
        stdout,
        stderr,
    })
}

#[derive(Debug, Clone)]
pub struct IsolatedBox {
    pub box_id: u32,
    pub workdir: String,

    stdout_file: String,
}

#[derive(Default, Debug, Builder)]
#[builder(setter(into))]
pub struct IsolatedBoxOptions {
    #[builder(default)]
    pub environment: Option<HashMap<String, String>>,

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
        let output = exec_command(vec!["isolate", "--cg", &format!("-b {}", box_id), "--init"])?;

        let workdir = output.stdout.trim().to_string();
        let stdout_file = format!("{}/stdout", workdir);

        exec_command(vec![
            "touch",
            &stdout_file,
            "&&",
            "chown",
            "$(whoami):",
            &stdout_file,
        ])?;

        Ok(IsolatedBox {
            box_id,
            workdir,
            stdout_file: stdout_file.clone(),
        })
    }

    pub fn upload_file<S: Into<String>>(
        &self,
        path_string: S,
        buf: &[u8],
    ) -> Result<PathBuf, io::Error> {
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

    pub fn exec<I, S>(
        &self,
        command: I,
        options: IsolatedBoxOptions,
    ) -> io::Result<ExecutedCommandResult>
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let box_id_arg = format!("-b {}", self.box_id);
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
            // Box ID
            &box_id_arg,
            // stderr -> stdout
            "--stderr-to-stdout",
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

        let mut environment_variables: Vec<String> = vec![];

        if let Some(environment) = options.environment {
            for (key, value) in environment.iter() {
                environment_variables.push(format!("-E{}={}", key, value));
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
        args.append(&mut command.into_iter().map(Into::into).collect());

        exec_command(args)
    }

    pub fn cleanup(&self) -> io::Result<ExecutedCommandResult> {
        let box_id_arg = format!("-b {}", self.box_id);

        let isolate_args = vec!["isolate", "--cg", &box_id_arg, "--cleanup"];

        exec_command(isolate_args)
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

    pub fn init_box(&mut self) -> Result<IsolatedBox, io::Error> {
        let box_id = thread_rng().gen_range(0..=(i32::MAX as u32));
        let isolated_box = IsolatedBox::new(box_id)?;

        self.boxes.insert(box_id, isolated_box.clone());

        Ok(isolated_box)
    }

    pub fn destroy_box(&mut self, isolated_box: &IsolatedBox) -> Result<(), io::Error> {
        isolated_box.cleanup()?;

        self.boxes.remove(&isolated_box.box_id);

        Ok(())
    }
}
