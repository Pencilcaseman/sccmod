use std::fs;
use toml::Table;

#[derive(Debug)]
pub struct Config {
    pub module_paths: Vec<String>,
    pub build_root: String,
    pub install_root: String,
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

    let module_paths: Vec<String> = table["module_paths"]
        .as_array()
        .ok_or_else(|| "`module_paths` must be an array of strings".to_string())
        .and_then(|paths| {
            paths
                .iter()
                .map(|item| {
                    item.as_str()
                        .map(std::string::ToString::to_string)
                        .ok_or_else(|| "`module_paths` must be an array of strings".to_string())
                })
                .collect()
        })?;

    let build_root: String = table["build_root"]
        .as_str()
        .ok_or_else(|| "`build_root` must be a string".to_string())?
        .to_string();

    let install_root: String = table["install_root"]
        .as_str()
        .ok_or_else(|| "`install_root` must be a string".to_string())?
        .to_string();

    Ok(Config {
        module_paths,
        build_root,
        install_root,
    })
}
