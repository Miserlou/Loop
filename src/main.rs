mod loop_step;
mod setup;
mod state;
mod util;

use loop_step::{loop_step, Env};
use setup::{setup, Opt};
use state::{Counters, State, Summary};

use std::fs::File;
use std::process;

fn main() {
    // Time
    let (opt, items, count_precision, program_start) = setup();

    let cmd_with_args = &opt.input.join(" ");
    if cmd_with_args.is_empty() {
        println!("No command supplied, exiting.");
        return;
    }

    // Counters and State
    let mut state = State::default();
    let env: &Env = &RealEnv {};

    for (i, actual_count) in counters_from_opt(&opt, &items).iter().enumerate() {
        let counters = Counters {
            count_precision,
            index: i,
            actual_count: *actual_count,
        };
        if loop_step(
            &mut state,
            &opt,
            &items,
            cmd_with_args,
            counters,
            program_start,
            env,
        ) {
            break;
        }
    }

    exit_app(
        opt.only_last,
        opt.summary,
        state.exit_status,
        state.summary,
        state.tmpfile,
    )
}

fn counters_from_opt(opt: &Opt, items: &[String]) -> Vec<f64> {
    let mut counters = vec![];
    let mut start = opt.offset - opt.count_by;
    let mut index = 0_f64;
    let step_by = opt.count_by;
    let end = if let Some(num) = opt.num {
        num
    } else if !items.is_empty() {
        items.len() as f64
    } else {
        std::f64::INFINITY
    };
    loop {
        start += step_by;
        index += 1_f64;
        if index <= end {
            counters.push(start)
        } else {
            break;
        }
    }
    counters
}

fn exit_app(
    only_last: bool,
    print_summary: bool,
    exit_status: i32,
    summary: Summary,
    mut tmpfile: File,
) {
    use util::StringFromTempfileStart;

    if only_last {
        String::from_temp_start(&mut tmpfile)
            .lines()
            .for_each(|line| println!("{}", line));
    }

    if print_summary {
        summary.print()
    }

    process::exit(exit_status);
}

struct RealEnv {}

impl Env for RealEnv {
    fn set_var(&self, k: &str, v: &str) {
        std::env::set_var(k, v);
    }
}
