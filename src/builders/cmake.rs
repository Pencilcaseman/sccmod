use pyo3::prelude::PyAnyMethods;
use pyo3::{Bound, PyAny};
use std::{fs, path, path::Path, process::Command};

use crate::{builders::builder_trait::BuilderImpl, cli::child_logger, log};

#[derive(Debug)]
pub enum CMakeBuildType {
    Debug,
    Release,
    RelWithDebInfo,
    MinSizeRel,
}

#[derive(Debug)]
pub struct CMake {
    pub build_type: CMakeBuildType,
    pub jobs: usize,
    pub configure_flags: Option<Vec<String>>,
}

impl CMake {
    fn configure<P0: AsRef<Path> + std::fmt::Debug, P1: AsRef<Path> + std::fmt::Debug>(
        &self,
        source_path: &P0,
        output_path: &P1,
    ) -> Result<(), String> {
        let source_path = path::absolute(source_path).map_err(|err| err.to_string())?;
        let output_path = path::absolute(output_path).map_err(|err| err.to_string())?;

        // Attempt to create the output directory if necessary
        fs::create_dir_all(&output_path).map_err(|e| e.to_string())?;

        let mut cmake = Command::new("cmake");
        cmake.current_dir(&output_path);
        cmake.arg(&source_path);

        if let Some(flags) = &self.configure_flags {
            for flag in flags {
                cmake.arg(flag);
            }
        }

        // After other flags, so used preferentially
        cmake.arg(format!("-DCMAKE_BUILD_TYPE={:?}", self.build_type));
        cmake.stdout(std::process::Stdio::piped());
        cmake.stderr(std::process::Stdio::piped());

        let spawn = cmake.spawn().map_err(|e| e.to_string())?;
        let (result, stdout, stderr) = child_logger(spawn);

        if result.is_err() {
            return Err("Failed to run CMake command".to_string());
        }
        let result = result.unwrap();

        if !result.success() {
            return Err(format!(
                "Failed to execute {cmake:?}. Output:\n{}\n{}",
                stdout.join("\n"),
                stderr.join("\n")
            ));
        }

        Ok(())
    }

    fn compile<P: AsRef<Path> + std::fmt::Debug>(&self, path: &P) -> Result<(), String> {
        let mut cmake = Command::new("cmake");
        cmake.current_dir(path);
        cmake.arg("--build");
        cmake.arg(".");
        cmake.arg(format!("--config {:?}", self.build_type));
        cmake.arg(format!("--parallel {:?}", self.jobs));
        cmake.stdout(std::process::Stdio::piped());
        cmake.stderr(std::process::Stdio::piped());

        let spawn = cmake.spawn().map_err(|e| e.to_string())?;
        let (result, stdout, stderr) = child_logger(spawn);

        if result.is_err() {
            return Err("Failed to run CMake command".to_string());
        }
        let result = result.unwrap();

        if !result.success() {
            return Err(format!(
                "Failed to execute {cmake:?}. Output:\n{}\n{}",
                stdout.join("\n"),
                stderr.join("\n")
            ));
        }

        Ok(())
    }
}

impl BuilderImpl for CMake {
    fn from_py(object: &Bound<PyAny>) -> Option<Self> {
        let build_type = match object
            .getattr("build_type")
            .ok()?
            .extract::<String>()
            .ok()?
            .to_lowercase()
            .as_str()
        {
            "debug" => CMakeBuildType::Debug,
            "release" => CMakeBuildType::Release,
            "relwithdebinfo" => CMakeBuildType::RelWithDebInfo,
            "minsizerel" => CMakeBuildType::MinSizeRel,
            other => log::error(&format!("Unknown CMake build type {other}")),
        };

        let jobs: usize = object.getattr("jobs").ok()?.extract().ok()?;

        let configure_flags: Option<Vec<String>> =
            object.getattr("configure_flags").ok()?.extract().ok()?;

        Some(Self {
            build_type,
            jobs,
            configure_flags,
        })
    }

    fn build<P0: AsRef<Path> + std::fmt::Debug, P1: AsRef<Path> + std::fmt::Debug>(
        &self,
        source_path: &P0,
        output_path: &P1,
    ) -> Result<(), String> {
        self.configure(source_path, output_path)?;
        self.compile(output_path)?;
        Ok(())
    }

    fn install<P0: AsRef<Path>, P1: AsRef<Path>>(
        &self,
        build_path: &P0,
        install_path: &P1,
    ) -> Result<(), String> {
        let build_path = path::absolute(build_path).map_err(|err| err.to_string())?;
        let install_path = path::absolute(install_path).map_err(|err| err.to_string())?;

        fs::create_dir_all(&install_path).map_err(|e| e.to_string())?;

        if !build_path.exists() {
            return Err(format!("Build directory {build_path:?} does not exist"));
        }

        let mut cmake = Command::new("cmake");
        cmake.current_dir(&build_path);
        cmake.arg("--install");
        cmake.arg(".");
        cmake.arg(format!("--config {:?}", self.build_type));
        cmake.arg(format!(
            "--prefix {}",
            install_path
                .to_str()
                .ok_or("Failed to convert path to string")?
        ));

        cmake.stdout(std::process::Stdio::piped());
        cmake.stderr(std::process::Stdio::piped());
        let spawn = cmake.spawn().map_err(|e| e.to_string())?;

        let (result, stdout, stderr) = child_logger(spawn);

        if result.is_err() {
            return Err("Failed to run CMake command".to_string());
        }
        let result = result.unwrap();

        if !result.success() {
            return Err(format!(
                "Failed to execute {cmake:?}. Output:\n{}\n{}",
                stdout.join("\n"),
                stderr.join("\n")
            ));
        }

        Ok(())
    }
}
