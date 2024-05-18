use crate::{archive, log};
use pyo3::prelude::*;
use std::{fs, path::Path, process::Command};

pub trait DownloaderImpl: Sized {
    /// Convert from a Python `Downloader` instance to a Rust [`Downloader`] instance.
    /// If this is not possible (due to an invalid value, for example), [`Err`] is returned
    /// containing an error message as a [`String`]
    ///
    /// # Note
    /// `object` must be a valid `Downloader` instance in Python.
    ///
    /// # Errors
    ///
    /// Errors if the object cannot be converted correctly to a Rust type
    fn from_py(object: &Bound<PyAny>) -> Result<Self, String>;

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

#[derive(Debug, Clone)]
pub struct Curl {
    url: String,
    sha256: Option<String>,
    archive: Option<String>,
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
    fn from_py(object: &Bound<PyAny>) -> Result<Self, String> {
        let url: String = object
            .getattr("url")
            .map_err(|_| "Object does not contain an attribute named 'url'")?
            .extract()
            .map_err(|_| "Failed to convert 'url' to Rust String")?;

        let branch: Option<String> = match object.getattr("branch") {
            Ok(x) => x
                .extract()
                .map_err(|_| "Failed to convert 'branch' to Rust String")?,
            Err(_) => None,
        };

        let commit: Option<String> = match object.getattr("commit") {
            Ok(x) => x
                .extract()
                .map_err(|_| "Failed to convert 'commit' to Rust String")?,
            Err(_) => None,
        };

        let submodules: bool = match object.getattr("submodules") {
            Ok(x) => x
                .extract()
                .map_err(|_| "Failed to convert 'submodules' to Rust bool")?,
            Err(_) => false,
        };

        Ok(Self {
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
                command.arg("--recursive");
            }

            // Clone into `path`
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

        Ok(())
    }
}

impl Curl {
    #[must_use]
    pub fn new(url: &str) -> Self {
        Self {
            url: url.to_string(),
            sha256: None,
            archive: None,
        }
    }
}

impl DownloaderImpl for Curl {
    fn from_py(object: &Bound<PyAny>) -> Result<Self, String> {
        let url: String = object
            .getattr("url")
            .map_err(|_| "Object does not contain an attribute named 'url'")?
            .extract()
            .map_err(|_| "Could not convert attribute 'url' to Rust String")?;

        let sha256: Option<String> = match object.getattr("sha256") {
            Ok(x) => x
                .extract()
                .map_err(|_| "Could not convert attribute 'sha256' to Rust String")?,
            Err(_) => None,
        };

        let archive: Option<String> = match object.getattr("archive") {
            Ok(x) => x
                .extract()
                .map_err(|_| "Could not convert attribute 'archive' to Rust String")?,
            Err(_) => None,
        };

        Ok(Self {
            url,
            sha256,
            archive,
        })
    }

    fn download<P: AsRef<Path>>(&self, path: &P) -> Result<(), String> {
        // Todo: Check if the hashes match. If they do, there is no need to re-download

        // Ensure the directory exists
        fs::create_dir_all(&path).map_err(|e| e.to_string())?;

        const FILE_NAME: &str = "curl_download_result";

        let mut command = Command::new("curl");
        command.current_dir(path.as_ref());
        command.arg("-o");
        command.arg(FILE_NAME);
        command.arg(&self.url);

        command.stdout(std::process::Stdio::piped());
        command.stderr(std::process::Stdio::piped());

        let spawn = command.spawn().map_err(|e| e.to_string())?;
        let (result, stdout, stderr) = crate::cli::child_logger(spawn);

        if result.is_err() {
            return Err("Failed to run curl command".to_string());
        }
        let result = result.unwrap();

        if !result.success() {
            return Err(format!(
                "Failed to download from URL: \n{}\n{}",
                stdout.join("\n"),
                stderr.join("\n")
            ));
        }

        // Extract the archive if necessary
        if let Some(archive) = &self.archive {
            archive::extract(path, FILE_NAME, &archive)?;
        }

        Ok(())
    }
}

#[derive(Debug)]
pub enum Downloader {
    GitClone(GitClone),
    Curl(Curl),
}

impl DownloaderImpl for Downloader {
    fn from_py(object: &Bound<PyAny>) -> Result<Self, String> {
        let name = object.get_type().name().unwrap().to_string();

        match name.as_str() {
            "GitClone" => Ok(Self::GitClone(GitClone::from_py(object)?)),
            "Curl" => Ok(Self::Curl(Curl::from_py(object)?)),
            _ => Err("Invalid downloader type".to_string()),
        }
    }

    fn download<P: AsRef<Path>>(&self, path: &P) -> Result<(), String> {
        match self {
            Self::GitClone(clone) => clone.download(path),
            Self::Curl(curl) => curl.download(path),
        }
    }
}
