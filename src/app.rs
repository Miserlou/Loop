use crate::io::ExitCode;
use crate::io::Printer;
use crate::loop_iterator::LoopIterator;
use crate::loop_step::LoopModel;
use crate::state::State;

use std::time::{Duration, Instant};

pub struct App {
    pub every: Option<Duration>,
    pub loop_model: LoopModel,
    pub iterator: LoopIterator,
}

impl App {
    #[must_use]
    pub fn run(
        self,
        setup_environment: &impl Fn(Option<String>, f64, f64),
        command: &impl Fn() -> (String, ExitCode),
        mut printer: Printer,
    ) -> State {
        let m = self.loop_model;
        let mut state = State::default();

        for it in self.iterator {
            let step_start_time = Instant::now();

            let is_last = it.is_last;

            let setup_envs =
                || setup_environment(it.item, it.actual_count, it.count);

            let (break_loop, new_state) =
                m.step(state, setup_envs, command, &mut printer);
            state = new_state;

            if break_loop {
                break;
            }

            if is_last {
                printer.terminatory_print(|| state.to_string());
            } else {
                // Delay until next iteration time
                if let Some(every) = self.every {
                    let since = Instant::now().duration_since(step_start_time);

                    if let Some(time) = every.checked_sub(since) {
                        std::thread::sleep(time);
                    }
                }
            }
        }

        state
    }
}

#[test]
#[allow(non_snake_case)]
fn run__num() {
    use std::cell::RefCell;

    // test that loop step method is called twice, flag: --num 2
    let expected_loop_count = 2;

    let app = App {
        every: None,
        iterator: {
            let num = Some(expected_loop_count as f64);
            let items =
                vec!["a", "b", "c"].into_iter().map(str::to_owned).collect();
            let offset = 0_f64;
            let count_by = 1_f64;
            LoopIterator::new(offset, count_by, num, items)
        },
        loop_model: LoopModel::default(),
    };

    let cmd_output = RefCell::new(vec![]);

    let state = app.run(
        &|_item, _index, _count| {},
        &|| {
            let mut buf = cmd_output.borrow_mut();
            let output = match buf.len() {
                0 => "123".to_owned(),
                _ => "abc".to_owned(),
            };
            buf.push(output.clone());

            (output, ExitCode::Okay)
        },
        Printer::default(),
    );
    assert_eq!(ExitCode::Okay, state.exit_code);
    assert_eq!(expected_loop_count, cmd_output.borrow().len());
    assert_eq!(vec!["123", "abc"], cmd_output.into_inner());
}
