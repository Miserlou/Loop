mod loop_step;
mod setup;
mod state;
mod util;

use loop_step::{loop_step, Env, ShellCommand};
use setup::{setup, Opt};
use state::{Counters, State, Summary};

use std::fs::File;
use std::process;

use subprocess::{Exec, ExitStatus, Redirection};

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
    let shell_command: &ShellCommand = &RealShellCommand {};

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
            shell_command,
        ) {
            break;
        }
    }

    pre_exit_tasks(opt.only_last, opt.summary, state.summary, state.tmpfile);

    process::exit(state.exit_status);
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

fn pre_exit_tasks(only_last: bool, print_summary: bool, summary: Summary, mut tmpfile: File) {
    use util::StringFromTempfileStart;

    if only_last {
        String::from_temp_start(&mut tmpfile)
            .lines()
            .for_each(|line| println!("{}", line));
    }

    if print_summary {
        summary.print()
    }
}

struct RealEnv {}

impl Env for RealEnv {
    fn set_var(&self, k: &str, v: &str) {
        std::env::set_var(k, v);
    }
}

struct RealShellCommand {}

impl ShellCommand for RealShellCommand {
    fn run(&self, state: &mut State, cmd_with_args: &str) -> ExitStatus {
        use std::io::{prelude::*, SeekFrom};

        state.tmpfile.seek(SeekFrom::Start(0)).ok();
        state.tmpfile.set_len(0).ok();

        Exec::shell(cmd_with_args)
            .stdout(Redirection::File(state.tmpfile.try_clone().unwrap()))
            .stderr(Redirection::Merge)
            .capture()
            .unwrap()
            .exit_status
    }
}
