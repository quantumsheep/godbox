use rand::{thread_rng, Rng};
use std::ffi::OsStr;
use std::fs::{self, File};
use std::io;
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

#[derive(Debug)]
pub struct IsolatedBox {
    pub box_id: u32,
    pub workdir: String,
}

impl IsolatedBox {
    pub fn new(box_id: u32, workdir: String) -> IsolatedBox {
        IsolatedBox { box_id, workdir }
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
            println!("directory: {}", directory);
            fs::create_dir_all(directory)?;
        }

        let file_absolute_path = format!("{}{}{}", self.workdir, separator, path.to_string_lossy());
        println!("file_absolute_path: {}", file_absolute_path);

        let mut file = File::create(&file_absolute_path)?;
        file.write_all(buf)?;

        Ok(Path::new(&file_absolute_path).to_owned())
    }

    pub fn exec<I, S>(&self, command: I) -> io::Result<Output>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        let mut cmd = Command::new("isolate");

        // Enable control groups
        cmd.arg("--cg")
            // Box ID
            .arg(format!("-b {}", self.box_id))
            // stderr -> stdout
            .arg("--stderr-to-stdout")
            // Run time limit
            .arg("-t 2")
            // Extra time limit
            .arg("-x 1")
            // Wall Time limit
            .arg("-w 4")
            // Stack size limit
            .arg("-k 128000")
            // Process count limit
            .arg("-p60")
            // Enable per process/thread time limit
            .arg("--no-cg-timing")
            // Memory limit in KB
            .arg("-m 512000")
            // Storage size limit in KB
            .arg("-f 10240")
            // Run a command
            .arg("--run")
            .arg("--")
            .args(command);

        println!("Executed in isolation: {:?}", cmd);

        return cmd.output();
    }
}

#[derive(Debug)]
pub struct Isolate {
    pub boxes: Vec<IsolatedBox>,
}

impl Isolate {
    pub fn new() -> Isolate {
        Isolate { boxes: vec![] }
    }

    pub fn init_box(&mut self) -> Result<IsolatedBox, io::Error> {
        let box_id = thread_rng().gen::<u32>();

        let output = Command::new("isolate")
            .arg("--cg")
            .arg(format!("-b {}", box_id))
            .arg("--init")
            .output()?;

        println!(
            "init() stdout: {}",
            String::from_utf8_lossy(&output.stdout.clone())
        );
        println!(
            "init() stdout: {}",
            String::from_utf8_lossy(&output.stderr.clone())
        );

        Ok(IsolatedBox::new(
            box_id,
            String::from_utf8_lossy(&output.stdout.clone()).trim().to_string(),
        ))
    }
}
