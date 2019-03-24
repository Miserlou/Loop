use crate::io::ExitCode;

pub struct State {
    pub has_matched: bool,
    pub previous_stdout: Option<String>,
    pub successes: u32,
    pub failures: Vec<u32>,
    pub exit_code: ExitCode,
}

impl State {
    pub fn update_summary(&mut self, exit_code: ExitCode) {
        match exit_code {
            ExitCode::Okay => self.successes += 1,
            err => self.failures.push(err.into()),
        }
    }
}

impl ToString for State {
    fn to_string(&self) -> String {
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
        let mut s = String::new();
        s.push_str(&format!("Total runs:\t{}\n", total));
        s.push_str(&format!("Successes:\t{}\n", self.successes));
        s.push_str(&format!("Failures:\t{}\n", errors));
        s
    }
}

impl Default for State {
    fn default() -> State {
        State {
            has_matched: false,
            previous_stdout: None,
            successes: 0,
            failures: vec![],
            exit_code: ExitCode::Okay,
        }
    }
}
