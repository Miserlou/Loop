use std::fs::File;

use subprocess::ExitStatus;

static UNKONWN_EXIT_CODE: u32 = 99;

pub struct State {
    pub tmpfile: File,
    pub summary: Summary,
    pub exit_status: i32,
    pub previous_stdout: Option<String>,
    pub has_matched: bool,
}

pub struct Counters {
    pub index: usize,
    pub count_precision: usize,
    pub actual_count: f64,
}

impl State {
    pub fn summary_exit_status(&mut self, exit_status: subprocess::ExitStatus) {
        match exit_status {
            ExitStatus::Exited(0) => self.summary.successes += 1,
            ExitStatus::Exited(n) => self.summary.failures.push(n),
            _ => self.summary.failures.push(UNKONWN_EXIT_CODE),
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
            exit_status: 0,
        }
    }
}

#[derive(Debug)]
pub struct Summary {
    successes: u32,
    failures: Vec<u32>,
}

impl Summary {
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
                    .map(|f| (-(f as i32)).to_string())
                    .collect::<Vec<String>>()
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
