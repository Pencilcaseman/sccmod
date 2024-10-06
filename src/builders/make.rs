use std::{fs, path, path::Path, process::Command};

use pyo3::{prelude::PyAnyMethods, Bound, PyAny};

use crate::{
    builders::builder_trait::BuilderImpl, cli::child_logger, log, shell::Shell,
};

#[derive(Debug, Clone)]
pub struct Make {
    pub configure: bool,
    pub jobs: usize,
    pub prefix_args: Option<Vec<String>>,
    pub configure_flags: Option<Vec<String>>,
    pub make_root: Option<String>,
}

impl Make {
    fn configure<
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
        log::status("Configuring");

        fs::create_dir_all(build_path).map_err(|e| e.to_string())?;

        let source_path =
            path::absolute(source_path).map_err(|err| err.to_string())?;
        let build_path =
            path::absolute(build_path).map_err(|err| err.to_string())?;
        let install_path =
            path::absolute(install_path).map_err(|err| err.to_string())?;

        let mut shell = Shell::default();
        shell.set_current_dir(&build_path);

        println!("Module Source: {source_path:?}");
        println!("{dependencies:?}");

        for dep in dependencies {
            log::info(&format!("Loading module: {dep}"));
            shell.add_command(&format!("module load {dep}"));
        }

        // let mut configure_cmd = format!("{source_path:?}/configure");

        // Apply prefix args
        let mut configure_cmd = String::new();
        if let Some(args) = &self.prefix_args {
            for arg in args {
                configure_cmd.push_str(&format!("{arg} "));
            }
        }
        configure_cmd
            .push_str(&format!("{}/configure", source_path.to_str().unwrap()));

        if let Some(flags) = &self.configure_flags {
            for flag in flags {
                configure_cmd.push_str(&format!(" {flag}"));
            }
        }

        // Add this last so it overrides anything passed in `configure_flags`
        configure_cmd.push_str(&format!(" --prefix={install_path:?}"));

        shell.add_command(&configure_cmd);

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

        let mut shell = Shell::default();

        for dep in dependencies {
            shell.add_command(&format!("module load {dep}"));
        }

        shell.set_current_dir(&path.as_ref().to_str().unwrap());
        shell.add_command(&format!("make -j {}", self.jobs));

        let (result, stdout, stderr) = shell.exec();
        let result = result.map_err(|_| "Failed to run make")?;

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
            .map_err(|_| {
                "Failed to read attribute 'configure' of Builder object"
            })?
            .extract()
            .map_err(|_| {
                "Failed to convert attribute 'configure' to Rust bool"
            })?;

        let jobs: usize = object
            .getattr("jobs")
            .map_err(|_| "Failed to read attribute 'jobs' of Builder object")?
            .extract()
            .map_err(|_| "Failed to convert attribute 'jobs' to Rust usize")?;

        let prefix_args: Option<Vec<String>> = object
            .getattr("prefix_args")
            .map_err(|_| {
                "Failed to read attribute 'prefix_args' of Builder object"
            })?
            .extract()
            .map_err(|_| {
                "Failed to convert attribute 'prefix_args' to Rust Vec<String>"
            })?;

        let configure_flags: Option<Vec<String>> = object
            .getattr("configure_flags")
            .map_err(|_| "Failed to read attribute 'configure_flags' of Builder object")?
            .extract()
            .map_err(|_| "Failed to convert attribute 'configure_flags' to Rust Vec<String>")?;

        let make_root: Option<String> = object
            .getattr("make_root")
            .map_err(|_| {
                "Failed to read attribute 'make_root' of Builder object"
            })?
            .extract()
            .map_err(|_| {
                "Failed to convert attribute 'make_root' to Rust String"
            })?;

        Ok(Self { configure, jobs, prefix_args, configure_flags, make_root })
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
        let make_source_path = if let Some(root) = &self.make_root {
            source_path.as_ref().to_str().unwrap().to_owned()
                + PATH_SEP.to_string().as_ref()
                + root
        } else {
            source_path.as_ref().to_str().unwrap().to_owned()
        };

        self.configure(
            &make_source_path,
            build_path,
            install_path,
            dependencies,
        )?;
        self.compile(build_path, dependencies)?;
        Ok(())
    }

    fn install<P0: AsRef<Path>, P1: AsRef<Path>, P2: AsRef<Path>>(
        &self,
        _: &P0,
        build_path: &P1, // Build path is the source path
        install_path: &P2,
        dependencies: &[String],
    ) -> Result<(), String> {
        let build_path =
            path::absolute(build_path).map_err(|err| err.to_string())?;
        let install_path =
            path::absolute(install_path).map_err(|err| err.to_string())?;

        fs::create_dir_all(install_path).map_err(|e| e.to_string())?;

        if !build_path.exists() {
            return Err(format!(
                "Source directory {build_path:?} does not exist"
            ));
        }

        let mut shell = Shell::default();
        shell.set_current_dir(&build_path.to_str().unwrap());

        for dep in dependencies {
            shell.add_command(&format!("module load {dep}"));
        }

        shell.add_command("make install");

        let (result, stdout, stderr) = shell.exec();

        if result.is_err() {
            return Err("Failed to run make install".to_string());
        }
        let result = result.unwrap();

        if !result.success() {
            return Err(format!(
                "Failed to execute make install. Output:\n{}\n{}",
                stdout.join("\n"),
                stderr.join("\n")
            ));
        }

        Ok(())
    }
}
