use crate::io::ExitCode;

use std::fs::File;

pub struct State {
    pub tmpfile: File,
    pub summary: Summary,
    pub exit_code: ExitCode,
    pub previous_stdout: Option<String>,
    pub has_matched: bool,
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

pub struct Summary {
    successes: u32,
    failures: Vec<u32>,
}

impl Summary {
    pub fn update(&mut self, exit_code: ExitCode) {
        match exit_code {
            ExitCode::Okay => self.successes += 1,
            err => self.failures.push(err.into()),
        }
    }

    pub fn print(self) {
        let total = self.successes + self.failures.len() as u32;

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
