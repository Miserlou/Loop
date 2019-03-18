use crate::io::ExitCode;
use crate::state::State;
use crate::util::StringFromTempfileStart;

use std::time::{Duration, Instant, SystemTime};

pub struct LoopModel {
    pub for_duration: Option<Duration>,
    pub error_duration: bool,
    pub until_time: Option<SystemTime>,
    pub until_error: Option<ExitCode>,
    pub until_success: bool,
    pub until_fail: bool,
    pub summary: bool,
    pub until_changes: bool,
    pub until_same: bool,
    pub program_start: Instant,
}

impl LoopModel {
    #[must_use]
    pub fn step(
        &self,
        mut state: State,
        setup_environment: impl Fn(),
        shell_command: impl Fn(State) -> (ExitCode, State),
        result_printer: impl Fn(&str, State) -> State,
    ) -> (bool, State) {
        // Set counters before execution
        setup_environment();

        // Finish if we're over our duration
        if let Some(duration) = self.for_duration {
            let since = Instant::now().duration_since(self.program_start);
            if since >= duration {
                if self.error_duration {
                    state.exit_code = ExitCode::Timeout;
                }
                return (true, state);
            }
        }

        // Finish if our time until has passed
        // In this location, the loop will execute at least once,
        // even if the start time is beyond the until time.
        if let Some(until_time) = self.until_time {
            if SystemTime::now().duration_since(until_time).is_ok() {
                return (true, state);
            }
        }

        // Main executor
        let (new_state, exit_code, stdout) = run_command(
            state,
            self.until_error,
            self.until_success,
            self.until_fail,
            shell_command,
            result_printer,
        );
        state = new_state;

        if self.summary {
            state.summary.update(exit_code);
        }

        // Finish if we matched
        if state.has_matched {
            return (true, state);
        }

        if let Some(ref previous_stdout) = state.previous_stdout {
            // --until-changes
            if self.until_changes && *previous_stdout != stdout {
                return (true, state);
            }

            // --until-same
            if self.until_same && *previous_stdout == stdout {
                return (true, state);
            }
        }
        state.previous_stdout = Some(stdout);

        (false, state)
    }
}

fn run_command(
    state: State,
    until_error: Option<ExitCode>,
    until_success: bool,
    until_fail: bool,
    shell_command: impl Fn(State) -> (ExitCode, State),
    result_printer: impl Fn(&str, State) -> State,
) -> (State, ExitCode, String) {
    let (exit_code, mut state) = shell_command(state);

    // Print the results
    let stdout = String::from_temp_start(&mut state.tmpfile);
    state = result_printer(&stdout, state);

    // --until-error
    until_error_check(&mut state.has_matched, until_error, exit_code);

    // --until-success
    state.has_matched = until_success && exit_code.success();

    // --until-fail
    state.has_matched = until_fail && !exit_code.success();

    (state, exit_code, stdout)
}

/// Check if the exit-code is non-zero, or the given exit-code.
fn until_error_check(has_matched: &mut bool, to_check: Option<ExitCode>, exit_code: ExitCode) {
    match to_check {
        Some(ExitCode::Error) => *has_matched = !exit_code.success(),
        Some(expected_exit_code) => {
            *has_matched = expected_exit_code == exit_code;
        }
        _ => (),
    }
}

impl Default for LoopModel {
    fn default() -> LoopModel {
        use std::time::Instant;

        LoopModel {
            for_duration: None,
            error_duration: false,
            until_time: None,
            until_error: None,
            until_success: false,
            until_fail: false,
            summary: false,
            until_changes: false,
            until_same: false,
            program_start: Instant::now(),
        }
    }
}
