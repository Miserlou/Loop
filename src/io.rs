use crate::state::State;

use std::io::Write;

use regex::Regex;
use smart_default::SmartDefault;
use subprocess::{Exec, ExitStatus, Redirection};

pub struct Printer<T: Write> {
    pub only_last: bool,
    pub until_contains: Option<String>,
    pub until_match: Option<Regex>,
    pub summary: bool,
    pub last_output: String,
    pub w: T,
}

impl<W> Printer<W>
where
    W: Write,
{
    pub fn terminatory_print(&mut self, create_summary: impl Fn() -> String) {
        if self.only_last {
            writeln!(self.w, "{}", self.last_output).unwrap();
        }

        if self.summary {
            self.w.write_all(create_summary().as_bytes()).unwrap();
        }
    }

    pub fn print(&mut self, text: &str, state: &mut State) {
        let mut last_line = String::default();

        text.lines().for_each(|line| {
            // --only-last
            // If we only want output from the last execution,
            // defer printing until later
            if !self.only_last {
                writeln!(self.w, "{}", line).unwrap(); // THIS IS THE MAIN PRINT FUNCTION
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

            if self.only_last {
                last_line = line.to_owned();
            }
        });

        // maybe keep the last line for later
        if self.only_last {
            self.last_output = last_line;
        }
    }
}

pub struct ShellCommand {
    pub cmd_with_args: String,
}

impl ShellCommand {
    #[must_use]
    pub fn run(&self) -> (String, ExitCode) {
        Exec::shell(&self.cmd_with_args)
            .stdout(Redirection::Pipe)
            .stderr(Redirection::Merge)
            .capture()
            .map(|it| (it.stdout_str(), it.exit_status.into()))
            .unwrap()
    }
}

pub struct SetupEnv {
    pub count_precision: usize,
}

impl SetupEnv {
    pub fn run(&self, item: Option<String>, index: f64, count: f64) {
        use std::env::set_var;

        // THESE ARE FLIPPED AND I CAN'T UNFLIP THEM.
        set_var("ACTUALCOUNT", index.to_string());
        set_var("COUNT", format!("{:.*}", self.count_precision, count));

        // Set current item as environment variable
        if let Some(item) = item {
            set_var("ITEM", item);
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy, SmartDefault)]
pub enum ExitCode {
    #[default]
    Okay,

    Error,
    /// e.g. when no argument is passed to loop-rs (2)
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
