use crate::setup::{ErrorCode, Opt};
use crate::state::{Counters, State};
use crate::util::StringFromTempfileStart;

use std::time::{Instant, SystemTime};

use subprocess::{Exec, ExitStatus, Redirection};

// same exit-code as used by the `timeout` shell command
static TIMEOUT_EXIT_CODE: i32 = 124;

pub fn loop_step(
    state: &mut State,
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
                state.exit_status = TIMEOUT_EXIT_CODE
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
    let exit_status = run_shell_command(state, cmd_with_args);

    // Print the results
    let stdout = String::from_temp_start(&mut state.tmpfile);
    state.print_results(&opt, &stdout);

    // --until-error
    check_for_error(&opt.until_error, &mut state.has_matched, exit_status);

    // --until-success
    if opt.until_success && exit_status.success() {
        state.has_matched = true;
    }

    // --until-fail
    if opt.until_fail && !(exit_status.success()) {
        state.has_matched = true;
    }

    if opt.summary {
        state.summary_exit_status(exit_status);
    }

    // Finish if we matched
    if state.has_matched {
        return true;
    }

    if let Some(ref previous_stdout) = state.previous_stdout {
        // --until-changes
        if opt.until_changes && *previous_stdout != stdout {
            return true;
        }

        // --until-same
        if opt.until_same && *previous_stdout == stdout {
            return true;
        }
    }
    state.previous_stdout = Some(stdout);

    // Delay until next iteration time
    let since = Instant::now().duration_since(loop_start);
    if let Some(time) = opt.every.checked_sub(since) {
        thread::sleep(time);
    }

    false
}

fn run_shell_command(state: &mut State, cmd_with_args: &str) -> ExitStatus {
    use std::io::{prelude::*, SeekFrom};

    state.tmpfile.seek(SeekFrom::Start(0)).ok();
    state.tmpfile.set_len(0).ok();

    Exec::shell(cmd_with_args)
        .stdout(Redirection::File(state.tmpfile.try_clone().unwrap()))
        .stderr(Redirection::Merge)
        .capture()
        .unwrap()
        .exit_status
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
