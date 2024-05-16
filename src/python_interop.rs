use pyo3::prelude::*;
use std::fs::read_to_string;
use std::path::Path;

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

pub fn extract_object<'a>(
    object: &Bound<'a, PyAny>,
    name: &str,
) -> Result<Bound<'a, PyAny>, String> {
    object
        .getattr(name)
        .map_err(|err| format!("Failed to extract target attribute: {err}"))
}
