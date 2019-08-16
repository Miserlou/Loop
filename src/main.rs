extern crate structopt;
extern crate humantime;
extern crate atty;
extern crate regex;
extern crate subprocess;
extern crate tempfile;

use std::env;
use std::f64;
use std::io::prelude::*;
use std::io::{self, BufRead};
use std::io::{stdout, stderr};
use std::process;
use std::thread::{self, JoinHandle};
use std::sync::mpsc::{Sender, Receiver, channel};
use std::time::{Duration, Instant, SystemTime};
use subprocess::{ExitStatus, CaptureData};

use humantime::{parse_duration, parse_rfc3339_weak};
use regex::Regex;
use structopt::StructOpt;

mod supervisor;
mod evaluator;

use evaluator::{execute, Response};

// same exit code as use of `timeout` shell command
static TIMEOUT_EXIT_CODE: i32 = 124;
static UNKONWN_EXIT_CODE: u32 = 99;

fn main() {

    // Load the CLI arguments
    let opt = Opt::from_args();
    let count_precision = Opt::clap()
        .get_matches()
        .value_of("count_by")
        .map(precision_of)
        .unwrap_or(0);

    let mut exit_status = 0;

    // Time
    let program_start = Instant::now();

    // Number of iterations
    let mut items = if let Some(ref items) = opt.ffor { items.clone() } else { vec![] };

    // Get any lines from stdin
    if opt.stdin || atty::isnt(atty::Stream::Stdin) {
        let stdin = io::stdin();
        for line in stdin.lock().lines() {
            items.push(line.unwrap().to_owned())
        }
    }

    let joined_input = &opt.input.join(" ");
    if joined_input == "" {
        println!("No command supplied, exiting.");
        return;
    }

    // Counters and State
    let num = if let Some(num) = opt.num {
        num
    } else if !items.is_empty() {
        items.len() as f64
    } else {
        f64::INFINITY
    };

    let counter = Counter {
            start: opt.offset - opt.count_by,
            iters: 0.0,
            end: num,
            step_by: opt.count_by
    };

    let mut detach_data = if opt.detach {
        let (requests_in, requests_out) = channel();
        let (responses_in, responses_out) = channel();
        let supervisor_thread = thread::spawn(move || {
            supervisor::supervisor(requests_out, responses_in); 
        });
        Some(DetachData{
            supervisor: supervisor_thread,
            requests_in: requests_in,
            responses_out: responses_out
        })
    } else {
        None
    };

    let mut app_state = AppState{
        options: &opt,
        summary: Summary { successes: 0, failures: Vec::new() },
        previous_result: None,
    };

    for (count, actual_count) in counter.enumerate() {

        // Time Start
        let loop_start = Instant::now();

        // Set counters before execution
        // THESE ARE FLIPPED AND I CAN'T UNFLIP THEM.
        env::set_var("ACTUALCOUNT", count.to_string());
        env::set_var("COUNT", format!("{:.*}", count_precision, actual_count));

        // Set iterated item as environment variable
        if let Some(item) = items.get(count) {
            env::set_var("ITEM", item);
        }

        // Finish if we have result from detached thread
        if let Some(ref detach_data) = detach_data{
            if let Ok(response) = detach_data.responses_out.try_recv(){
                app_state = handle_output(&response, app_state);
                if response.should_stop(){
                    break;
                }
            }
        }

        // Finish if we're over our duration
        if let Some(duration) = opt.for_duration {
            let since = Instant::now().duration_since(program_start);
            if since >= duration {
                if opt.error_duration {
                    exit_status = TIMEOUT_EXIT_CODE
                }
                break;
            }
        }

        // Finish if our time until has passed
        // In this location, the loop will execute at least once,
        // even if the start time is beyond the until time.
        if let Some(until_time) = opt.until_time {
            if SystemTime::now().duration_since(until_time).is_ok() {
                break;
            }
        }

        // Main executor
        if opt.detach{
            let th_opt = opt.clone();
            let dd = detach_data.as_mut().unwrap();
            dd.requests_in.send((count, th_opt)).unwrap();
        } else {
            let (opt, result) = execute(opt.clone());

            // Finish if we matched
            let response = Response{
                options: opt,
                result: supervisor::clone_data(&result),
                last_result: match app_state.previous_result{
                    Some(ref lr) => Some(supervisor::clone_data(&lr)),
                    None => None,
                }
            };
            app_state = handle_output(&response, app_state);
            if response.should_stop(){
                break;
            }
        }

        // Delay until next iteration time
        let since = Instant::now().duration_since(loop_start);
        if let Some(time) = opt.every.checked_sub(since) {
            thread::sleep(time);
        }
    }

    if opt.only_last {
        if let Some(ref previous_result) = &app_state.previous_result {
            stdout().write_all(&previous_result.stdout).unwrap();
            stderr().write_all(&previous_result.stderr).unwrap();
        }
    }

    if opt.summary {
        app_state.summary.print()
    }
    if let Some(detach_data) = detach_data{
        detach_data.supervisor.join().unwrap();
    }
    process::exit(exit_status);
}

fn handle_output<'a>(response: &Response, mut state: AppState<'a>) -> AppState<'a> {
    if !state.options.only_last {
        stdout().write_all(&response.result.stdout).unwrap();
        stderr().write_all(&response.result.stderr).unwrap();
    }
    if state.options.summary {
        match response.result.exit_status {
            ExitStatus::Exited(0)  => state.summary.successes += 1,
            ExitStatus::Exited(n) => state.summary.failures.push(n),
            _ => state.summary.failures.push(UNKONWN_EXIT_CODE),
        }
    }
    AppState{
        previous_result: Some(supervisor::clone_data(&response.result)),
        ..state
    }
}

