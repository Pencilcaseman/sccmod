use std::io;
use std::io::{BufRead, ErrorKind};
use std::process::ExitStatus;
use std::thread;

use anstyle::AnsiColor;
use clap::builder::styling::Styles;

use crate::{config, log};

/// Log the output of a process as info messages on a single line
///
/// # Errors
/// If the process fails to start or run, an error message is returned.
///
/// # Panics
/// Should never panic
pub fn child_logger(
    mut spawn: std::process::Child,
) -> (io::Result<ExitStatus>, Vec<String>, Vec<String>) {
    let stdout = spawn.stdout.take();
    let stderr = spawn.stderr.take();

    if stdout.is_none() || stderr.is_none() {
        return (Err(ErrorKind::BrokenPipe.into()), Vec::new(), Vec::new());
    }

    let stdout = stdout;
    let stderr = stderr;

    if stdout.is_none() || stderr.is_none() {
        return (
            Err(ErrorKind::ConnectionRefused.into()),
            Vec::new(),
            Vec::new(),
        );
    }
    let stdout = stdout.unwrap();
    let stderr = stderr.unwrap();

    let stdout_reader = io::BufReader::new(stdout);
    let stderr_reader = io::BufReader::new(stderr);

    let console_width = crossterm::terminal::size().unwrap_or((24_u16, 80_u16)).0 as usize;

    let stdout_lines = stdout_reader.lines().map_while(Result::ok);
    let stderr_lines = stderr_reader.lines().map_while(Result::ok);

    let mut stdout = Vec::new();
    let mut stderr = Vec::new();

    let stdout_thread = thread::Builder::new()
        .name("STDOUT".to_string())
        .spawn(move || {
            for mut line in stdout_lines {
                stdout.push(line.clone());
                let trunc = line.floor_char_boundary(console_width.max(13) - 13);
                line.truncate(trunc);
                log::info_carriage(&line);
            }

            stdout
        });

    if stdout_thread.is_err() {
        return (Err(ErrorKind::BrokenPipe.into()), Vec::new(), Vec::new());
    }
    let stdout_thread = stdout_thread.unwrap();

    let stderr_thread = thread::Builder::new()
        .name("STDERR".to_string())
        .spawn(move || {
            for mut line in stderr_lines {
                stderr.push(line.clone());
                let trunc = line.floor_char_boundary(console_width.max(13) - 13);
                line.truncate(trunc);
                log::warn_carriage(&line);
            }

            stderr
        });

    if stderr_thread.is_err() {
        return (Err(ErrorKind::BrokenPipe.into()), Vec::new(), Vec::new());
    }
    let stderr_thread = stderr_thread.unwrap();

    let stdout = stdout_thread
        .join()
        .unwrap_or_else(|_| vec!["stdout failed".to_string()]);

    let stderr = stderr_thread
        .join()
        .unwrap_or_else(|_| vec!["stderr failed".to_string()]);

    print!("\x1b[K\r");

    (spawn.wait(), stdout, stderr)
}

pub trait CommandBuilder {
    #[must_use]
    fn add_subcommand(self, cmd: clap::Command) -> Self;

    #[must_use]
    fn add_argument(self, name: &'static str, help: &'static str) -> Self;
}

#[must_use]
pub fn command_group(id: &'static str, multiple: bool) -> clap::Command {
    clap::Command::new("sccmod").group(clap::ArgGroup::new(id).multiple(multiple))
}

impl CommandBuilder for clap::Command {
    fn add_subcommand(self, cmd: clap::Command) -> Self {
        self.subcommand(cmd)
    }

    fn add_argument(self, name: &'static str, help: &'static str) -> Self {
        self.arg(clap::Arg::new(name).help(help))
    }
}

type CommandCallback = fn(&config::Config) -> Result<(), String>;
type ArgumentCallback = fn(&str, &config::Config) -> Result<(), String>;

pub struct Arg {
    pub name: &'static str,
    pub help: &'static str,
    pub callback: ArgumentCallback,
}

pub struct Command {
    pub name: &'static str,
    pub subcommands: Vec<Command>,
    pub arguments: Vec<Arg>,
    pub help: &'static str,
    pub callback: Option<CommandCallback>,
}

impl Command {
    #[must_use]
    pub fn generate_command(&self) -> clap::Command {
        let mut res = clap::command!().name(self.name).styles(
            Styles::styled()
                .usage(AnsiColor::BrightMagenta.on_default().bold().underline())
                .header(AnsiColor::BrightBlue.on_default().bold().underline())
                .literal(AnsiColor::BrightCyan.on_default().bold())
                .placeholder(AnsiColor::BrightCyan.on_default()),
        );

        if self.callback.is_none() {
            res = res.arg_required_else_help(true);
        }

        for sub in &self.subcommands {
            res = res.subcommand(sub.generate_command());
        }

        res = res.about(self.help);

        for arg in &self.arguments {
            res = res.add_argument(arg.name, arg.help);
        }

        res
    }

    /// Consume a command recursively, running callbacks where suitable.
    ///
    /// # Errors
    ///
    /// Errors if invalid commands were passed or if the callback fails.
    pub fn consume(
        &self,
        config: &config::Config,
        matches: &clap::ArgMatches,
    ) -> Result<(), String> {
        let mut arg_count = 0;
        for arg in &self.arguments {
            if let Some(value) = matches.get_one::<String>(arg.name) {
                (arg.callback)(value, config)?;
                arg_count += 1;

                if arg_count > 1 {
                    return Err("Too many arguments passed".to_string());
                }
            }
        }

        let mut sub_count = 0;
        for sub in &self.subcommands {
            if let Some(matches) = matches.subcommand_matches(sub.name) {
                sub.consume(config, matches)?;
                sub_count += 1;

                if sub_count + arg_count > 1 {
                    return Err("Too many arguments passed".to_string());
                }
            }
        }

        if arg_count + sub_count == 0 {
            if let Some(callback) = self.callback {
                callback(config)?;
            }
        }

        Ok(())
    }
}
