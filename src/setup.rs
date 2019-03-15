use crate::loop_step::{Env, LoopModel, ResultPrinter, ShellCommand};

use regex::Regex;
use std::time::{Duration, Instant, SystemTime};

use humantime::{parse_duration, parse_rfc3339_weak};
use structopt::StructOpt;

pub fn setup() -> (Opt, Vec<String>, usize, Instant) {
    use std::io::{self, BufRead};

    // Time
    let program_start = Instant::now();

    // Load the CLI arguments
    let opt = Opt::from_args();
    let count_precision = Opt::clap()
        .get_matches()
        .value_of("count_by")
        .map(precision_of)
        .unwrap_or(0);

    // Number of iterations
    let mut items: Vec<String> = opt.ffor.clone().unwrap_or_else(|| vec![]);

    // Get any lines from stdin
    if opt.stdin || atty::isnt(atty::Stream::Stdin) {
        io::stdin()
            .lock()
            .lines()
            .map(|line| line.unwrap().to_owned())
            .for_each(|line| items.push(line));
    }

    (opt, items, count_precision, program_start)
}

fn precision_of(s: &str) -> usize {
    let after_point = match s.find('.') {
        // '.' is ASCII so has len 1
        Some(point) => point + 1,
        None => return 0,
    };
    let exp = s.find(&['e', 'E'][..]).unwrap_or_else(|| s.len());
    exp - after_point
}

#[derive(Debug, Clone, Copy)]
pub enum ErrorCode {
    Any,
    Code(u32),
}

fn get_error_code(input: &str) -> ErrorCode {
    input
        .parse()
        .map(ErrorCode::Code)
        .unwrap_or_else(|_| ErrorCode::Any)
}

fn get_values(input: &str) -> Vec<String> {
    if input.contains('\n') {
        input.split('\n').map(String::from).collect()
    } else if input.contains(',') {
        input.split(',').map(String::from).collect()
    } else {
        input.split(' ').map(String::from).collect()
    }
}

#[derive(StructOpt, Debug)]
#[structopt(
    name = "loop",
    author = "Rich Jones <miserlou@gmail.com>",
    about = "UNIX's missing `loop` command"
)]
pub struct Opt {
    /// Number of iterations to execute
    #[structopt(short = "n", long = "num")]
    pub num: Option<f64>,

    /// Amount to increment the counter by
    #[structopt(short = "b", long = "count-by", default_value = "1")]
    pub count_by: f64,

    /// Amount to offset the initial counter by
    #[structopt(short = "o", long = "offset", default_value = "0")]
    pub offset: f64,

    /// How often to iterate. ex., 5s, 1h1m1s1ms1us
    #[structopt(
        short = "e",
        long = "every",
        default_value = "1us",
        parse(try_from_str = "parse_duration")
    )]
    pub every: Duration,

    /// A comma-separated list of values, placed into 4ITEM. ex., red,green,blue
    #[structopt(long = "for", parse(from_str = "get_values"))]
    pub ffor: Option<Vec<String>>,

    /// Keep going until the duration has elapsed (example 1m30s)
    #[structopt(
        short = "d",
        long = "for-duration",
        parse(try_from_str = "parse_duration")
    )]
    pub for_duration: Option<Duration>,

    /// Keep going until the output contains this string
    #[structopt(short = "c", long = "until-contains")]
    pub until_contains: Option<String>,

    /// Keep going until the output changes
    #[structopt(short = "C", long = "until-changes")]
    pub until_changes: bool,

    /// Keep going until the output changes
    #[structopt(short = "S", long = "until-same")]
    pub until_same: bool,

    /// Keep going until the output matches this regular expression
    #[structopt(short = "m", long = "until-match", parse(try_from_str = "Regex::new"))]
    pub until_match: Option<Regex>,

    /// Keep going until a future time, ex. "2018-04-20 04:20:00" (Times in UTC.)
    #[structopt(
        short = "t",
        long = "until-time",
        parse(try_from_str = "parse_rfc3339_weak")
    )]
    pub until_time: Option<SystemTime>,

    /// Keep going until the command exit status is non-zero, or the value given
    #[structopt(short = "r", long = "until-error", parse(from_str = "get_error_code"))]
    pub until_error: Option<ErrorCode>,

    /// Keep going until the command exit status is zero
    #[structopt(short = "s", long = "until-success")]
    pub until_success: bool,

    /// Keep going until the command exit status is non-zero
    #[structopt(short = "f", long = "until-fail")]
    pub until_fail: bool,

    /// Only print the output of the last execution of the command
    #[structopt(short = "l", long = "only-last")]
    pub only_last: bool,

    /// Read from standard input
    #[structopt(short = "i", long = "stdin")]
    pub stdin: bool,

    /// Exit with timeout error code on duration
    #[structopt(short = "D", long = "error-duration")]
    pub error_duration: bool,

    /// Provide a summary
    #[structopt(long = "summary")]
    pub summary: bool,

    /// The command to be looped
    #[structopt(raw(multiple = "true"))]
    pub input: Vec<String>,
}

impl<'a> Opt {
    pub fn into_loop_model(
        self,
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
            for_duration: self.for_duration,
            error_duration: self.error_duration,
            until_time: self.until_time,
            until_error: self.until_error,
            until_success: self.until_success,
            until_fail: self.until_fail,
            summary: self.summary,
            until_changes: self.until_changes,
            until_same: self.until_same,
            every: self.every,
        }
    }
}
