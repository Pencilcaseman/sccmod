use sccmod::{callbacks, cli, cli::NumParams, config};

#[allow(clippy::cognitive_complexity)]
fn main() -> Result<(), String> {
    let config = config::read()?;
    println!("Module Paths: {:?}", config.module_paths);
    println!("Build Root: {}", config.build_root);
    println!("Install Root: {}", config.install_root);

    cli(&config)?;

    Ok(())
}

fn cli(config: &config::Config) -> Result<(), String> {
    let command = cli::Command {
        name: "sccmod",
        subcommands: vec![
            cli::Command {
                name: "list",
                subcommands: Vec::new(),
                arguments: Vec::new(),
                help: "List all available modules",
                callback: Some(callbacks::list_callback),
            },
            cli::Command {
                name: "download",
                subcommands: Vec::new(),
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
                subcommands: Vec::new(),
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
                subcommands: Vec::new(),
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
