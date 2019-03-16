use crate::io::ExitCode;
use crate::setup::App;

#[must_use]
pub fn run(a: App) -> ExitCode {
    use crate::io::pre_exit_tasks;
    use crate::state::{Counters, State};

    let mut state = State::default();
    let loop_model = a.loop_model;

    for (index, actual_count) in a.iterator.enumerate() {
        let counters = Counters {
            count_precision: a.count_precision,
            index,
            actual_count,
        };

        let (break_loop, new_state) =
            loop_model.step(state, counters, &a.env, &a.shell_command, &a.result_printer);

        state = new_state;

        if break_loop {
            break;
        }
    }

    pre_exit_tasks(a.opt_only_last, a.opt_summary, state.summary, state.tmpfile);

    state.exit_code
}
