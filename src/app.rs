use crate::io::{ExitCode, Printer};
use crate::loop_iterator::LoopIterator;
use crate::loop_step::LoopModel;
use crate::state::State;

use std::io::Write;
use std::time::{Duration, Instant};

pub struct App {
    pub every: Option<Duration>,
    pub loop_model: LoopModel,
    pub iterator: LoopIterator,
}

impl App {
    #[must_use]
    pub fn run<W: Write>(
        self,
        setup_environment: &impl Fn(Option<String>, f64, f64),
        command: &impl Fn() -> (String, ExitCode),
        printer: &mut Printer<W>,
    ) -> ExitCode {
        let mut state = State::default();

        for it in self.iterator {
            let step_start_time = Instant::now();

            let is_last = it.is_last;

            let setup_envs =
                || setup_environment(it.item, it.actual_count, it.count);

            let (break_loop, new_state) =
                self.loop_model.step(state, setup_envs, command, printer);
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

        state.exit_code
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fmt::{self, Debug, Formatter};

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
                let items = vec!["a", "b", "c"]
                    .into_iter()
                    .map(str::to_owned)
                    .collect();
                let offset = 0_f64;
                let count_by = 1_f64;
                LoopIterator::new(offset, count_by, num, items)
            },
            loop_model: LoopModel::default(),
        };

        let mut printer = Printer::default();
        let counter = RefCell::new(0);

        let exit_code = app.run(
            &|_item, _index, _count| {},
            &|| {
                let mut count = counter.borrow_mut();
                let output = match *count {
                    0 => "123".to_owned(),
                    _ => "abc".to_owned(),
                };
                *count += 1;

                (output, ExitCode::Okay)
            },
            &mut printer,
        );

        let inner_data = printer.into_inner();

        assert_eq!(ExitCode::Okay, exit_code);
        assert_eq!(expected_loop_count, *counter.borrow());
        assert_eq!(vec!["123", "abc"], inner_data);
    }

    impl Printer<Vec<u8>> {
        #[allow(dead_code)]
        pub fn into_inner(self) -> Vec<String> {
            String::from_utf8(self.w)
                .unwrap_or_default()
                .lines()
                .map(str::to_owned)
                .collect()
        }
    }

    impl<T> Debug for Printer<T>
    where
        T: Write + Debug,
    {
        fn fmt(&self, f: &mut Formatter) -> fmt::Result {
            write!(f, "{:?}", self)
        }
    }

    impl Default for Printer<Vec<u8>> {
        fn default() -> Printer<Vec<u8>> {
            Printer {
                only_last: false,
                until_contains: None,
                until_match: None,
                summary: false,
                last_output: String::default(),
                w: vec![],
            }
        }
    }
}
