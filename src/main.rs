mod io;
mod loop_iterator;
mod loop_step;
mod setup;
mod state;
mod util;

static EXIT_CODE_MINOR_ERROR: i32 = 2;

fn main() {
    use io::pre_exit_tasks;
    use setup::setup;
    use state::{Counters, State};
    use std::process;

    let m = setup();

    if m.is_no_command_supplied {
        eprintln!("No command supplied, exiting.");
        process::exit(EXIT_CODE_MINOR_ERROR);
    }

    let mut state = State::default();
    let loop_model = m.loop_model;

    for (index, actual_count) in m.iterator.enumerate() {
        let counters = Counters {
            count_precision: m.count_precision,
            index,
            actual_count,
        };

        let (break_loop, new_state) =
            loop_model.step(state, counters, &m.env, &m.shell_command, &m.result_printer);

        state = new_state;

        if break_loop {
            break;
        }
    }

    pre_exit_tasks(m.opt_only_last, m.opt_summary, state.summary, state.tmpfile);

    process::exit(state.exit_status);
}
