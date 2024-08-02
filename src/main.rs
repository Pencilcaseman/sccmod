use sccmod::{callbacks, cli, cli::NumParams, config};

#[allow(clippy::cognitive_complexity)]
fn main() -> Result<(), String> {
    let config = config::read()?;
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
            cli::Command {
                name: "modulefile",
                subcommands: vec![cli::Command {
                    name: "all",
                    subcommands: Vec::new(),
                    arguments: Vec::new(),
                    help: "Write modulefiles for all available modules",
                    callback: Some(callbacks::write_modulefile_all),
                }],
                arguments: vec![cli::Arg {
                    name: "module",
                    help: "Write a modulefile for the specified module",
                    num_params: NumParams::Any,
                    callback: callbacks::write_modulefile,
                }],
                help: "Automatically generate modulefiles",
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
