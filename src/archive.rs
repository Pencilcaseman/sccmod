use std::{path::Path, process::Command};

/// Extract an archive file
///
/// # Errors
///
/// Errors if the directory cannot be created or the file extraction command
/// fails
pub fn extract<P: AsRef<Path>>(
    path: &P,
    name: &str,
    archive_type: &str,
) -> Result<(), String> {
    let mut command = match archive_type.to_lowercase().as_ref() {
        "tar" | "tar.gz" | "targz" | "tgz" | "tar.xz" | "txz" | "tarxz" => {
            let mut cmd = Command::new("tar");
            cmd.arg("-xvf"); // Extract verbose file
            cmd.arg(name); // File name
            cmd.arg("--strip-components=1"); // Curl into this directory
            cmd
        }
        invalid => return Err(format!("Invalid archive type '{invalid}'")),
    };

    command.current_dir(path);

    command.stdout(std::process::Stdio::piped());
    command.stderr(std::process::Stdio::piped());

    let spawn = command.spawn().map_err(|e| e.to_string())?;
    let (result, stdout, stderr) = crate::cli::child_logger(spawn);

    // if result.is_err() {
    //     return Err("Failed to run tar command".to_string());
    // }
    // let result = result.unwrap();

    let result = match result {
        Ok(x) => x,
        Err(x) => return Err(x.to_string()),
    };

    if !result.success() {
        return Err(format!(
            "Failed to extract archive: \n{}\n{}",
            stdout.join("\n"),
            stderr.join("\n")
        ));
    }

    Ok(())
}
