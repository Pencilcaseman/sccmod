use pyo3::prelude::*;
use std::fs::read_to_string;
use std::path::Path;

/// Load a Python program from a file path.
///
/// # Errors
/// Errors if the file does not exist, cannot be read or contains invalid
/// Python code.
pub fn load_program<'a, P: AsRef<Path> + std::fmt::Debug>(
    py: &'a Python,
    path: &P,
) -> Result<Bound<'a, PyModule>, String> {
    let mut source_dir: String = env!("CARGO_MANIFEST_DIR").into();
    source_dir.push_str("/src");

    let file = read_to_string(path).map_err(|err| err.to_string())?;
    let code = format!("import sys\nsys.path.append('{source_dir}')\n{file}");

    PyModule::from_code_bound(*py, &code, "", "")
        .map_err(|err| format!("Failed to load python program: {err}"))
}

/// Extract a named attribute of a Python object.
///
/// # Errors
/// Errors if the attribute cannot be found.
pub fn extract_object<'a>(
    object: &Bound<'a, PyAny>,
    name: &str,
) -> Result<Bound<'a, PyAny>, String> {
    object
        .getattr(name)
        .map_err(|err| format!("Failed to extract target attribute: {err}"))
}
