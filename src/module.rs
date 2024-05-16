use crate::{
    builders::builder_trait::{Builder, BuilderImpl},
    config,
    downloaders::{Downloader, DownloaderImpl},
    file_manager::{recursive_list_dir, PATH_SEP},
    log,
};

use crate::python_interop::{extract_object, load_program};
use pyo3::prelude::*;
use std::collections::HashMap;
use std::fs::DirEntry;
use std::path::Path;

#[derive(Debug)]
pub struct Module {
    /// Unique identifier for the module
    pub identifier: String,

    /// Path to the module file
    pub modulefile: Vec<String>,

    /// Path to download the source code
    pub download_path: String,

    /// Path to build the source code
    pub build_path: String,

    /// Path to install the source coe
    pub install_path: String,

    pub dependencies: Vec<Box<Module>>,
    pub downloader: Downloader,
    pub builder: Builder,
    pub metadata: HashMap<String, String>,
}

impl Module {
    /// Download the source code for the module, based on its [`Downloader`].
    ///
    /// # Errors
    /// This will error if the download fails, with an error [`String`] containing
    /// either an error message or the output of the errored command.
    pub fn download<P: AsRef<Path>>(&self, path: &P) -> Result<(), String> {
        self.downloader.download(path)
    }

    /// Build the source code for this module, based on its [`Builder`].
    ///
    /// # Errors
    /// This will error if the build fails, with an error [`String`] containing
    /// either an error message or the output of the errored command.
    pub fn build<P0: AsRef<Path> + std::fmt::Debug, P1: AsRef<Path> + std::fmt::Debug>(
        &self,
        source_path: &P0,
        output_path: &P1,
    ) -> Result<(), String> {
        self.builder.build(source_path, output_path)
    }

    /// Install the source code for this module based on its [`Builder`].
    ///
    /// # Errors
    /// Errors if the installation fails. The [`Result`] output contains a [`String`]
    /// with either an error message or the output of the errored program.
    pub fn install<P0: AsRef<Path> + std::fmt::Debug, P1: AsRef<Path> + std::fmt::Debug>(
        &self,
        build_path: &P0,
        install_path: &P1,
    ) -> Result<(), String> {
        self.builder.install(build_path, install_path)
    }

    /// Extract a [`Module`] object from a python object.
    ///
    /// # Errors
    /// This method will return [`Err(msg)`] if the object cannot be parsed
    /// successfully. `msg` is a string and contains the error message.
    pub fn from_object<P0: AsRef<Path>>(
        object: &Bound<PyAny>,
        path: &P0,
        config: &config::Config,
    ) -> Result<Self, String> {
        Python::with_gil(|_| {
            let metadata: HashMap<String, String> = extract_object(object, "metadata")?
                .call0()
                .map_err(|err| format!("Failed to call `metadata`: {err}"))?
                .extract()
                .map_err(|err| {
                    format!("Failed to convert metadata output to Rust HashMap: {err}")
                })?;

            let downloader = Downloader::from_py(
                &extract_object(object, "download")?
                    .call0()
                    .map_err(|err| format!("Failed to call `download` in module class: {err}"))?,
            )
            .ok_or_else(|| "Could not extract downloader from module class".to_string())?;

            // todo: build requirements

            let builder = Builder::from_py(
                &extract_object(object, "build")?
                    .call0()
                    .map_err(|err| format!("Failed to call `build` in module class: {err}"))?,
            )
            .ok_or_else(|| "Could not extract builder from module class".to_string())?;

            // Extract modulefile from the path
            let modulefile: Vec<String> = path
                .as_ref()
                .to_str()
                .ok_or("Failed to convert filename to string")?
                .split(PATH_SEP)
                .map(std::string::ToString::to_string)
                .collect();

            let identifier = metadata
                .get("identifier")
                .ok_or("Metadata does not contain 'identifier' tag")?
                .replace([' ', '\t'], "_");

            // Download path is ${root}/(${download_path} or ${identifier})
            let download_path = format!(
                "{}{}{}",
                config.build_root,
                PATH_SEP,
                metadata
                    .get("download_path")
                    .map_or(&identifier, |path| path)
            );

            let build_path = format!(
                "{}{}{}",
                config.build_root,
                PATH_SEP,
                metadata.get("build_path").map_or(&identifier, |path| path)
            );

            let install_path = format!("{}{}{}", config.install_root, PATH_SEP, identifier);

            Ok(Self {
                identifier,
                modulefile,
                download_path,
                build_path,
                install_path,
                dependencies: vec![],
                downloader,
                builder,
                metadata,
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
            .module_paths
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
                        .map(|module| Module::from_object(module, &path.path(), &config))
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
    log::status(&format!("Downloading '{}'", module.identifier));
    module.download(&module.download_path)
}

/// Download and build a module.
///
/// # Errors
/// Errors if [`Module.download`] fails or [`Module.build`] fails.
pub fn build(module: &Module) -> Result<(), String> {
    download(module)?;

    log::status(&format!("Building '{}'", module.identifier));
    module.build(&module.download_path, &module.install_path)
}

/// Download, build and install a module.
///
/// # Errors
/// Errors if [`Module.download`], [`Module.build`] or [`Module.install`] fails.
pub fn install(module: &Module) -> Result<(), String> {
    build(module)?;

    log::status(&format!("Installing '{}'", module.identifier));
    module.install(&module.build_path, &module.install_path)
}
