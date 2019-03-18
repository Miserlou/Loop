use crate::io::ExitCode;

use std::io::Cursor;

pub struct State {
    /// shell command in-memory output buffer
    pub buf: Cursor<Vec<u8>>,
    pub summary: Summary,
    pub exit_code: ExitCode,
    pub previous_stdout: Option<String>,
    pub has_matched: bool,
}

impl State {
    pub fn buffer_to_string(&mut self) -> String {
        use std::io::SeekFrom;
        use std::io::{Read, Seek};

        let mut output = String::new();
        self.buf.seek(SeekFrom::Start(0)).ok();
        self.buf.read_to_string(&mut output).ok();
        output
    }
}

impl Default for State {
    fn default() -> State {
        State {
            has_matched: false,
            buf: Cursor::new(vec![]),
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

    pub fn print(&self) {
        let total = self.successes + self.failures.len() as u32;

        let errors = if self.failures.is_empty() {
            String::from("0")
        } else {
            format!(
                "{} ({})",
                self.failures.len(),
                self.failures
                    .iter()
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
