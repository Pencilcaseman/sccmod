use crate::{
    builders::builder_trait::{Builder, BuilderImpl},
    config,
    downloaders::{Downloader, DownloaderImpl},
    file_manager::{recursive_list_dir, PATH_SEP},
    flavours, log, modulefile,
    shell::Shell,
};

use crate::python_interop::{extract_object, load_program};
use pyo3::prelude::*;
use std::collections::HashMap;
use std::fs::DirEntry;

#[derive(Debug, Clone)]
pub enum Dependency {
    Class(String),
    Module(String),
}

#[derive(Debug, Clone)]
pub enum Environment {
    Set(String),
    Append(String),
    Prepend(String),
}

#[derive(Debug, Clone)]
pub struct Module {
    /// Name of the module
    pub name: String,

    /// Module version
    pub version: String,

    /// Module class (flavours)
    pub class: String,

    /// Module dependencies
    pub dependencies: Vec<Dependency>,

    /// Module metadata
    pub metadata: HashMap<String, String>,

    /// Environment variables to set/change
    pub environment: HashMap<String, Environment>,

    /// Path root (${root}/${name}/${version})
    pub root: String,

    /// A list of commands to run before building
    pub pre_build: Option<Vec<String>>,

    /// Downloader to download the source code
    pub downloader: Downloader,

    /// Builder to build and install the source code
    pub builder: Builder,

    pub source_path: String,
    pub build_path: String,
    pub install_path: String,
}

impl Module {
    /// Parse a flavour into:
    ///  - flavour_str: a postfix to a path pointing to a flavour directory
    ///  - build_path: updated build path
    ///  - install_path: updated install path
    ///  - modules: module names necessary for installation
    pub fn parse(&self, flavour: &(&[Module], usize)) -> (String, String, String, Vec<String>) {
        // Generate extension to build path based on flavour
        let mut flavour_str = format!("{PATH_SEP}1{PATH_SEP}"); // '/1/' for revision

        // If no class modules are required, install into `default` flavour
        if flavour.1 == 0 {
            flavour_str.push_str(&format!("default"))
        } else {
            for (i, flav) in (0..flavour.1).zip(flavour.0.iter()) {
                flavour_str.push_str(&format!("{}-{}", &flav.name, &flav.version));

                if i + 1 < flavour.1 {
                    flavour_str.push('-');
                }
            }
        }

        let build_path = self.build_path.clone() + &flavour_str;
        let install_path = self.install_path.clone() + &flavour_str;

        let modules: Vec<String> = flavour
            .0
            .iter()
            .map(|flav| format!("{}/{}/{}", flav.root, flav.name, flav.version))
            .collect();

        (flavour_str, build_path, install_path, modules)
    }

    pub fn identifier(&self) -> String {
        format!("{}/{}/{}", self.root, self.name, self.version)
    }

    /// Download the source code for the module, based on its [`Downloader`].
    ///
    /// # Errors
    /// This will error if the download fails, with an error [`String`] containing
    /// either an error message or the output of the errored command.
    pub fn download(&self) -> Result<(), String> {
        self.downloader.download(&self.source_path)
    }

    /// Build the source code for this module, based on its [`Builder`].
    ///
    /// # Errors
    /// This will error if the build fails, with an error [`String`] containing
    /// either an error message or the output of the errored command.
    pub fn build(
        &self,
        flavour: (&[Module], usize), // ([dep0, dep1, ..., depN], num_flavour)
    ) -> Result<(), String> {
        if let Some(commands) = &self.pre_build {
            log::status(&"Running pre-build commands");
            let mut shell = Shell::default();
            shell.set_current_dir(&self.source_path);
            for cmd in commands {
                shell.add_command(&cmd);
            }

            let (result, stdout, stderr) = shell.exec();

            let result = result.map_err(|_| "Failed to run CMake command")?;

            if !result.success() {
                return Err(format!(
                    "Failed to execute command. Output:\n{}\n{}",
                    stdout.join("\n"),
                    stderr.join("\n")
                ));
            }

            log::status(&"Building...");
        }

        let (_, build_path, install_path, modules) = self.parse(&flavour);

        self.builder
            .build(&self.source_path, &build_path, &install_path, &modules)
    }

