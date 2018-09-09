#[macro_use]
extern crate structopt;
extern crate humantime;
extern crate isatty;
extern crate regex;
extern crate subprocess;

use std::env;
use std::f64;
use std::io::{self, BufRead};
use std::thread;
use std::time::{Duration, Instant, SystemTime};

use humantime::{parse_duration, parse_rfc3339_weak};
use isatty::{stdin_isatty};
use regex::Regex;
use subprocess::{Exec, ExitStatus, Redirection};
use structopt::StructOpt;

static UNKONWN_EXIT_CODE: u32 = 99;

fn main() {

    // Load the CLI arguments
    let opt = Opt::from_args();

    // Time
    let program_start = Instant::now();

    // Number of iterations
    let mut items = if let Some(items) = opt.ffor { items.clone() } else { vec![] };

    // Get any lines from stdin
    if opt.stdin || !stdin_isatty() {
        let stdin = io::stdin();
        for line in stdin.lock().lines() {
            items.push(line.unwrap().to_owned())
        }
    }

    // Counters

    let num = if let Some(num) = opt.num {
        num
    } else if !items.is_empty() {
        items.len() as f64
    } else {
        f64::INFINITY
    };

    let mut has_matched = false;

    let mut summary = Summary { successes: 0, failures: Vec::new() };

    let counter = Counter { start: 0.0, end: num, step_by: opt.count_by};
    for (count, actual_count) in counter.enumerate() {

        // Time Start
        let loop_start = Instant::now();

        // Set counters before execution
        // THESE ARE FLIPPED AND I CAN'T UNFLIP THEM.
        env::set_var("ACTUALCOUNT", count.to_string());
        env::set_var("COUNT", actual_count.to_string());

        // Set iterated item as environment variable
        if let Some(item) = items.get(count) {
            env::set_var("ITEM", item);
        }

        // Main executor
        let result = Exec::shell(&opt.input)
            .stdout(Redirection::Pipe)
            .stderr(Redirection::Merge)
            .capture().unwrap();

        // Print the results
        for line in result.stdout_str().lines() {
            println!("{}", line);

            // --until-contains
            // We defer loop breaking until the entire result is printed.
            if let Some(string) = &opt.until_contains {
                if line.contains(string){
                    has_matched = true;
                }
            }

            // --until-match
            if let Some(regex) = &opt.until_match {
                if regex.captures(&line).is_some() {
                    has_matched = true;
                }
            }

            // --until-error
            if let Some(error_code) = &opt.until_error {
                match error_code {
                    ErrorCode::Any => if !result.exit_status.success() {
                        has_matched = true;
                    },
                    ErrorCode::Code(code) =>  {
                        if result.exit_status == ExitStatus::Exited(*code) {
                            has_matched = true;
                        }
                    }
                }
            }

            // --until-success
            if opt.until_success && result.exit_status.success() {
                    has_matched = true;
            }
        }

        if opt.summary {
            match result.exit_status {
                ExitStatus::Exited(0)  =>  summary.successes += 1,
                ExitStatus::Exited(n) => summary.failures.push(n),
                _ => summary.failures.push(UNKONWN_EXIT_CODE),
            }
        }

        // Finish if we matched
        if has_matched {
            break;
        }

        // Finish if we're over our duration
        if let Some(duration) = opt.for_duration {
            let since = Instant::now().duration_since(program_start);
            if since >= duration {
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

        // Delay until next iteration time
        let since = Instant::now().duration_since(loop_start);
        if let Some(time) = opt.every.checked_sub(since) {
            thread::sleep(time);
        }
    }

    if opt.summary {
        summary.print()
    }
}

#[derive(StructOpt, Debug)]
#[structopt(name = "loop", author = "Rich Jones <miserlou@gmail.com>",
            about = "UNIX's missing `loop` command")]
struct Opt {
    /// The command to be looped
    #[structopt()]
    input: String,

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
    #[structopt(short = "f", long = "for", parse(from_str = "get_values"))]
    ffor: Option<Vec<String>>,

    /// Keep going until the output contains this string
    #[structopt(short = "d", long = "for-duration", parse(try_from_str = "parse_duration"))]
    for_duration: Option<Duration>,

    /// Keep going until the output contains this string
    #[structopt(short = "c", long = "until-contains")]
    until_contains: Option<String>,

    /// Keep going until the output matches this regular expression
    #[structopt(short = "m", long = "until-match", parse(try_from_str = "Regex::new"))]
    until_match: Option<Regex>,

    /// Keep going until a future time, ex. "2018-04-20 04:20:00" (Times in UTC.)
    #[structopt(short = "t", long = "until-time", parse(try_from_str = "parse_rfc3339_weak"))]
    until_time: Option<SystemTime>,

    /// Keep going until the command exit status is non-zero, or the value given
    #[structopt(short = "r", long = "until-error", parse(from_str = "get_error_code"))]
    until_error: Option<ErrorCode>,

    /// Keep going until the command exit status is non-zero, or the value given
    #[structopt(short = "s", long = "until-success")]
    until_success: bool,

    /// Read from standard input
    #[structopt(short = "i", long = "stdin")]
    stdin: bool,

    /// Provide a summary
    #[structopt(long = "summary")]
    summary: bool
}

#[derive(Debug)]
enum ErrorCode {
    Any,
    Code(u32),
}

fn get_error_code(input: &&str) -> ErrorCode {
    if let Ok(code) = input.parse::<u32>() {
        ErrorCode::Code(code)
    } else {
        ErrorCode::Any
    }
}

fn get_values(input: &&str) -> Vec<String> {
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
    end: f64,
    step_by: f64,
}

#[derive(Debug)]
struct Summary {
    successes: u32,
    failures: Vec<u32>
}

impl Summary {
    fn print(self) {
        let total = self.successes + self.failures.len() as u32;

        let errors = if self.failures.is_empty() {
            String::from("0")
        } else {
            format!("{} ({})", self.failures.len(), self.failures.into_iter()
                    .map(|f| (-(f as i32)).to_string())
                    .collect::<Vec<String>>()
                    .join(", "))
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
        if self.start <= self.end {
            Some(self.start)
        } else {
            None
        }
    }
}
