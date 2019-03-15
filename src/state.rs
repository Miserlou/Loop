use std::fs::File;

use subprocess::ExitStatus;

pub struct State {
    pub tmpfile: File,
    pub summary: Summary,
    pub exit_code: ExitCode,
    pub previous_stdout: Option<String>,
    pub has_matched: bool,
}

impl State {
    pub fn update_summary(&mut self, exit_status: ExitStatus) {
        match exit_status {
            ExitStatus::Exited(0) => self.summary.successes += 1,
            ExitStatus::Exited(n) => self.summary.failures.push(n as i32),
            _ => self.summary.failures.push(ExitCode::Unkonwn.into()),
        }
    }
}

impl Default for State {
    fn default() -> State {
        State {
            has_matched: false,
            tmpfile: tempfile::tempfile().unwrap(),
            summary: Summary::default(),
            previous_stdout: None,
            exit_code: ExitCode::Okay,
        }
    }
}

pub struct Counters {
    pub index: usize,
    pub count_precision: usize,
    pub actual_count: f64,
}

#[derive(Debug)]
pub struct Summary {
    successes: i32,
    failures: Vec<i32>,
}

impl Summary {
    pub fn print(self) {
        let total = self.successes + self.failures.len() as i32;

        let errors = if self.failures.is_empty() {
            String::from("0")
        } else {
            format!(
                "{} ({})",
                self.failures.len(),
                self.failures
                    .into_iter()
                    .map(|f| f.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        };

        println!("Total runs:\t{}", total);
        println!("Successes:\t{}", self.successes);
        println!("Failures:\t{}", errors);
    }
}

impl Default for Summary {
    fn default() -> Summary {
        Summary {
            successes: 0,
            failures: Vec::new(),
        }
    }
}

pub enum ExitCode {
    Okay = 0,
    MinorError = 2,
    /// same exit-code as used by the `timeout` shell command
    Timeout = 124,
    Unkonwn = 99,
}

impl From<ExitCode> for i32 {
    fn from(ec: ExitCode) -> i32 {
        ec as i32
    }
}