    /// Install the source code for this module based on its [`Builder`].
    ///
    /// # Errors
    /// Errors if the installation fails. The [`Result`] output contains a [`String`]
    /// with either an error message or the output of the errored program.
    pub fn install(&self, flavour: (&[Module], usize)) -> Result<(), String> {
        let (_, build_path, install_path, modules) = self.parse(&flavour);

        self.builder
            .install(&self.source_path, &build_path, &install_path, &modules)
    }

    /// Extract a [`Module`] object from a python object.
    ///
    /// # Errors
    /// This method will return [`Err(msg)`] if the object cannot be parsed
    /// successfully. `msg` is a string and contains the error message.
    pub fn from_object(object: &Bound<PyAny>, config: &config::Config) -> Result<Self, String> {
        Python::with_gil(|_| {
            let metadata: HashMap<String, String> = extract_object(object, "metadata")?
                .call0()
                .map_err(|err| format!("Failed to call `metadata`: {err}"))?
                .extract()
                .map_err(|err| {
                    format!("Failed to convert metadata output to Rust HashMap: {err}")
                })?;

            let name = metadata
                .get("name")
                .ok_or("metadata does not contain key 'name'")?
                .to_owned();

            let version = metadata
                .get("version")
                .ok_or("Metadata does not contain key 'version'")?
                .to_owned();

            let class = metadata
                .get("class")
                .ok_or("Metadata does not contain key 'class'")?
                .to_owned();

            let downloader = Downloader::from_py(
                &extract_object(object, "download")?
                    .call0()
                    .map_err(|err| format!("Failed to call `download` in module class: {err}"))?,
            )?;

            let dependencies: Vec<&PyAny> = extract_object(object, "dependencies")?
                .call0()
                .map_err(|err| format!("Failed to call `build_requirements`: {err}"))?
                .extract()
                .map_err(|err| format!("Failed to convert `dependencies()` to Rust Vec: {err}"))?;

            // Convert dependencies into a Rust vector
            let dependencies: Vec<Dependency> = dependencies.iter().map(|dep| {
                match dep.get_type().to_string().as_ref() {
                    "<class 'sccmod.module.Class'>" => {
                        match dep.getattr("name").map_err(|err| format!("Dependency is a Class instance, but does not contain a .name attribute: {err}"))?.extract::<String>() {
                            Ok(name) => {
                                Ok(Dependency::Class(name))
                            },
                            Err(e) => Err(format!("Could not convert .name attribute to Rust String: {e}"))
                        }
                    },
                    _ => Ok(Dependency::Module(dep.to_string())),
                }
            }).collect::<Result<Vec<Dependency>, String>>()?;

            let environment: HashMap<String, (String, String)> = extract_object(
                object,
                "environment",
            )?
            .call0()
            .map_err(|err| format!("Failed to call '.environment()': {err}"))?
            .extract()
            .map_err(|err| {
                format!("Failed to convert output of `.environment()` to Rust HashMap<String, (String, String)>: {err}")
            })?;

            // Convert (String, String) to Environment(String)
            let environment = environment
                .into_iter()
                .map(|(name, (op, value))| match op.as_ref() {
                    "set" => Ok((name, Environment::Set(value))),
                    "append" => Ok((name, Environment::Append(value))),
                    "prepend" => Ok((name, Environment::Prepend(value))),
                    other => Err(format!("Invalid environment variable operation '{other}'")),
                })
                .collect::<Result<HashMap<String, Environment>, String>>()?;

            let builder = Builder::from_py(
                &extract_object(object, "build")?
                    .call0()
                    .map_err(|err| format!("Failed to call `build` in module class: {err}"))?,
            )?;

            let root = metadata
                .get("root")
                .ok_or("Metadata does not contain key 'root'")?
                .to_owned();

            let pre_build: Option<Vec<String>> = match extract_object(object, "pre_build") {
                Ok(obj) => Some(
                    obj.call0()
                        .map_err(|err| {
                            format!("Failed to call 'pre_build()` in module class: {err}")
                        })?
                        .extract()
                        .map_err(|err| {
                            format!("Failed to convert object to Rust Vec<String>: {err}")
                        })?,
                ),
                Err(_) => None,
            };

            let source_path = format!(
                "{}{PATH_SEP}{}{PATH_SEP}{}",
                config.build_root, name, version
            );

            let build_path = format!("{source_path}/sccmod_build");

            let install_path = format!(
                "{1:}{0:}{2:}{0:}{3:}",
                PATH_SEP, config.install_root, root, name
            );

            Ok(Self {
                name,
                version,
                class,
                dependencies,
                environment,
                metadata,
                root,
                pre_build,
                downloader,
                builder,
                source_path,
                build_path,
                install_path,
            })
        })
    }
}

