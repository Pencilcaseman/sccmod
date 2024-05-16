use pyo3::prelude::*;
use std::path::Path;
use std::process::Command;

pub trait DownloaderImpl: Sized {
    /// Convert from a Python `Downloader` instance to a Rust [`Downloader`] instance.
    /// If this is not possible (due to an invalid value, for example), [`None`] is returned.
    ///
    /// # Note
    /// `object` must be a valid `Downloader` instance in Python.
    fn from_py(object: &Bound<PyAny>) -> Option<Self>;

    /// Download the source code into the specified `path`.
    ///
    /// If the action is performed successfully, the path specified by [`path`] will contain the
    /// source code (or binaries, depending on the [`Downloader`] implementation) of the requested
    /// program.
    ///
    /// # Errors
    /// The function will return [`Err::<String>`], where the [`String`] contains an appropriate
    /// error message.
    fn download<P: AsRef<Path>>(&self, path: &P) -> Result<(), String>;
}

#[derive(Debug, Clone)]
pub struct GitClone {
    url: String,
    branch: Option<String>,
    commit: Option<String>,
    submodules: bool,
}

impl GitClone {
    #[must_use]
    pub fn new(url: &str) -> Self {
        Self {
            url: url.to_string(),
            branch: None,
            commit: None,
            submodules: false,
        }
    }
}

impl DownloaderImpl for GitClone {
    fn from_py(object: &Bound<PyAny>) -> Option<Self> {
        let url: String = object
            .getattr("url")
            .expect("Failed to find attribute .url")
            .extract()
            .expect("Failed to extract url");

        let branch: Option<String> = object.getattr("branch").ok()?.extract().ok();
        let commit: Option<String> = object.getattr("commit").ok()?.extract().ok();
        let submodules: bool = object.getattr("submodules").ok()?.extract().ok()?;

        Some(Self {
            url,
            branch,
            commit,
            submodules,
        })
    }

    fn download<P: AsRef<Path>>(&self, path: &P) -> Result<(), String> {
        // Check if the directory already exists

        let skip_clone = std::fs::try_exists(path).map_err(|err| err.to_string())?;

        if skip_clone {
            crate::log::warn("Module download directory already exists. Pulling latest changes");
        } else {
            let mut command = Command::new("git");
            command.arg("clone");
            command.arg(&self.url);

            if let Some(branch) = &self.branch {
                command.arg("-b");
                command.arg(branch);
            }

            if self.submodules {
                command.arg("--recurse");
            }

            command.arg(path.as_ref());
            command.stdout(std::process::Stdio::piped());
            command.stderr(std::process::Stdio::piped());

            let spawn = command.spawn().map_err(|e| e.to_string())?;
            let (result, stdout, stderr) = crate::cli::child_logger(spawn);

            if result.is_err() {
                return Err("Failed to run git command".to_string());
            }
            let result = result.unwrap();

            if !result.success() {
                return Err(format!(
                    "Failed to clone repository: \n{}\n{}",
                    stdout.join("\n"),
                    stderr.join("\n")
                ));
            }
        }

        // Checkout or pull, depending on the commit specified
        let mut command = Command::new("git");
        command.current_dir(path);

        let msg = match &self.commit {
            Some(commit) => {
                command.arg("checkout");
                command.arg(commit);
                format!("Failed to checkout commit '{commit}'")
            }
            None => {
                command.arg("pull");
                "Failed to pull changes".to_string()
            }
        };

        command.stdout(std::process::Stdio::piped());
        command.stderr(std::process::Stdio::piped());

        let spawn = command.spawn().map_err(|e| e.to_string())?;
        let (result, stdout, stderr) = crate::cli::child_logger(spawn);

        if result.is_err() || !result.unwrap().success() {
            return Err(format!(
                "{msg}: \n{}\n{}",
                stdout.join("\n"),
                stderr.join("\n")
            ));
        }

        // if let Some(commit) = &self.commit {
        //     let mut command = Command::new("git");
        //     command.current_dir(path);
        //     command.arg("checkout");
        //     command.arg(commit);
        //     command.stdout(std::process::Stdio::piped());
        //     command.stderr(std::process::Stdio::piped());
        //
        //     let spawn = command.spawn().map_err(|e| e.to_string())?;
        //     let (result, stdout, stderr) = crate::cli::child_logger(spawn);
        //
        //     if result.is_err() || !result.unwrap().success() {
        //         return Err(format!(
        //             "Failed to checkout commit {commit:?}: \n{}\n{}",
        //             stdout.join("\n"),
        //             stderr.join("\n")
        //         ));
        //     }
        // } else {
        //     // No commit specified, so pull latest changes
        //
        //     let mut command = Command::new("git");
        //     command.current_dir(path);
        //     command.arg("pull");
        //     command.stdout(std::process::Stdio::piped());
        //     command.stderr(std::process::Stdio::piped());
        //
        //     let spawn = command.spawn().map_err(|e| e.to_string())?;
        //     let (result, stdout, stderr) = crate::cli::child_logger(spawn);
        //
        //     if result.is_err() || !result.unwrap().success() {
        //         return Err(format!(
        //             "Failed to pull: \n{}\n{}",
        //             stdout.join("\n"),
        //             stderr.join("\n")
        //         ));
        //     }
        // }

        Ok(())
    }
}

#[derive(Debug)]
pub enum Downloader {
    GitClone(GitClone),
    Curl,
}

impl DownloaderImpl for Downloader {
    fn from_py(object: &Bound<PyAny>) -> Option<Self> {
        let name = object.get_type().name().unwrap().to_string();

        match name.as_str() {
            "GitClone" => Some(Self::GitClone(GitClone::from_py(object)?)),
            _ => None,
        }
    }

    fn download<P: AsRef<Path>>(&self, path: &P) -> Result<(), String> {
        match self {
            Self::GitClone(clone) => clone.download(path),
            Self::Curl => Err("Not implemented yet".to_string()),
        }
    }
}
