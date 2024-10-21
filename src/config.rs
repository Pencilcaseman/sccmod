use std::fs;

use toml::Table;

use crate::module::Module;

#[derive(Debug)]
pub struct Config {
    pub sccmod_module_paths: Vec<String>,
    pub modulefile_root: String,
    pub build_root: String,
    pub install_root: String,
    pub shell: String,
    pub class_no_conflict: Vec<String>,
    pub num_threads: usize,
}

/// Read the sccmod configuration toml file and return the result.
///
/// # Errors
/// Returns `Err(message)` if:
///  - The environment variable [`SCCMOD_CONFIG`] cannot be read
///  - The file pointed to by [`SCCMOD_CONFIG`] does not exist or cannot be read
///  - [`module_paths`] is not an array of strings
///  - [`build_root`] is not a string
///  - [`install_root`] is not a string
pub fn read() -> Result<Config, String> {
    // Read config file
    let config_path = std::env::var("SCCMOD_CONFIG").map_err(|_| {
        "The SCCMOD_CONFIG environment variable could not be found. Ensure it is set"
    })?;

    let config = fs::read_to_string(&config_path).map_err(|_| {
        format!(
            "Could not read the file at $SCCMOD_CONFIG='{config_path}' \
-- ensure SCCMOD_CONFIG is correct and that the file exists"
        )
    })?;

    let table = config.parse::<Table>().map_err(|err| err.to_string())?;

    let sccmod_module_paths: Vec<String> = table["sccmod_module_paths"]
        .as_array()
        .ok_or_else(|| "`module_paths` must be an array of strings".to_string())
        .and_then(|paths| {
            paths
                .iter()
                .map(|item| {
                    item.as_str()
                        .map(std::string::ToString::to_string)
                        .ok_or_else(|| {
                            "`module_paths` must be an array of strings"
                                .to_string()
                        })
                })
                .collect()
        })?;

    let modulefile_root: String = table["modulefile_root"]
        .as_str()
        .ok_or_else(|| "`modulefile_root` must be a string".to_string())?
        .to_string();

    let build_root: String = table["build_root"]
        .as_str()
        .ok_or_else(|| "`build_root` must be a string".to_string())?
        .to_string();

    let install_root: String = table["install_root"]
        .as_str()
        .ok_or_else(|| "`install_root` must be a string".to_string())?
        .to_string();

    let shell: String = table["shell"]
        .as_str()
        .ok_or_else(|| "`install_root` must be a string".to_string())?
        .to_string();

    let class_no_conflict: Vec<String> = table["class_no_conflict"]
        .as_array()
        .ok_or_else(|| {
            "`class_no_conflict` must be an array of strings".to_string()
        })?
        .iter()
        .map(|item| {
            item.as_str().map(std::string::ToString::to_string).ok_or_else(
                || {
                    "`class_no_conflict` must be an array of strings"
                        .to_string()
                },
            )
        })
        .collect::<Result<Vec<String>, String>>()?;

    let num_threads: usize = table["num_threads"]
        .as_integer()
        .ok_or_else(|| "`num_threads` must be an integer".to_string())?
        .try_into()
        .map_err(|_| "`num_threads` must be a positive integer".to_string())?;

    // .or(Some(64i64))
    // .ok_or_else(|| "`num_threads` must be an integer".to_string())?
    // .try_into()
    // .map_err(|_| "`num_threads` must be a positive integer".to_string())?;

    Ok(Config {
        sccmod_module_paths,
        modulefile_root,
        build_root,
        install_root,
        shell,
        class_no_conflict,
        num_threads,
    })
}