/// List all available modules
///
/// # Errors
/// Will error if:
///  - The configuration file cannot be read (see [`Config`])
///  - Any specified directory cannot be read (see [`recursive_list_dir`])
pub fn get_modules() -> Result<Vec<Module>, String> {
    config::read().and_then(|config| {
        config // Extract module paths
            .sccmod_module_paths
            .iter()
            .flat_map(|path| {
                // Expand search paths recursively to get *all* files
                recursive_list_dir(path).map_or_else(
                    || vec![Err("Failed to extract paths".to_string())],
                    |paths| {
                        // Map path -> Ok(path)
                        paths.into_iter().map(Ok).collect()
                    },
                )
            })
            .collect::<Result<Vec<DirEntry>, _>>()? // Collect and propagate Result
            .iter()
            .map(|path| {
                // Extract modules from files
                Python::with_gil(|py| {
                    let program = load_program(&py, &path.path())?;
                    let modules: Vec<_> = program
                        .getattr("generate")
                        .map_err(|err| format!("Failed to load generator: {err}"))?
                        .call0()
                        .map_err(|err| format!("Failed to call generator: {err}"))?
                        .extract()
                        .map_err(|err| {
                            format!("Failed to convert output of `generate` to Vec: {err}")
                        })?;

                    modules // Map python objects to Modules
                        .iter()
                        .map(|module| Module::from_object(module, &config))
                        .collect::<Result<Vec<Module>, String>>()
                })
            })
            .flat_map(|v| {
                // Flat map vectors to extract errors
                v.map_or_else(
                    |err| vec![Err(format!("Something went wrong: {err}"))],
                    |vec| vec.into_iter().map(Ok).collect(),
                )
            })
            .collect::<Result<Vec<_>, _>>() // Collect as result
    })
}

/// Download a module.
///
/// # Errors
/// Errors if [`Module.download`] fails.
pub fn download(module: &Module) -> Result<(), String> {
    log::status(&format!("Downloading '{}-{}'", module.name, module.version));
    module.download()
}

/// Download and build a module.
///
/// # Errors
/// Errors if [`Module.download`] fails or [`Module.build`] fails.
pub fn build(module: &Module) -> Result<(), String> {
    download(module)?;

    log::status(&format!("Building '{}-{}'", module.name, module.version));

    let flavs = flavours::generate(module)?;

    for flav in &flavs {
        module.build((&flav.0, flav.1))?;
    }

    Ok(())
}

/// Download, build and install a module.
///
/// # Errors
/// Errors if [`Module.download`], [`Module.build`] or [`Module.install`] fails.
pub fn install(module: &Module) -> Result<(), String> {
    build(module)?;

    log::status(&format!("Installing '{}-{}'", module.name, module.version));

    let flavs = flavours::generate(module)?;

    for flav in &flavs {
        module.install((&flav.0, flav.1))?;
    }

    // Write modulefile
    log::status("Writing Modulefile");
    let conf = config::read()?;
    let dir = format!(
        "{}{PATH_SEP}{}{PATH_SEP}{}{PATH_SEP}{}",
        conf.modulefile_root, module.root, module.name, module.version
    );
    let dir = std::path::Path::new(&dir);

    let content = modulefile::generate(module);

    std::fs::create_dir_all(dir.parent().unwrap()).unwrap();
    std::fs::write(dir, content).map_err(|err| format!("Failed to write modulefile: {err}"))
}
