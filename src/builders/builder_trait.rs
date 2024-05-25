use crate::builders::{cmake::CMake, make::Make};
use pyo3::prelude::{PyAnyMethods, PyTypeMethods};
use pyo3::{Bound, PyAny};
use std::fmt::Debug;
use std::path::Path;

pub trait BuilderImpl: Sized {
    /// Generate a builder object from a python object.
    ///
    /// Returns [`Ok(obj)`] if the operation was successful, otherwise [`Err(string)`]
    ///
    /// # Errors
    /// Errors if any attribute cannot be extracted or converted, or if the provided
    /// object is invalid.
    fn from_py(object: &Bound<PyAny>) -> Result<Self, String>;

    /// Perform the build operation specified by the struct.
    ///
    /// For example, if this is a [`CMake`] instance, `cmake` is run on the
    /// `source_path` and the resulting build files are written to `output_path`.
    ///
    /// # Errors
    ///
    /// Returns [`Err(string)`] if the build script fails to run. This could happen
    /// for many reasons, including:
    ///  - Source directory does not exist
    ///  - Source directory does not contain a valid build script configuration
    ///  - The code fails to compile
    ///  - The build files cannot be written to the build directory
    fn build<P0: AsRef<Path> + Debug, P1: AsRef<Path> + Debug, P2: AsRef<Path>>(
        &self,
        source_path: &P0,
        build_path: &P1,
        install_path: &P2, // Necessary for make
        dependencies: &[String],
    ) -> Result<(), String>;

    /// Perform the install operation specified by the struct.
    ///
    /// For example, if this is a [`CMake`] instance, `cmake --install ...` is run
    /// on the `build_path` and the install files are installed in `install_path`.
    ///
    /// # Errors
    ///
    /// Returns [`Err(string)`] if the install script fails to run. This could
    /// happen for many reasons, including:
    ///  - Build directory does not exist
    ///  - Build directory does not contain valid installation information
    ///  - The install files cannot be written to `install_path`
    fn install<P0: AsRef<Path>, P1: AsRef<Path>>(
        &self,
        build_path: &P0,
        install_path: &P1,
        dependencies: &[String],
    ) -> Result<(), String>;
}

#[derive(Debug)]
pub enum Builder {
    CMake(CMake),
    Make(Make),
}

impl BuilderImpl for Builder {
    fn from_py(object: &Bound<PyAny>) -> Result<Self, String> {
        let name = object.get_type().name().unwrap().to_string();

        match name.as_str() {
            "CMake" => Ok(Self::CMake(CMake::from_py(object)?)),
            "Make" => Ok(Self::Make(Make::from_py(object)?)),
            _ => Err("Invalid builder type".to_string()),
        }
    }

    fn build<P0: AsRef<Path> + Debug, P1: AsRef<Path> + Debug, P2: AsRef<Path>>(
        &self,
        source_path: &P0,
        build_path: &P1,
        install_path: &P2,
        dependencies: &[String],
    ) -> Result<(), String> {
        match self {
            Self::CMake(cmake) => cmake.build(source_path, build_path, install_path, dependencies),
            Self::Make(make) => make.build(source_path, build_path, install_path, dependencies),
        }
    }

    fn install<P0: AsRef<Path>, P1: AsRef<Path>>(
        &self,
        build_path: &P0,
        install_path: &P1,
        dependencies: &[String],
    ) -> Result<(), String> {
        match self {
            Self::CMake(cmake) => cmake.install(build_path, install_path, dependencies),
            Self::Make(make) => make.install(build_path, install_path, dependencies),
        }
    }
}
