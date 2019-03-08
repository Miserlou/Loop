mod setup;

use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::io::SeekFrom;
use std::process;
use std::thread;
use std::time::{Instant, SystemTime};

use setup::{setup, ErrorCode, Opt};
use subprocess::{Exec, ExitStatus, Redirection};

static UNKONWN_EXIT_CODE: u32 = 99;

// same exit code as use of `timeout` shell command
static TIMEOUT_EXIT_CODE: i32 = 124;

fn main() {
    // Time
    let (opt, items, count_precision, program_start) = setup();

    let joined_input = &opt.input.join(" ");
    if joined_input.is_empty() {
        println!("No command supplied, exiting.");
        return;
    }

    // Counters and State
    let mut state = State::default();
    let counters = counters_from_opt(&opt, &items);
    for (index, actual_count) in counters.iter().enumerate() {
        let counters = Counters {
            index,
            count_precision,
            actual_count: *actual_count,
        };
        if state.loop_body(&opt, &items, joined_input, counters, program_start) {
            break;
        }
    }

    exit_app(
        opt.only_last,
        opt.summary,
        state.exit_status,
        state.summary,
        state.tmpfile,
    )
}

struct State {
    has_matched: bool,
    tmpfile: File,
    summary: Summary,
    previous_stdout: Option<String>,
    exit_status: i32,
}

struct Counters {
    index: usize,
    count_precision: usize,
    actual_count: f64,
}

impl State {
    fn loop_body(
        &mut self,
        opt: &Opt,
        items: &[String],
        joined_input: &str,
        counters: Counters,
        program_start: Instant,
    ) -> bool {
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
        let exit_status = self.run_shell_command(joined_input);

        // Print the results
        let stdout = String::from_temp_start(&mut self.tmpfile);
        self.print_results(&opt, &stdout);

        // --until-error
        check_error_code(&opt.until_error, &mut self.has_matched, exit_status);

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

    fn run_shell_command(&mut self, joined_input: &str) -> ExitStatus {
        self.tmpfile.seek(SeekFrom::Start(0)).ok();
        self.tmpfile.set_len(0).ok();
        Exec::shell(joined_input)
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

fn counters_from_opt(opt: &Opt, items: &[String]) -> Vec<f64> {
    let mut counters = vec![];
    let mut start = opt.offset - opt.count_by;
    let mut index = 0_f64;
    let step_by = opt.count_by;
    let end = if let Some(num) = opt.num {
        num
    } else if !items.is_empty() {
        items.len() as f64
    } else {
        std::f64::INFINITY
    };
    loop {
        start += step_by;
        index += 1_f64;
        if index <= end {
            counters.push(start)
        } else {
            break;
        }
    }
    counters
}

fn check_error_code(
    maybe_error: &Option<ErrorCode>,
    has_matched: &mut bool,
    exit_status: subprocess::ExitStatus,
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

fn exit_app(
    only_last: bool,
    print_summary: bool,
    exit_status: i32,
    summary: Summary,
    mut tmpfile: File,
) {
    if only_last {
        String::from_temp_start(&mut tmpfile)
            .lines()
            .for_each(|line| println!("{}", line));
    }

    if print_summary {
        summary.print()
    }

    process::exit(exit_status);
}

trait StringFromTempfileStart {
    fn from_temp_start(tmpfile: &mut File) -> String;
}

impl StringFromTempfileStart for String {
    fn from_temp_start(tmpfile: &mut File) -> String {
        let mut stdout = String::new();
        tmpfile.seek(SeekFrom::Start(0)).ok();
        tmpfile.read_to_string(&mut stdout).ok();
        stdout
    }
}

#[derive(Debug)]
struct Summary {
    successes: u32,
    failures: Vec<u32>,
}

impl Summary {
    fn print(self) {
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
