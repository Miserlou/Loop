use crate::setup::{ErrorCode, Opt};
use crate::util::StringFromTempfileStart;

use std::fs::File;
use std::time::{Instant, SystemTime};

use subprocess::{Exec, ExitStatus, Redirection};

// same exit-code as used by the `timeout` shell command
static TIMEOUT_EXIT_CODE: i32 = 124;
static UNKONWN_EXIT_CODE: u32 = 99;

pub struct State {
    pub tmpfile: File,
    pub summary: Summary,
    pub exit_status: i32,
    previous_stdout: Option<String>,
    has_matched: bool,
}

pub struct Counters {
    pub index: usize,
    pub count_precision: usize,
    pub actual_count: f64,
}

impl State {
    pub fn loop_body(
        &mut self,
        opt: &Opt,
        items: &[String],
        cmd_with_args: &str,
        counters: Counters,
        program_start: Instant,
    ) -> bool {
        use std::env;
        use std::thread;

        // Time Start
        let loop_start = Instant::now();

        // Set counters before execution
        // THESE ARE FLIPPED AND I CAN'T UNFLIP THEM.
        env::set_var("ACTUALCOUNT", counters.index.to_string());
        env::set_var(
            "COUNT",
            format!("{:.*}", counters.count_precision, counters.actual_count),
        );

        // Set iterated item as environment variable
        if let Some(item) = items.get(counters.index) {
            env::set_var("ITEM", item);
        }

        // Finish if we're over our duration
        if let Some(duration) = opt.for_duration {
            let since = Instant::now().duration_since(program_start);
            if since >= duration {
                if opt.error_duration {
                    self.exit_status = TIMEOUT_EXIT_CODE
                }
                return true;
            }
        }

        // Finish if our time until has passed
        // In this location, the loop will execute at least once,
        // even if the start time is beyond the until time.
        if let Some(until_time) = opt.until_time {
            if SystemTime::now().duration_since(until_time).is_ok() {
                return true;
            }
        }

        // Main executor
        let exit_status = self.run_shell_command(cmd_with_args);

        // Print the results
        let stdout = String::from_temp_start(&mut self.tmpfile);
        self.print_results(&opt, &stdout);

        // --until-error
        check_for_error(&opt.until_error, &mut self.has_matched, exit_status);

        // --until-success
        if opt.until_success && exit_status.success() {
            self.has_matched = true;
        }

        // --until-fail
        if opt.until_fail && !(exit_status.success()) {
            self.has_matched = true;
        }

        if opt.summary {
            self.summary_exit_status(exit_status);
        }

        // Finish if we matched
        if self.has_matched {
            return true;
        }

        if let Some(ref previous_stdout) = self.previous_stdout {
            // --until-changes
            if opt.until_changes && *previous_stdout != stdout {
                return true;
            }

            // --until-same
            if opt.until_same && *previous_stdout == stdout {
                return true;
            }
        }
        self.previous_stdout = Some(stdout);

        // Delay until next iteration time
        let since = Instant::now().duration_since(loop_start);
        if let Some(time) = opt.every.checked_sub(since) {
            thread::sleep(time);
        }

        false
    }

    fn run_shell_command(&mut self, cmd_with_args: &str) -> ExitStatus {
        use std::io::{prelude::*, SeekFrom};

        self.tmpfile.seek(SeekFrom::Start(0)).ok();
        self.tmpfile.set_len(0).ok();

        Exec::shell(cmd_with_args)
            .stdout(Redirection::File(self.tmpfile.try_clone().unwrap()))
            .stderr(Redirection::Merge)
            .capture()
            .unwrap()
            .exit_status
    }

    fn summary_exit_status(&mut self, exit_status: subprocess::ExitStatus) {
        match exit_status {
            ExitStatus::Exited(0) => self.summary.successes += 1,
            ExitStatus::Exited(n) => self.summary.failures.push(n),
            _ => self.summary.failures.push(UNKONWN_EXIT_CODE),
        }
    }

    fn print_results(&mut self, opt: &Opt, stdout: &str) {
        stdout.lines().for_each(|line| {
            // --only-last
            // If we only want output from the last execution,
            // defer printing until later
            if !opt.only_last {
                println!("{}", line);
            }

            // --until-contains
            // We defer loop breaking until the entire result is printed.
            if let Some(ref string) = opt.until_contains {
                self.has_matched = line.contains(string);
            }

            // --until-match
            if let Some(ref regex) = opt.until_match {
                self.has_matched = regex.captures(&line).is_some();
            }
        })
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

fn check_for_error(
    maybe_error: &Option<ErrorCode>,
    has_matched: &mut bool,
    exit_status: ExitStatus,
) {
    match maybe_error {
        Some(ErrorCode::Any) => {
            *has_matched = !exit_status.success();
        }
        Some(ErrorCode::Code(code)) => {
            if exit_status == ExitStatus::Exited(*code) {
                *has_matched = true;
            }
        }
        _ => (),
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
