use crate::io::{ExitCode, PreExitTasks, Printer};
use crate::loop_iterator::LoopIterator;
use crate::loop_step::LoopModel;

use std::time::{Duration, Instant};

pub struct App {
    pub count_precision: usize,
    pub cmd_with_args: String,
    pub every: Duration,
    pub iterator: LoopIterator,
    pub loop_model: LoopModel,
    pub items: Vec<String>,
}

impl App {
    #[must_use]
    pub fn run(self, printer: Printer, exit_tasks: PreExitTasks) -> ExitCode {
        use crate::io::{setup_environment, shell_command};
        use crate::state::State;

        let mut state = State::default();
        let loop_model = self.loop_model;
        let cmd_with_args = self.cmd_with_args;

        let command = |state: State| -> (ExitCode, State) {
            // cmd_with_args doesn't change, keep it in the closue
            shell_command(&cmd_with_args, state)
        };
        let printer = |stdout: &str, state: State| -> State {
            // object of struct Printer doesn't change, keep it in the closue
            printer.print(stdout, state)
        };

        for (i, actual_count) in self.iterator.enumerate() {
            let step_start_time = Instant::now();
            let count_precision = self.count_precision;
            let item = self.items.get(i);

            let setup_envs = || setup_environment(item, i, count_precision, actual_count);

            let (break_loop, new_state) = loop_model.step(state, setup_envs, command, printer);
            state = new_state;

            if break_loop {
                break;
            }

            // Delay until next iteration time
            maybe_sleep(step_start_time, self.every);
        }

        exit_tasks.run(state.summary, state.tmpfile);

        state.exit_code
    }
}

fn maybe_sleep(step_start: Instant, every: Duration) {
    use std::thread;

    let since = Instant::now().duration_since(step_start);
    if let Some(time) = every.checked_sub(since) {
        thread::sleep(time);
    }
}
