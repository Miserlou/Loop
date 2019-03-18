use crate::state::State;

use regex::Regex;
use subprocess::{Exec, ExitStatus, Redirection};

pub fn setup_environment(
    item: Option<&String>,
    index: usize,
    count_precision: usize,
    actual_count: f64,
) {
    use std::env::set_var;

    // THESE ARE FLIPPED AND I CAN'T UNFLIP THEM.
    set_var("ACTUALCOUNT", index.to_string());
    set_var("COUNT", format!("{:.*}", count_precision, actual_count));

    // Set current item as environment variable
    if let Some(item) = item {
        set_var("ITEM", item);
    }
}

#[must_use]
pub fn shell_command(cmd_with_args: &str, mut state: State) -> (ExitCode, State) {
    use std::io::{prelude::*, SeekFrom};

    state.tmpfile.seek(SeekFrom::Start(0)).ok();
    state.tmpfile.set_len(0).ok();

    let exit_status = Exec::shell(cmd_with_args)
        .stdout(Redirection::File(state.tmpfile.try_clone().unwrap()))
        .stderr(Redirection::Merge)
        .capture()
        .unwrap()
        .exit_status;

    (exit_status.into(), state)
}

pub struct Printer {
    pub only_last: bool,
    pub until_contains: Option<String>,
    pub until_match: Option<Regex>,
}

impl Printer {
    #[must_use]
    pub fn print(&self, stdout: &str, mut state: State) -> State {
        stdout.lines().for_each(|line| {
            // --only-last
            // If we only want output from the last execution,
            // defer printing until later
            if !self.only_last {
                println!("{}", line); // THIS IS THE MAIN PRINT FUNCTION
            }

            // --until-contains
            // We defer loop breaking until the entire result is printed.
            if let Some(ref string) = self.until_contains {
                state.has_matched = line.contains(string);
            }

            // --until-match
            if let Some(ref regex) = self.until_match {
                state.has_matched = regex.captures(&line).is_some();
            }
        });

        state
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ExitCode {
    Okay,
    Error,
    MinorError,
    /// same exit-code as used by the `timeout` shell command (124)
    Timeout,
    /// the process has completed, but the exit-code is unknown (99)
    Unkonwn,
    Other(u32),
}

impl ExitCode {
    pub fn success(self) -> bool {
        ExitCode::Okay == self
    }
}

impl From<u32> for ExitCode {
    fn from(n: u32) -> ExitCode {
        match n {
            0 => ExitCode::Okay,
            1 => ExitCode::Error,
            2 => ExitCode::MinorError,
            99 => ExitCode::Unkonwn,
            124 => ExitCode::Timeout,
            code => ExitCode::Other(code),
        }
    }
}

impl From<i32> for ExitCode {
    fn from(n: i32) -> ExitCode {
        ExitCode::from(n as u32)
    }
}

impl From<ExitStatus> for ExitCode {
    fn from(exit_status: ExitStatus) -> ExitCode {
        match exit_status {
            ExitStatus::Exited(code) => ExitCode::from(code),
            _ => ExitCode::Unkonwn,
        }
    }
}

impl Into<u32> for ExitCode {
    fn into(self) -> u32 {
        match self {
            ExitCode::Okay => 0,
            ExitCode::Error => 1,
            ExitCode::MinorError => 2,
            ExitCode::Unkonwn => 99,
            ExitCode::Timeout => 124,
            ExitCode::Other(code) => code,
        }
    }
}

impl Into<i32> for ExitCode {
    fn into(self) -> i32 {
        Into::<u32>::into(self) as i32
    }
}
