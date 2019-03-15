mod loop_step;
mod setup;
mod state;
mod util;

use loop_step::{Env, LoopModel, ResultPrinter, ShellCommand};
use setup::{setup, Opt};
use state::{Counters, State, Summary};

use std::fs::File;
use std::process;

use std::time::Instant;
use subprocess::{Exec, ExitStatus, Redirection};

fn main() {
    // Time
    let (opt, items, count_precision, program_start) = setup();

    let cmd_with_args = opt.input.join(" ");
    if cmd_with_args.is_empty() {
        println!("No command supplied, exiting.");
        return;
    }

    // Counters and State
    let mut state = State::default();
    let env: &dyn Env = &RealEnv {};
    let shell_command: &dyn ShellCommand = &RealShellCommand {};
    let result_printer: &dyn ResultPrinter = &RealResultPrinter { opt: &opt };

    let data_vec = counters_from_opt(&opt, &items);
    let loop_model = create_loop_model(
        &opt,
        cmd_with_args,
        program_start,
        items,
        env,
        shell_command,
        result_printer,
    );

    for (i, actual_count) in data_vec.iter().enumerate() {
        let counters = Counters {
            count_precision,
            index: i,
            actual_count: *actual_count,
        };

        let break_loop = loop_model.step(&mut state, counters);

        if break_loop {
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

struct RealResultPrinter<'a> {
    opt: &'a Opt,
}

impl<'a> ResultPrinter for RealResultPrinter<'a> {
    fn print_and_mutate(&self, state: &mut State, stdout: &str) {
        stdout.lines().for_each(|line| {
            // --only-last
            // If we only want output from the last execution,
            // defer printing until later
            if !self.opt.only_last {
                println!("{}", line);
            }

            // --until-contains
            // We defer loop breaking until the entire result is printed.
            if let Some(ref string) = self.opt.until_contains {
                state.has_matched = line.contains(string);
            }

            // --until-match
            if let Some(ref regex) = self.opt.until_match {
                state.has_matched = regex.captures(&line).is_some();
            }
        })
    }
}

fn create_loop_model<'a>(
    opt: &Opt,
    cmd_with_args: String,
    program_start: Instant,
    items: Vec<String>,
    env: &'a Env,
    shell_command: &'a ShellCommand,
    result_printer: &'a ResultPrinter,
) -> LoopModel<'a> {
    LoopModel {
        cmd_with_args,
        program_start,
        items,
        env,
        shell_command,
        result_printer,
        for_duration: opt.for_duration,
        error_duration: opt.error_duration,
        until_time: opt.until_time,
        until_error: opt.until_error,
        until_success: opt.until_success,
        until_fail: opt.until_fail,
        summary: opt.summary,
        until_changes: opt.until_changes,
        until_same: opt.until_same,
        every: opt.every,
    }
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
