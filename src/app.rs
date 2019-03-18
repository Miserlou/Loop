use crate::io::ExitCode;
use crate::loop_iterator::LoopIterator;
use crate::loop_step::LoopModel;
use crate::state::{State, Summary};

use std::fs::File;
use std::time::{Duration, Instant};

pub struct App {
    pub every: Duration,
    pub iterator: LoopIterator,
    pub loop_model: LoopModel,
}

impl App {
    #[must_use]
    pub fn run(
        self,
        printer: &impl Fn(&str, State) -> State,
        command: &impl Fn(State) -> (ExitCode, State),
        exit_tasks: &impl Fn(Summary, File),
        setup_environment: &impl Fn(Option<String>, f64, f64),
    ) -> ExitCode {
        let loop_model = self.loop_model;
        let mut state = State::default();

        for it in self.iterator {
            let step_start_time = Instant::now();

            let setup_envs = || setup_environment(it.item.clone(), it.index, it.actual_count);

            let (break_loop, new_state) = loop_model.step(state, setup_envs, command, printer);
            state = new_state;

            if break_loop {
                break;
            }

            // Delay until next iteration time
            maybe_sleep(step_start_time, self.every);
        }

        exit_tasks(state.summary, state.tmpfile);

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
