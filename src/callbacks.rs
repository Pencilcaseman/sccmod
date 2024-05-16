use crate::{config, log, module::get_modules};
use colored::Colorize;

/// A callback function to list all available modules
///
/// # Errors
///
/// This function will error if an invalid modulefile is found.
pub fn list_callback(_config: &config::Config) -> Result<(), String> {
    println!("{}", "Available Modules:".bold().purple());

    for p in &get_modules()? {
        println!(" > {}", p.identifier.bold().cyan());
    }

    Ok(())
}

/// A callback function to download a module from its name.
///
/// # Errors
///
/// Will error if a single module cannot be resolved from the specified name,
/// or if the call to [`Module.download`] fails.
pub fn download_module(name: &str, _config: &config::Config) -> Result<(), String> {
    for module in &get_modules()? {
        if name == module.identifier {
            log::status(&format!("Downloading '{}'", module.identifier));
            module.download(&module.download_path)?;
        }
    }

    Ok(())
}

/// A callback function to build a module based on its name.
///
/// # Errors
///
/// Errors if a single module cannot be resolved from the specified name,
/// or if the call to [`Module.build`] fails.
pub fn build_module(name: &str, _config: &config::Config) -> Result<(), String> {
    for module in &get_modules()? {
        if name == module.identifier {
            log::status(&format!("Downloading '{}'", module.identifier));
            module.download(&module.download_path)?;

            log::status(&format!("Building '{}'", module.identifier));
            module.build(&module.download_path, &module.install_path)?;
        }
    }

    Ok(())
}

/// A callback function to install a module from its name.
///
/// # Errors
///
/// Returns [`Err(string)`] if a single module cannot be resolved from the
/// specified name, or if the call to [`Module.install`] fails.
pub fn install_module(name: &str, _config: &config::Config) -> Result<(), String> {
    for module in &get_modules()? {
        if name == module.identifier {
            log::status(&format!("Downloading '{}'", module.identifier));
            module.download(&module.download_path)?;

            log::status(&format!("Building '{}'", module.identifier));
            module.build(&module.download_path, &module.build_path)?;

            log::status(&format!("Installing '{}'", module.identifier));
            module.install(&module.build_path, &module.install_path)?;
        }
    }

    Ok(())
}
