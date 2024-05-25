use crate::{builders::builder_trait::BuilderImpl, log, shell::Shell};
use pyo3::prelude::PyAnyMethods;
use pyo3::{Bound, PyAny};
use std::{fs, path, path::Path};

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
        dependencies: &[String],
    ) -> Result<(), String> {
        let source_path = path::absolute(source_path).map_err(|err| err.to_string())?;
        let output_path = path::absolute(output_path).map_err(|err| err.to_string())?;

        // Attempt to create the output directory if necessary
        fs::create_dir_all(&output_path).map_err(|e| e.to_string())?;

        let mut shell = Shell::default();
        shell.set_current_dir(output_path.to_str().unwrap());

        for dep in dependencies {
            shell.add_command(&format!("module load {dep}"));
        }

        let mut cmake_cmd = format!("cmake {source_path:?}");

        if let Some(flags) = &self.configure_flags {
            for flag in flags {
                // cmake.arg(flag);
                cmake_cmd.push_str(&format!(" {flag}"));
            }
        }

        cmake_cmd.push_str(&format!("-DCMAKE_BUILD_TYPE={:?}", self.build_type));
        shell.add_command(&cmake_cmd);

        let (result, stdout, stderr) = shell.exec();

        if result.is_err() {
            return Err("Failed to run CMake command".to_string());
        }
        let result = result.unwrap();

        if !result.success() {
            return Err(format!(
                "Failed to execute command. Output:\n{}\n{}",
                stdout.join("\n"),
                stderr.join("\n")
            ));
        }

        Ok(())
    }

    fn compile<P: AsRef<Path> + std::fmt::Debug>(
        &self,
        path: &P,
        dependencies: &[String],
    ) -> Result<(), String> {
        // let mut cmake = Command::new("cmake");
        // cmake.current_dir(path);
        // cmake.arg("--build");
        // cmake.arg(".");
        // cmake.arg(format!("--config {:?}", self.build_type));
        // cmake.arg(format!("--parallel {:?}", self.jobs));
        // cmake.stdout(std::process::Stdio::piped());
        // cmake.stderr(std::process::Stdio::piped());

        let mut shell = Shell::default();
        shell.set_current_dir(path.as_ref().to_str().unwrap());
        for dep in dependencies {
            shell.add_command(&format!("module load {dep}"));
        }

        shell.add_command(&format!(
            "cmake --build . --config {:?} --parallel {:?}",
            self.build_type, self.jobs
        ));

        // let spawn = cmake.spawn().map_err(|e| e.to_string())?;
        // let (result, stdout, stderr) = child_logger(spawn);

        let (result, stdout, stderr) = shell.exec();

        let result = result.map_err(|_| "Failed to run CMake command")?;

        if !result.success() {
            return Err(format!(
                "Failed to execute command. Output:\n{}\n{}",
                stdout.join("\n"),
                stderr.join("\n")
            ));
        }

        Ok(())
    }
}

impl BuilderImpl for CMake {
    fn from_py(object: &Bound<PyAny>) -> Result<Self, String> {
        let build_type = match object
            .getattr("build_type")
            .map_err(|_| "Failed to read attribute 'build_type' of Builder object")?
            .extract::<String>()
            .map_err(|_| "Failed to convert attribute 'build_type' to Rust String")?
            .to_lowercase()
            .as_str()
        {
            "debug" => CMakeBuildType::Debug,
            "release" => CMakeBuildType::Release,
            "relwithdebinfo" => CMakeBuildType::RelWithDebInfo,
            "minsizerel" => CMakeBuildType::MinSizeRel,
            other => log::error(&format!("Unknown CMake build type {other}")),
        };

        let jobs: usize = object
            .getattr("jobs")
            .map_err(|_| "Failed to read attribute 'jobs' of Builder object")?
            .extract()
            .map_err(|_| "Failed to convert attribute 'jobs' to Rust usize")?;

        let configure_flags: Option<Vec<String>> = object
            .getattr("configure_flags")
            .map_err(|_| "Failed to read attribute 'configure_flags' of Builder object")?
            .extract()
            .map_err(|_| "Failed to convert attribute 'configure_flags' to Rust Vec<String>")?;

        Ok(Self {
            build_type,
            jobs,
            configure_flags,
        })
    }

    fn build<
        P0: AsRef<Path> + std::fmt::Debug,
        P1: AsRef<Path> + std::fmt::Debug,
        P2: AsRef<Path>,
    >(
        &self,
        source_path: &P0,
        build_path: &P1,
        _: &P2,
        dependencies: &[String],
    ) -> Result<(), String> {
        self.configure(source_path, build_path, dependencies)?;
        self.compile(build_path, dependencies)?;
        Ok(())
    }

    fn install<P0: AsRef<Path>, P1: AsRef<Path>>(
        &self,
        build_path: &P0,
        install_path: &P1,
        dependencies: &[String],
    ) -> Result<(), String> {
        let build_path = path::absolute(build_path).map_err(|err| err.to_string())?;
        let install_path = path::absolute(install_path).map_err(|err| err.to_string())?;

        fs::create_dir_all(&install_path).map_err(|e| e.to_string())?;

        if !build_path.exists() {
            return Err(format!("Build directory {build_path:?} does not exist"));
        }

        // let mut cmake = Command::new("cmake");
        // cmake.current_dir(&build_path);
        // cmake.arg("--install");
        // cmake.arg(".");
        // cmake.arg(format!("--config {:?}", self.build_type));
        // cmake.arg(format!(
        //     "--prefix {}",
        //     install_path
        //         .to_str()
        //         .ok_or("Failed to convert path to string")?
        // ));

        let mut shell = Shell::default();
        shell.set_current_dir(build_path.to_str().unwrap());

        for dep in dependencies {
            shell.add_command(&format!("module load {dep}"));
        }

        shell.add_command(&format!("cmake --install . --preifx {install_path:?}"));

        // cmake.stdout(std::process::Stdio::piped());
        // cmake.stderr(std::process::Stdio::piped());
        // let spawn = cmake.spawn().map_err(|e| e.to_string())?;

        // let (result, stdout, stderr) = child_logger(spawn);

        let (result, stdout, stderr) = shell.exec();

        let result = result.map_err(|_| "Failed to run CMake command")?;

        if !result.success() {
            return Err(format!(
                "Failed to execute command. Output:\n{}\n{}",
                stdout.join("\n"),
                stderr.join("\n")
            ));
        }

        Ok(())
    }
}
