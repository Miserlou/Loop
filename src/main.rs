mod io;
mod loop_iterator;
mod loop_step;
mod setup;
mod state;
mod util;

use io::{pre_exit_tasks, RealEnv, RealResultPrinter, RealShellCommand};
use loop_iterator::LoopIterator;
use setup::setup;
use state::{Counters, State};

use std::process;

fn main() {
    // Time
    let (opt, items, count_precision, program_start) = setup();

    let opt_only_last = opt.only_last;
    let opt_summary = opt.summary;

    let cmd_with_args = opt.input.join(" ");
    if cmd_with_args.is_empty() {
        println!("No command supplied, exiting.");
        return;
    }

    // Counters and State
    let env = RealEnv {};
    let shell_command = RealShellCommand {};
    let result_printer = RealResultPrinter::new(
        opt.only_last,
        opt.until_contains.clone(),
        opt.until_match.clone(),
    );

    let iterator = LoopIterator::new(opt.offset, opt.count_by, opt.num, &items);
    let loop_model = opt.into_loop_model(cmd_with_args, program_start, items);

    let mut state = State::default();

    for (index, actual_count) in iterator.enumerate() {
        let counters = Counters {
            count_precision,
            index,
            actual_count,
        };

        let (break_loop, new_state) =
            loop_model.step(state, counters, &env, &shell_command, &result_printer);
        state = new_state;

        if break_loop {
            break;
        }
    }

    pre_exit_tasks(opt_only_last, opt_summary, state.summary, state.tmpfile);

    process::exit(state.exit_status);
}
