use crate::loop_step::{Env, ResultPrinter, ShellCommand};
use crate::state::{State, Summary};

use std::fs::File;

use regex::Regex;
use subprocess::{Exec, ExitStatus, Redirection};

pub struct RealEnv {}

impl Env for RealEnv {
    fn set(&self, k: &str, v: &str) {
        std::env::set_var(k, v);
    }
}

pub struct RealShellCommand {}

impl ShellCommand for RealShellCommand {
    fn run(&self, mut state: State, cmd_with_args: &str) -> (ExitStatus, State) {
        use std::io::{prelude::*, SeekFrom};

        state.tmpfile.seek(SeekFrom::Start(0)).ok();
        state.tmpfile.set_len(0).ok();

        let status = Exec::shell(cmd_with_args)
            .stdout(Redirection::File(state.tmpfile.try_clone().unwrap()))
            .stderr(Redirection::Merge)
            .capture()
            .unwrap()
            .exit_status;

        (status, state)
    }
}

pub struct RealResultPrinter {
    only_last: bool,
    until_contains: Option<String>,
    until_match: Option<Regex>,
}

impl RealResultPrinter {
    pub fn new(
        only_last: bool,
        until_contains: Option<String>,
        until_match: Option<Regex>,
    ) -> RealResultPrinter {
        RealResultPrinter {
            only_last,
            until_contains,
            until_match,
        }
    }
}

impl ResultPrinter for RealResultPrinter {
    fn print(&self, mut state: State, stdout: &str) -> State {
        stdout.lines().for_each(|line| {
            // --only-last
            // If we only want output from the last execution,
            // defer printing until later
            if !self.only_last {
                println!("{}", line);
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

pub fn pre_exit_tasks(only_last: bool, print_summary: bool, summary: Summary, mut tmpfile: File) {
    use crate::util::StringFromTempfileStart;

    if only_last {
        String::from_temp_start(&mut tmpfile)
            .lines()
            .for_each(|line| println!("{}", line));
    }

    if print_summary {
        summary.print()
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ExitCode {
    Okay,
    Error,
    MinorError,
    /// same exit-code as used by the `timeout` shell command (99)
    Timeout,
    Unkonwn,
    Other(u32),
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
