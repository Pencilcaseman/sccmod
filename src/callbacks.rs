use crate::{config, log, module::get_modules};
use colored::*;

pub fn list_callback(_config: &config::Config) -> Result<(), String> {
    println!("{}", "Available Modules:".bold().purple());

    for p in &get_modules()? {
        println!("{} {}", " >", p.identifier.bold().cyan());
    }

    Ok(())
}

pub fn download_module(name: &str, _config: &config::Config) -> Result<(), String> {
    for module in &get_modules()? {
        if name == module.identifier {
            log::status(&format!("Downloading '{}'", module.identifier));
            module.download(&module.download_path)?;
        }
    }

    Ok(())
}

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
