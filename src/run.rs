use crate::setup::Setup;
use crate::state::ExitCode;

pub fn run(m: Setup) -> ExitCode {
    use crate::io::pre_exit_tasks;
    use crate::state::{Counters, State};

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

    state.exit_code
}
