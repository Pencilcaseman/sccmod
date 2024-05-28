use std::os::unix::process::CommandExt;

use sccmod::{callbacks, cli, cli::NumParams, config};

#[allow(clippy::cognitive_complexity)]
fn main() -> Result<(), String> {
    // let mut shell = sccmod::shell::Shell::new("fish");
    // shell.set_current_dir("/Users/tobydavis/apps/module_build/gcc");
    // shell.add_command("contrib/download_prerequisites");

    // println!("{:?}", shell.exec());

    let config = config::read()?;

    // println!("Module Paths: {:?}", config.module_paths);
    // println!("Build Root: {}", config.build_root);
    // println!("Install Root: {}", config.install_root);

    cli(&config)
}

fn cli(config: &config::Config) -> Result<(), String> {
    let command = cli::Command {
        name: "sccmod",
        subcommands: vec![
            cli::Command {
                name: "info",
                subcommands: Vec::new(),
                arguments: Vec::new(),
                help: "Print sccmod information",
                callback: Some(callbacks::info),
            },
            cli::Command {
                name: "list",
                subcommands: Vec::new(),
                arguments: Vec::new(),
                help: "List all available modules",
                callback: Some(callbacks::list_callback),
            },
            cli::Command {
                name: "download",
                subcommands: vec![cli::Command {
                    name: "all",
                    subcommands: Vec::new(),
                    arguments: Vec::new(),
                    help: "Download all available modules",
                    callback: Some(callbacks::download_all),
                }],
                arguments: vec![cli::Arg {
                    name: "module",
                    help: "Download the specified module",
                    num_params: NumParams::Any,
                    callback: callbacks::download_module,
                }],
                help: "Download a module",
                callback: None,
            },
            cli::Command {
                name: "build",
                subcommands: vec![cli::Command {
                    name: "all",
                    subcommands: Vec::new(),
                    arguments: Vec::new(),
                    help: "Build all available modules",
                    callback: Some(callbacks::build_all),
                }],
                arguments: vec![cli::Arg {
                    name: "module",
                    help: "Build the specified module",
                    num_params: NumParams::Any,
                    callback: callbacks::build_module,
                }],
                help: "Build a module",
                callback: None,
            },
            cli::Command {
                name: "install",
                subcommands: vec![cli::Command {
                    name: "all",
                    subcommands: Vec::new(),
                    arguments: Vec::new(),
                    help: "Install all available modules",
                    callback: Some(callbacks::install_all),
                }],
                arguments: vec![cli::Arg {
                    name: "module",
                    help: "Install the specified module",
                    num_params: NumParams::Any,
                    callback: callbacks::install_module,
                }],
                help: "Install a module",
                callback: None,
            },
        ],
        arguments: vec![],
        help: "A tool for managing and building modules",
        callback: None,
    };

    let cmd = command.generate_command();
    command.consume(config, &cmd.get_matches())
}
