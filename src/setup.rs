use crate::app::App;
use crate::io::{ExitCode, Printer, SetupEnv, ShellCommand};
use crate::loop_iterator::LoopIterator;
use crate::loop_step::LoopModel;

use std::time::{Duration, Instant, SystemTime};

use humantime::{parse_duration, parse_rfc3339_weak};
use regex::Regex;
use structopt::StructOpt;

pub fn setup(mut opt: Opt) -> Result<(App, SetupEnv, ShellCommand, Printer), AppError> {
    // Time
    let program_start = Instant::now();
    let cmd_with_args = opt.input.join(" ");

    if cmd_with_args.is_empty() {
        return Err(AppError::new(
            ExitCode::MinorError,
            "No command supplied, exiting.",
        ));
    }

    let printer = Printer {
        only_last: opt.only_last,
        until_contains: opt.until_contains.clone(),
        until_match: opt.until_match.clone(),
        summary: opt.summary,
        last_output: String::new(),
    };

    let setup_env = SetupEnv {
        count_precision: Opt::clap()
            .get_matches()
            .value_of("count_by")
            .map(precision_of)
            .unwrap_or(0),
    };

    let shell_command = ShellCommand { cmd_with_args };

    let app = App {
        every: opt.every,
        iterator: LoopIterator::from(&mut opt),
        loop_model: LoopModel::from_opt(opt, program_start),
    };

    Ok((app, setup_env, shell_command, printer))
}

#[derive(Debug, PartialEq, Clone)]
pub struct AppError {
    pub exit_code: ExitCode,
    pub message: String,
}

impl AppError {
    pub fn new<M>(exit_code: ExitCode, msg: M) -> AppError
    where
        M: Into<String>,
    {
        AppError {
            exit_code,
            message: msg.into(),
        }
    }
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

fn get_exit_code(input: &str) -> ExitCode {
    input
        .parse::<u32>()
        .map(ExitCode::from)
        .unwrap_or_else(|_| ExitCode::Error)
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
    num: Option<f64>,

    /// Amount to increment the counter by
    #[structopt(short = "b", long = "count-by", default_value = "1")]
    count_by: f64,

    /// Amount to offset the initial counter by
    #[structopt(short = "o", long = "offset", default_value = "0")]
    offset: f64,

    /// How often to iterate. ex., 5s, 1h1m1s1ms1us
    #[structopt(short = "e", long = "every", parse(try_from_str = "parse_duration"))]
    every: Option<Duration>,

    /// A comma-separated list of values, placed into 4ITEM. ex., red,green,blue
    #[structopt(long = "for", parse(from_str = "get_values"))]
    ffor: Option<Vec<String>>,

    /// Keep going until the duration has elapsed (example 1m30s)
    #[structopt(
        short = "d",
        long = "for-duration",
        parse(try_from_str = "parse_duration")
    )]
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
    #[structopt(
        short = "t",
        long = "until-time",
        parse(try_from_str = "parse_rfc3339_weak")
    )]
    until_time: Option<SystemTime>,

    /// Keep going until the command exit status is non-zero, or the value given
    #[structopt(short = "r", long = "until-error", parse(from_str = "get_exit_code"))]
    until_error: Option<ExitCode>,

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

    /// The command to be looped
    #[structopt(raw(multiple = "true"))]
    input: Vec<String>,
}

impl Default for Opt {
    fn default() -> Opt {
        Opt {
            num: None,
            count_by: 1_f64,
            offset: 0_f64,
            every: None,
            ffor: None,
            for_duration: None,
            until_contains: None,
            until_changes: false,
            until_same: false,
            until_match: None,
            until_time: None,
            until_error: None,
            until_success: false,
            until_fail: false,
            only_last: false,
            stdin: false,
            error_duration: false,
            summary: false,
            input: vec![],
        }
    }
}

impl From<&mut Opt> for LoopIterator {
    fn from(opt: &mut Opt) -> LoopIterator {
        use std::io::{self, BufRead};
        use std::mem;

        // Number of iterations
        let mut items: Vec<String> = vec![];

        if let Some(ref mut v) = opt.ffor {
            mem::swap(&mut items, v);
            opt.ffor = None;
        }

        // Get any lines from stdin
        if opt.stdin || atty::isnt(atty::Stream::Stdin) {
            io::stdin()
                .lock()
                .lines()
                .map(|line| line.unwrap().to_owned())
                .for_each(|line| items.push(line));
        }

        LoopIterator::new(opt.offset, opt.count_by, opt.num, items)
    }
}

impl LoopModel {
    fn from_opt(opt: Opt, program_start: Instant) -> LoopModel {
        LoopModel {
            program_start,
            for_duration: opt.for_duration,
            error_duration: opt.error_duration,
            until_time: opt.until_time,
            until_error: opt.until_error,
            until_success: opt.until_success,
            until_fail: opt.until_fail,
            summary: opt.summary,
            until_changes: opt.until_changes,
            until_same: opt.until_same,
        }
    }
}

#[test]
#[allow(non_snake_case)]
fn setup__okay() {
    let mut opt = Opt::default();
    opt.input = vec!["foobar".to_owned()];
    assert!(setup(opt).is_ok());
}

#[test]
#[allow(non_snake_case)]
fn setup__no_command() {
    // no command supplied to loop-rs
    let opt = Opt::default();
    let app_error = AppError::new(ExitCode::MinorError, "No command supplied, exiting.");
    match setup(opt) {
        Err(err) => assert_eq!(err, app_error),
        _ => panic!(),
    }
}

#[test]
fn test_get_exit_code() {
    assert_eq!(ExitCode::Okay, get_exit_code("0"));
    assert_eq!(ExitCode::Error, get_exit_code(""));
    assert_eq!(ExitCode::Timeout, get_exit_code("124"));
    assert_eq!(ExitCode::Other(50), get_exit_code("50"));
}