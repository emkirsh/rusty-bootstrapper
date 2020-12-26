use std::fmt;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::process::Command;

use zip::result::ZipResult;
use zip::ZipArchive;

#[derive(Eq, PartialEq)]
pub struct LauncherBinary {
    path: PathBuf,
}

impl LauncherBinary {
    pub fn new(path: PathBuf) -> LauncherBinary {
        LauncherBinary { path }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn test_jar(&self, main_class: &str) -> ZipResult<bool> {
        let jar_file = File::open(self.path())?;
        let mut zip = ZipArchive::new(jar_file)?;

        zip.by_name(main_class).map(|e| e.is_file())
    }

    pub fn delete(&self) {
        std::fs::remove_file(self.path()).expect("Failed to remove a launcher file.");
    }

    pub fn create_launcher(&self, main_class: String, args: Vec<&str>) -> JavaLauncher {
        JavaLauncher {
            main_class,
            args: args.into_iter().map(String::from).collect(),
            opts: vec!(),
            jar_path: self.path.clone(),
        }
    }
}

#[derive(Debug)]
pub enum JavaError {
    IoError(std::io::Error),
    ExitCode(i32),
}

impl From<std::io::Error> for JavaError {
    fn from(err: std::io::Error) -> Self {
        JavaError::IoError(err)
    }
}

impl fmt::Display for JavaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            JavaError::IoError(err) =>
                write!(f, "IO error: {}", err),
            JavaError::ExitCode(code) =>
                write!(f, "Launcher exited with non-zero status code {}", code),
        }
    }
}

pub struct JavaLauncher {
    opts: Vec<String>,
    args: Vec<String>,
    jar_path: PathBuf,
    main_class: String,
}

impl JavaLauncher {
    pub fn launch(&self) -> Result<(), JavaError> {
        let cmd = Command::new("java")
            .args(&self.opts)
            .arg("-cp").arg(&self.jar_path)
            .arg(&self.main_class)
            .args(&self.args)
            .status()?;

        if !cmd.success() {
            let code = cmd.code().unwrap_or(1);
            Err(JavaError::ExitCode(code))
        } else {
            Ok(())
        }
    }
}
