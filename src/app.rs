use crate::io::ExitCode;
use crate::loop_iterator::LoopIterator;
use crate::loop_step::LoopModel;
use crate::state::{State, Summary};

use std::fs::File;
use std::time::{Duration, Instant};

pub struct App {
    pub every: Duration,
    pub loop_model: LoopModel,
    pub iterator: LoopIterator,
}

impl App {
    #[must_use]
    pub fn run(
        self,
        print: &impl Fn(&str, State) -> State,
        command: &impl Fn(State) -> (ExitCode, State),
        exit_tasks: &impl Fn(Summary, File),
        setup_environment: &impl Fn(Option<String>, f64, f64),
    ) -> ExitCode {
        let loop_model = self.loop_model;
        let mut state = State::default();

        for it in self.iterator {
            let step_start_time = Instant::now();

            let setup_envs = || setup_environment(it.item.clone(), it.index, it.actual_count);

            let (break_loop, new_state) = loop_model.step(state, setup_envs, command, print);
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

#[test]
fn test_run() {
    use humantime::parse_duration;
    use std::cell::RefCell;

    // test that the print closure is called twice
    let expected_loop_count = 2;
    let counter = RefCell::new(0);

    let app = App {
        every: parse_duration("1us").unwrap(),
        iterator: {
            let num = Some(expected_loop_count as f64);
            let items = vec!["a", "b", "c"].into_iter().map(str::to_owned).collect();
            let offset = 0_f64;
            let count_by = 1_f64;
            LoopIterator::new(offset, count_by, num, items)
        },
        loop_model: LoopModel::default(),
    };

    let exit_code = app.run(
        &|_stdout: &str, state: State| -> State {
            let mut my_ref = counter.borrow_mut();
            *my_ref += 1;
            state
        },
        &|state: State| (ExitCode::Okay, state),
        &|_summary: Summary, _tmpfile: File| {},
        &|_item: Option<String>, _index: f64, _actual_count: f64| {},
    );
    assert_eq!(ExitCode::Okay, exit_code);
    assert_eq!(expected_loop_count, *counter.borrow());
}