struct AppState <'a>{
    options: &'a Opt,
    summary: Summary,
    previous_result: Option<CaptureData>,
}


#[derive(StructOpt, Debug, Clone)]
#[structopt(name = "loop", author = "Rich Jones <miserlou@gmail.com>",
            about = "UNIX's missing `loop` command")]
pub struct Opt {
    /// Number of iterations to execute
    #[structopt(short = "n", long = "num")]
    num: Option<f64>,

    /// Amount to increment the counter by
    #[structopt(short = "b", long = "count-by", default_value = "1")]
    count_by: f64,

    /// Amount to offset the initial counter by
    #[structopt(short = "o", long = "offset", default_value = "0")]
    offset: f64,

    /// How often to iterate. ex., 5s, 1h1m1s1ms1us
    #[structopt(short = "e", long = "every", default_value = "1us",
                parse(try_from_str = "parse_duration"))]
    every: Duration,

    /// A comma-separated list of values, placed into 4ITEM. ex., red,green,blue
    #[structopt(long = "for", parse(from_str = "get_values"))]
    ffor: Option<Vec<String>>,

    /// Keep going until the duration has elapsed (example 1m30s)
    #[structopt(short = "d", long = "for-duration", parse(try_from_str = "parse_duration"))]
    for_duration: Option<Duration>,

    /// Keep going until the output contains this string
    #[structopt(short = "c", long = "until-contains")]
    until_contains: Option<String>,

    /// Keep going until the output changes
    #[structopt(short = "C", long = "until-changes")]
    until_changes: bool,

    /// Keep going until the output changes
    #[structopt(short = "S", long = "until-same")]
    until_same: bool,

    /// Keep going until the output matches this regular expression
    #[structopt(short = "m", long = "until-match", parse(try_from_str = "Regex::new"))]
    until_match: Option<Regex>,

    /// Keep going until a future time, ex. "2018-04-20 04:20:00" (Times in UTC.)
    #[structopt(short = "t", long = "until-time", parse(try_from_str = "parse_rfc3339_weak"))]
    until_time: Option<SystemTime>,

    /// Keep going until the command exit status is non-zero, or the value given
    #[structopt(short = "r", long = "until-error", parse(from_str = "get_error_code"))]
    until_error: Option<ErrorCode>,

    /// Keep going until the command exit status is zero
    #[structopt(short = "s", long = "until-success")]
    until_success: bool,

    /// Keep going until the command exit status is non-zero
    #[structopt(short = "f", long = "until-fail")]
    until_fail: bool,

    /// Only print the output of the last execution of the command
    #[structopt(short = "l", long = "only-last")]
    only_last: bool,

    /// Read from standard input
    #[structopt(short = "i", long = "stdin")]
    stdin: bool,

    /// Exit with timeout error code on duration
    #[structopt(short = "D", long = "error-duration")]
    error_duration: bool,

    /// Provide a summary
    #[structopt(long = "summary")]
    summary: bool,

    /// Do not be blocked by the running command
    #[structopt(long = "detach")]
    detach: bool,

    /// The command to be looped
    #[structopt(raw(multiple="true"))]
    input: Vec<String>

}

fn precision_of(s: &str) -> usize {
    let after_point = match s.find('.') {
        // '.' is ASCII so has len 1
        Some(point) => point + 1,
        None => return 0,
    };
    let exp = match s.find(&['e', 'E'][..]) {
        Some(exp) => exp,
        None => s.len(),
    };
    exp - after_point
}

#[derive(Debug, Clone)]
pub enum ErrorCode {
    Any,
    Code(u32),
}

pub fn get_error_code(input: &str) -> ErrorCode {
    if let Ok(code) = input.parse::<u32>() {
        ErrorCode::Code(code)
    } else {
        ErrorCode::Any
    }
}

fn get_values(input: &str) -> Vec<String> {
    if input.contains('\n'){
        input.split('\n').map(String::from).collect()
    } else if input.contains(','){
        input.split(',').map(String::from).collect()
    } else {
        input.split(' ').map(String::from).collect()
    }
}

struct Counter {
    start: f64,
    iters: f64,
    end: f64,
    step_by: f64,
}

#[derive(Debug)]
struct DetachData {
    supervisor: JoinHandle<()>,
    requests_in: Sender<(usize, Opt)>,
    responses_out: Receiver<supervisor::Response>
}

#[derive(Debug)]
pub struct Summary {
    successes: u32,
    failures: Vec<u32>
}

impl Summary {
    fn print(&self) {
        let total = self.successes + self.failures.len() as u32;

        let errors = if self.failures.is_empty() {
            String::from("0")
        } else {
            format!("{} (TODO)", self.failures.len())
           // format!("{} ({})", self.failures.len(), self.failures.into_iter()
           //         .map(|f| (-(f as i32)).to_string())
           //         .collect::<Vec<String>>()
           //         .join(", "))
        };

        println!("Total runs:\t{}", total);
        println!("Successes:\t{}", self.successes);
        println!("Failures:\t{}", errors);
    }
}

impl Iterator for Counter {
    type Item = f64;
    fn next(&mut self) -> Option<Self::Item> {
        self.start += self.step_by;
        self.iters += 1.0;
        if self.iters <= self.end {
            Some(self.start)
        } else {
            None
        }
    }
}
