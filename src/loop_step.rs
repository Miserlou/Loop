use crate::setup::ErrorCode;
use crate::state::{Counters, State};
use crate::util::StringFromTempfileStart;

use std::time::{Duration, Instant, SystemTime};

use subprocess::ExitStatus;

/// same exit-code as used by the `timeout` shell command
static TIMEOUT_EXIT_CODE: i32 = 124;

pub struct LoopModel<'a> {
    pub for_duration: Option<Duration>,
    pub error_duration: bool,
    pub until_time: Option<SystemTime>,
    pub until_error: Option<ErrorCode>,
    pub until_success: bool,
    pub until_fail: bool,
    pub summary: bool,
    pub until_changes: bool,
    pub until_same: bool,
    pub every: Duration,

    pub cmd_with_args: String,
    pub program_start: Instant,
    pub items: Vec<String>,

    pub env: &'a dyn Env,
    pub shell_command: &'a dyn ShellCommand,
    pub result_printer: &'a dyn ResultPrinter,
}

impl<'a> LoopModel<'a> {
    pub fn step(&self, state: &mut State, counters: Counters) -> bool {
        use std::thread;

        // Time Start
        let loop_start = Instant::now();

        let env = self.env;
        let shell_command = self.shell_command;
        let result_printer = self.result_printer;

        // Set counters before execution
        // THESE ARE FLIPPED AND I CAN'T UNFLIP THEM.
        env.set_var("ACTUALCOUNT", &counters.index.to_string());
        env.set_var(
            "COUNT",
            &format!("{:.*}", counters.count_precision, counters.actual_count),
        );

        // Set iterated item as environment variable
        if let Some(item) = self.items.get(counters.index) {
            env.set_var("ITEM", item);
        }

        // Finish if we're over our duration
        if let Some(duration) = self.for_duration {
            let since = Instant::now().duration_since(self.program_start);
            if since >= duration {
                if self.error_duration {
                    state.exit_status = TIMEOUT_EXIT_CODE
                }
                return true;
            }
        }

        // Finish if our time until has passed
        // In this location, the loop will execute at least once,
        // even if the start time is beyond the until time.
        if let Some(until_time) = self.until_time {
            if SystemTime::now().duration_since(until_time).is_ok() {
                return true;
            }
        }

        // Main executor
        let exit_status = shell_command.run(state, &self.cmd_with_args);

        // Print the results
        let stdout = String::from_temp_start(&mut state.tmpfile);
        result_printer.print_and_mutate(state, &stdout);

        // --until-error
        check_for_error(self.until_error, &mut state.has_matched, exit_status);

        // --until-success
        if self.until_success && exit_status.success() {
            state.has_matched = true;
        }

        // --until-fail
        if self.until_fail && !(exit_status.success()) {
            state.has_matched = true;
        }

        if self.summary {
            state.summary_exit_status(exit_status);
        }

        // Finish if we matched
        if state.has_matched {
            return true;
        }

        if let Some(ref previous_stdout) = state.previous_stdout {
            // --until-changes
            if self.until_changes && *previous_stdout != stdout {
                return true;
            }

            // --until-same
            if self.until_same && *previous_stdout == stdout {
                return true;
            }
        }
        state.previous_stdout = Some(stdout);

        // Delay until next iteration time
        let since = Instant::now().duration_since(loop_start);
        if let Some(time) = self.every.checked_sub(since) {
            thread::sleep(time);
        }

        false
    }
}

fn check_for_error(
    maybe_error: Option<ErrorCode>,
    has_matched: &mut bool,
    exit_status: ExitStatus,
) {
    match maybe_error {
        Some(ErrorCode::Any) => {
            *has_matched = !exit_status.success();
        }
        Some(ErrorCode::Code(code)) => {
            if exit_status == ExitStatus::Exited(code) {
                *has_matched = true;
            }
        }
        _ => (),
    }
}

pub trait Env {
    fn set_var(&self, k: &str, v: &str);
}

pub trait ShellCommand {
    fn run(&self, state: &mut State, cmd_with_args: &str) -> ExitStatus;
}

pub trait ResultPrinter {
    fn print_and_mutate(&self, state: &mut State, stdout: &str);
}
