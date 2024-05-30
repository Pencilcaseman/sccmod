use crate::{
    config, log,
    module::{self, get_modules, Module},
    module_resolver,
};
use colored::Colorize;
use std::io::Write;

/// Internal boilerplate handler which, given a set of partials and a function,
/// finds the specified module and passes it to the function.
///
/// # Errors
/// Errors if the module selection process fails or if the provided function
/// return an error.
pub fn resolver_boilerplate(
    partials: &[&str],
    func: fn(&Module) -> Result<(), String>,
) -> Result<(), String> {
    match module_resolver::resolve(partials)? {
        module_resolver::ResolveMatch::Full(m) => func(&m),
        module_resolver::ResolveMatch::Partial(m) => {
            let mut err = String::from("Multiple modules match the provided partial(s):\n");

            // Always valid, as `m.len()` >= 1, so `log(m.len())` >= 0
            #[allow(
                clippy::cast_possible_truncation,
                clippy::cast_sign_loss,
                clippy::cast_precision_loss
            )]
            let max_digits = (m.len() as f64).log10() as usize;

            err.push_str(&format!(
                "{}     {}\n",
                String::from(" ").repeat(max_digits),
                "all".bold().magenta()
            ));

            for (index, item) in m.iter().enumerate() {
                // Always valid, as `item` >= 0, so `log(item + 1)` >= 0
                #[allow(
                    clippy::cast_possible_truncation,
                    clippy::cast_sign_loss,
                    clippy::cast_precision_loss
                )]
                let digits = (index as f64 + 0.05).log10() as usize;

                let mut index_str = String::from(" ").repeat(max_digits - digits);
                index_str.push_str(&format!("{index}"));

                err.push_str(&format!(
                    "  {index_str}: {}\n",
                    item.identifier().bold().cyan()
                ));
            }

            log::warn(&err);

            let mut valid = false;
            let mut selection = String::new();
            let mut selection_index = 0;

            while !valid {
                print!("{}", "Please enter a selection: ".yellow().bold());
                std::io::stdout().flush().map_err(|e| e.to_string())?;

                match std::io::stdin().read_line(&mut selection) {
                    Ok(_) if selection.trim() == "all" => valid = true,
                    Ok(_) => {
                        match selection.trim().parse::<usize>() {
                            Ok(num) if num < m.len() => {
                                valid = true;
                                selection_index = num;
                            }
                            Ok(_) => {
                                log::warn("Invalid index selected");
                            }
                            Err(_) => {
                                log::warn("Invalid input received. Input must be a positive integer or 'all'");
                            }
                        }
                    }
                    Err(_) => {
                        log::warn("Failed to read input");
                    }
                }

                selection.clear(); // Clear the input buffer for the next iteration
            }

            func(&m[selection_index])
        }
        module_resolver::ResolveMatch::None => {
            log::error("No modules match the partials provided");
        }
    }
}

pub fn info(config: &config::Config) -> Result<(), String> {
    fn fmt<T: std::fmt::Debug>(name: &str, value: &T) {
        // println!("{}", format!("{name} {value:?}").bold().purple());
        println!("{} {}", name.bold().purple(), format!("{value:?}").cyan());
    }

    fmt("config file . . . . :", &std::env::var("SCCMOD_CONFIG"));
    fmt("sccmod_module_paths :", &config.sccmod_module_paths);
    fmt("modulefile root . . :", &config.modulefile_root);
    fmt("build_root  . . . . :", &config.build_root);
    fmt("install_root  . . . :", &config.install_root);
    fmt("shell . . . . . . . :", &config.shell);

    Ok(())
}

/// A callback function to list all available modules
///
/// # Errors
///
/// This function will error if an invalid modulefile is found.
pub fn list_callback(_config: &config::Config) -> Result<(), String> {
    println!("{}", "Available Modules:".bold().purple());

    for p in &get_modules()? {
        println!(" > {}", p.identifier().bold().cyan());
    }

    Ok(())
}

/// A callback function to download a module from its name.
///
/// # Errors
///
/// Will error if a single module cannot be resolved from the specified name,
/// or if the call to [`Module.download`] fails.
pub fn download_module(partials: &[&str], _config: &config::Config) -> Result<(), String> {
    resolver_boilerplate(partials, module::download)
}

/// A callback function to download all available modules
///
/// # Errors
///
/// Errors if the modules cannot be listed or if any module fails to download
pub fn download_all(_config: &config::Config) -> Result<(), String> {
    for m in &get_modules()? {
        module::download(m)?;
    }

    Ok(())
}

/// A callback function to build a module based on its name.
///
/// # Errors
///
/// Errors if a single module cannot be resolved from the specified name,
/// or if the call to [`Module.build`] fails.
pub fn build_module(partials: &[&str], _config: &config::Config) -> Result<(), String> {
    resolver_boilerplate(partials, module::build)
}

/// A callback function to build all available modules
///
/// # Errors
///
/// Errors if the modules cannot be listed or if any module fails to build
pub fn build_all(_config: &config::Config) -> Result<(), String> {
    for m in &get_modules()? {
        module::build(m)?;
    }

    Ok(())
}

/// A callback function to install a module from its name.
///
/// # Errors
///
/// Returns [`Err(string)`] if a single module cannot be resolved from the
/// specified name, or if the call to [`Module.install`] fails.
pub fn install_module(partials: &[&str], _config: &config::Config) -> Result<(), String> {
    resolver_boilerplate(partials, module::install)
}

pub fn write_modulefile(partials: &[&str], _config: &config::Config) -> Result<(), String> {
    resolver_boilerplate(partials, module::modulefile)
}

/// A callback function to install all available modules
///
/// # Errors
///
/// Errors if the modules cannot be listed or if any module fails to install
pub fn install_all(_config: &config::Config) -> Result<(), String> {
    for m in &get_modules()? {
        module::install(m)?;
    }

    Ok(())
}

pub fn write_modulefile_all(_config: &config::Config) -> Result<(), String> {
    for m in &get_modules()? {
        module::modulefile(m)?;
    }

    Ok(())
}
