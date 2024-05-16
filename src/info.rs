use colored::Colorize;
use pyo3::prelude::PyAnyMethods;
use pyo3::types::{IntoPyDict, PyString};
use pyo3::Python;

const CRATE_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Print module and Python information
pub fn print() {
    Python::with_gil(|py| {
        let sys = py.import_bound("sys").unwrap_or_else(|_| {
            crate::log::error("Failed to import sys module");
        });

        let version = sys.getattr("version").unwrap_or_else(|_| {
            crate::log::error("Failed to get version attribute from sys");
        });

        let version: String = version.extract().unwrap_or_else(|_| {
            crate::log::error("Could not convert sys version output to String -- this is likely an internal error");
        });

        let os = py.import_bound("os").unwrap_or_else(|_| {
            crate::log::error("Failed to import os module");
        });

        let locals = [("os", os)].into_py_dict_bound(py);
        let code = "os.getenv('USER') or os.getenv('USERNAME') or 'Unknown'";

        let user = py
            .eval_bound(code, None, Some(&locals))
            .unwrap_or_else(|_| {
                crate::log::warn("Failed to extract user from Python environment variables");
                PyString::new_bound(py, "Unknown").into_any()
            });

        let user: String = user.extract().unwrap_or_else(|_| {
            crate::log::error(
                "Could not convert user output to String -- this is likely an internal error",
            );
        });

        println!("{}: v{}", "SCCMod".purple().bold(), CRATE_VERSION.bold());
        println!("{}: {}", "Python".purple().bold(), version.bold());
        println!("{}  : {}", "User".purple().bold(), user.bold());
    });

    // Todo: Print loaded modules?
}
