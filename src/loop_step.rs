use crate::io::{ExitCode, Printer};
use crate::state::State;

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
        setup_environment: impl FnOnce(),
        shell_command: impl Fn() -> (String, ExitCode),
        printer: &mut Printer,
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
        let (new_state, exit_code, cmd_output) = run_command(
            state,
            self.until_error,
            self.until_success,
            self.until_fail,
            shell_command,
            printer,
        );
        state = new_state;

        if self.summary {
            state.update_summary(exit_code);
        }

        // Finish if we matched
        if state.has_matched {
            return (true, state);
        }

        if let Some(ref previous_stdout) = state.previous_stdout {
            // --until-changes
            if self.until_changes && *previous_stdout != cmd_output {
                return (true, state);
            }

            // --until-same
            if self.until_same && *previous_stdout == cmd_output {
                return (true, state);
            }
        }
        state.previous_stdout = Some(cmd_output);

        (false, state)
    }
}

fn run_command(
    mut state: State,
    until_error: Option<ExitCode>,
    until_success: bool,
    until_fail: bool,
    shell_command: impl Fn() -> (String, ExitCode),
    printer: &mut Printer,
) -> (State, ExitCode, String) {
    let (cmd_output, exit_code) = shell_command();

    // Print the results
    printer.print(&cmd_output, &mut state);

    // --until-error
    state.has_matched = check_until_error(until_error, exit_code);

    // --until-success
    state.has_matched = until_success && exit_code.success();

    // --until-fail
    state.has_matched = until_fail && !exit_code.success();

    (state, exit_code, cmd_output)
}

/// Check `exit_code` if --until-error flag is set.
fn check_until_error(should_check: Option<ExitCode>, exit_code: ExitCode) -> bool {
    match should_check {
        Some(ExitCode::Other(expected)) => expected == exit_code.into(),
        Some(_) => !exit_code.success(),
        _ => false,
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

#[test]
fn test_check_until_error() {
    // check generic error
    let has_matched = check_until_error(Some(ExitCode::Error), ExitCode::MinorError);
    assert!(has_matched);

    // check specific error-code
    let has_matched = check_until_error(Some(ExitCode::Other(99)), ExitCode::Other(99));
    assert!(has_matched);

    // check specific error-code, no match
    let has_matched = check_until_error(Some(ExitCode::Other(20)), ExitCode::Other(99));
    assert!(!has_matched);

    // --until-error flag not set
    let has_matched = check_until_error(None, ExitCode::Okay);
    assert!(!has_matched);
}
