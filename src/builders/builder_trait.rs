use crate::builders::cmake::CMake;
use pyo3::prelude::{PyAnyMethods, PyTypeMethods};
use pyo3::{Bound, PyAny};
use std::fmt::Debug;
use std::path::Path;

pub trait BuilderImpl: Sized {
    /// Generate a builder object from a python object.
    ///
    /// Returns [`Some(obj)`] if the operation was successful, otherwise [`None`]
    fn from_py(object: &Bound<PyAny>) -> Option<Self>;

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
    fn build<P0: AsRef<Path> + Debug, P1: AsRef<Path> + Debug>(
        &self,
        source_path: &P0,
        output_path: &P1,
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
    ) -> Result<(), String>;
}

#[derive(Debug)]
pub enum Builder {
    CMake(CMake),
}

impl BuilderImpl for Builder {
    fn from_py(object: &Bound<PyAny>) -> Option<Self> {
        let name = object.get_type().name().unwrap().to_string();

        match name.as_str() {
            "CMake" => Some(Self::CMake(CMake::from_py(object)?)),
            _ => None,
        }
    }

    fn build<P0: AsRef<Path> + Debug, P1: AsRef<Path> + Debug>(
        &self,
        source_path: &P0,
        output_path: &P1,
    ) -> Result<(), String> {
        // todo!()

        match self {
            Self::CMake(cmake) => cmake.build(source_path, output_path),
        }
    }

    fn install<P0: AsRef<Path>, P1: AsRef<Path>>(
        &self,
        build_path: &P0,
        install_path: &P1,
    ) -> Result<(), String> {
        // todo!()

        match self {
            Self::CMake(cmake) => cmake.install(build_path, install_path),
        }
    }
}
