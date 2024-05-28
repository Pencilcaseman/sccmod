#![feature(fs_try_exists)]
#![feature(round_char_boundary)]
#![feature(dir_entry_ext2)]

pub mod builders {
    pub mod builder_trait;
    pub mod cmake;
    pub mod make;
}

pub mod archive;
pub mod callbacks;
pub mod cli;
pub mod config;
pub mod downloaders;
pub mod environment;
pub mod file_manager;
pub mod flavours;
pub mod info;
pub mod log;
pub mod module;
pub mod module_resolver;
pub mod modulefile;
pub mod python_interop;
pub mod shell;
