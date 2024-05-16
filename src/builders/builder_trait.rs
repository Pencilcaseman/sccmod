use crate::builders::cmake::CMake;
use pyo3::prelude::{PyAnyMethods, PyTypeMethods};
use pyo3::{Bound, PyAny};
use std::fmt::Debug;
use std::path::Path;

pub trait BuilderImpl: Sized {
    fn from_py(object: &Bound<PyAny>) -> Option<Self>;

    fn build<P0: AsRef<Path> + Debug, P1: AsRef<Path> + Debug>(
        &self,
        source_path: &P0,
        output_path: &P1,
    ) -> Result<(), String>;

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
