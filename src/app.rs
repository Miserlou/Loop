use crate::io::{ExitCode, RealEnv, RealResultPrinter, RealShellCommand};
use crate::loop_iterator::LoopIterator;
use crate::loop_step::LoopModel;
use crate::state::Summary;

use std::fs::File;

#[derive(Debug)]
pub struct App {
    pub count_precision: usize,
    pub opt_only_last: bool,
    pub opt_summary: bool,

    pub env: RealEnv,
    pub shell_command: RealShellCommand,
    pub result_printer: RealResultPrinter,

    pub iterator: LoopIterator,
    pub loop_model: LoopModel,
}

impl App {
    #[must_use]
    pub fn run(self) -> ExitCode {
        // use crate::io::pre_exit_tasks;
        use crate::state::{Counters, State};

        let mut state = State::default();
        let loop_model = self.loop_model;

        for (index, actual_count) in self.iterator.enumerate() {
            let counters = Counters {
                count_precision: self.count_precision,
                index,
                actual_count,
            };

            let (break_loop, new_state) = loop_model.step(
                state,
                counters,
                &self.env,
                &self.shell_command,
                &self.result_printer,
            );

            state = new_state;

            if break_loop {
                break;
            }
        }

        pre_exit_tasks(
            self.opt_only_last,
            self.opt_summary,
            state.summary,
            state.tmpfile,
        );

        state.exit_code
    }
}

pub fn pre_exit_tasks(only_last: bool, print_summary: bool, summary: Summary, mut tmpfile: File) {
    use crate::util::StringFromTempfileStart;

    if only_last {
        String::from_temp_start(&mut tmpfile)
            .lines()
            .for_each(|line| println!("{}", line));
    }

    if print_summary {
        summary.print()
    }
}
