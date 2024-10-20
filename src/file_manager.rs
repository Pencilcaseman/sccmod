use std::{
    fs::DirEntry,
    path::{Path, PathBuf},
    process::Command,
};

use crate::log;

/// Defines the path separator for a given operating system
#[cfg(not(target_os = "windows"))]
pub static PATH_SEP: char = '/';
#[cfg(target_os = "windows")]
pub static PATH_SEP: char = '/';

/// Return a vector of all items in a nested folder structure
///
/// # Panics
/// - If an item is found and its nature cannot be determined (file, folder,
///   etc.)
pub fn recursive_list_dir<P: AsRef<Path>>(root: &P) -> Option<Vec<DirEntry>> {
    let root = std::fs::read_dir(root);
    if let Err(msg) = root {
        log::warn(&format!("Failed to read from directory {msg:?}"));
        return None;
    }
    let root = root.unwrap();

    let mut result = Vec::new();

    for item in root.flatten() {
        if item.file_type().ok()?.is_file() {
            result.push(item);
        } else if item.file_type().ok()?.is_dir() {
            result.append(&mut recursive_list_dir(&item.path())?);
        } else {
            log::warn(&format!("Found item with unknown filetype: {item:?}"));
        }
    }

    Some(result)
}

/// Count the depth of a path.
///
/// # Example
/// ```rust
/// use sccmod::file_manager::dir_level;
///
/// assert_eq!(dir_level(&"dir1"), 1);
/// assert_eq!(dir_level(&"dir1/dir2"), 2);
/// assert_eq!(dir_level(&"dir1/.."), 0);
/// assert_eq!(dir_level(&"dir1/../.."), -1);
/// assert_eq!(dir_level(&"dir1/./.."), 0);
/// assert_eq!(dir_level(&"././../../test/thing"), 0);
/// ```
///
/// # Panics
///
/// Panics if the provided path cannot be converted into a string
pub fn dir_level<P: AsRef<Path>>(path: &P) -> isize {
    path.as_ref()
        .to_str()
        .expect("Failed to convert path to string")
        .split('/')
        .map(|seg| match seg {
            "." => 0,
            ".." => -1,
            _ => 1,
        })
        .sum()
}

pub fn absolute_path<P: AsRef<Path>>(path: &P) -> Option<PathBuf> {
    // if std::fs::try_exists(path).ok()? {
    if std::fs::exists(path).ok()? {
        let mut command = Command::new("pwd");
        command.current_dir(path);

        Some(PathBuf::from(
            &String::from_utf8_lossy(&command.output().ok()?.stdout)
                .trim()
                .to_string(),
        ))
    } else {
        None
    }
}
