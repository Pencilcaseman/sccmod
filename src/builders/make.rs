use crate::{builders::builder_trait::BuilderImpl, cli::child_logger, log, shell::Shell};
use pyo3::prelude::PyAnyMethods;
use pyo3::{Bound, PyAny};
use std::{fs, path, path::Path, process::Command};

#[derive(Debug)]
pub struct Make {
    pub configure: bool,
    pub jobs: usize,
    pub configure_flags: Option<Vec<String>>,
}

impl Make {
    fn configure<
        P0: AsRef<Path> + std::fmt::Debug,
        P1: AsRef<Path> + std::fmt::Debug,
        P2: AsRef<Path>,
    >(
        &self,
        source_path: &P0,
        _: &P1,
        install_path: &P2,
        dependencies: &[String],
    ) -> Result<(), String> {
        log::status("Configuring");

        let source_path = path::absolute(source_path).map_err(|err| err.to_string())?;
        let install_path = path::absolute(install_path).map_err(|err| err.to_string())?;

        // let mut configure = Command::new("./configure");
        // configure.current_dir(&source_path);

        // if let Some(flags) = &self.configure_flags {
        //     for flag in flags {
        //         configure.arg(flag);
        //     }
        // }

        // // Set prefix to the install directory
        // configure.arg(format!(
        //     "--prefix={}",
        //     install_path
        //         .to_str()
        //         .ok_or("Failed to convert install path to string")?
        // ));

        let mut shell = Shell::default();
        shell.set_current_dir(source_path.to_str().unwrap());

        for dep in dependencies {
            shell.add_command(&format!("module load {dep}"));
        }

        shell.add_command(&format!("./configure --prefix={install_path:?}"));

        let mut make_cmd = format!("cmake {source_path:?}");

        if let Some(flags) = &self.configure_flags {
            for flag in flags {
                make_cmd.push_str(&format!(" {flag}"));
            }
        }

        shell.add_command(&make_cmd);

        // configure.stdout(std::process::Stdio::piped());
        // configure.stderr(std::process::Stdio::piped());

        // let spawn = configure.spawn().map_err(|e| e.to_string())?;
        // let (result, stdout, stderr) = child_logger(spawn);

        // if result.is_err() {
        //     return Err("Failed to run ./configure".to_string());
        // }
        // let result = result.unwrap();

        let (result, stdout, stderr) = shell.exec();
        let result = result.map_err(|_| "Failed to run ./configure")?;

        if !result.success() {
            return Err(format!(
                "Failed to configure. Output:\n{}\n{}",
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
        log::status("Running make");

        // let mut make = Command::new("make");
        // make.current_dir(path);
        // make.arg("-j");
        // make.arg(format!("{}", self.jobs));
        // make.stdout(std::process::Stdio::piped());
        // make.stderr(std::process::Stdio::piped());

        // println!("Command: {make:?}");

        // let spawn = make.spawn().map_err(|e| e.to_string())?;
        // let (result, stdout, stderr) = child_logger(spawn);

        let mut shell = Shell::default();

        for dep in dependencies {
            shell.add_command(&format!("module load {dep}"));
        }

        shell.set_current_dir(path.as_ref().to_str().unwrap());
        shell.add_command(&format!("make -j {}", self.jobs));

        let (result, stdout, stderr) = shell.exec();
        let result = result.map_err(|_| "Failed to run make")?;

        // if result.is_err() {
        //     return Err("Failed to run make".to_string());
        // }
        // let result = result.unwrap();

        if !result.success() {
            return Err(format!(
                "Failed to execute make. Output:\n{}\n{}",
                stdout.join("\n"),
                stderr.join("\n")
            ));
        }

        Ok(())
    }
}

impl BuilderImpl for Make {
    fn from_py(object: &Bound<PyAny>) -> Result<Self, String> {
        let configure: bool = object
            .getattr("configure")
            .map_err(|_| "Failed to read attribute 'configure' of Builder object")?
            .extract()
            .map_err(|_| "Failed to convert attribute 'configure' to Rust bool")?;

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
            configure,
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
        install_path: &P2,
        dependencies: &[String],
    ) -> Result<(), String> {
        self.configure(source_path, build_path, install_path, dependencies)?;
        self.compile(source_path, dependencies)?;
        Ok(())
    }

    fn install<P0: AsRef<Path>, P1: AsRef<Path>>(
        &self,
        build_path: &P0,
        install_path: &P1,
        _: &[String], // Dependencies are not necessary for installing
    ) -> Result<(), String> {
        let source_path = path::absolute(build_path).map_err(|err| err.to_string())?;
        let install_path = path::absolute(install_path).map_err(|err| err.to_string())?;

        fs::create_dir_all(install_path).map_err(|e| e.to_string())?;

        if !source_path.exists() {
            return Err(format!("Source directory {source_path:?} does not exist"));
        }

        let mut make = Command::new("make");
        make.current_dir(build_path);
        make.arg("install");

        make.stdout(std::process::Stdio::piped());
        make.stderr(std::process::Stdio::piped());
        let spawn = make.spawn().map_err(|e| e.to_string())?;

        let (result, stdout, stderr) = child_logger(spawn);

        if result.is_err() {
            return Err("Failed to run make install".to_string());
        }
        let result = result.unwrap();

        if !result.success() {
            return Err(format!(
                "Failed to execute {make:?}. Output:\n{}\n{}",
                stdout.join("\n"),
                stderr.join("\n")
            ));
        }

        Ok(())
    }
}
