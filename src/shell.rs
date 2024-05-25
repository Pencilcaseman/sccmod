// Long story short, Rust's std::process::Command doesn't let you do
// what I need to do, so now this exists...

use crate::{cli::child_logger, config};
use std::process::Command;
use std::process::ExitStatus;

pub enum ShellOutput {
    Success,
    StartupFailure(String),
    RuntimeFailure((Vec<String>, Vec<String>)),
}

pub struct Shell {
    shell: String,
    working_directory: String,
    commands: Vec<String>,
}

impl Shell {
    pub fn new(shell: &str) -> Self {
        Self {
            shell: shell.to_string(),
            working_directory: "/".to_string(),
            commands: Vec::new(),
        }
    }

    pub fn default() -> Self {
        Self {
            shell: config::read().unwrap().shell,
            working_directory: "/".to_string(),
            commands: Vec::new(),
        }
    }

    pub fn set_current_dir(&mut self, dir: &str) {
        self.working_directory = dir.to_string();
    }

    pub fn add_command(&mut self, cmd: &str) {
        self.commands.push(cmd.to_string());
    }

    pub fn exec(&self) -> (std::io::Result<ExitStatus>, Vec<String>, Vec<String>) {
        // Instantiate shell
        let mut shell = Command::new(&self.shell);
        shell.stdin(std::process::Stdio::piped());
        shell.stdout(std::process::Stdio::piped());
        shell.stderr(std::process::Stdio::piped());

        shell.arg("-c");

        let mut cmd = format!("cd \"{}\"", self.working_directory);
        for c in &self.commands {
            cmd.push_str(&format!(" && {}", c));
        }

        shell.arg(cmd);

        println!("Command: {shell:?}");

        child_logger(shell.spawn().unwrap())
    }
}
