#[macro_use]
extern crate clap;
extern crate humantime;
extern crate regex;
extern crate subprocess;

use std::env;
use std::f64;
use std::io::{self, BufRead};
use std::process::Command;
use std::time::{Instant, SystemTime};

use clap::App;
use humantime::{parse_duration, parse_rfc3339_weak};
use regex::Regex;
use subprocess::{Exec, ExitStatus, Redirection};

fn main() {

    // Load the CLI
    let yaml = load_yaml!("cli.yml");
    let matches = App::from_yaml(yaml).get_matches();

    // Main input command
    let input = matches.value_of("INPUT").unwrap();
    let input_s: String = input.to_string();

    // Time
    let program_start = Instant::now();
    let mut loop_start = Instant::now();
    let mut now = Instant::now();
    let mut since;

	// Number of iterations
    let mut num = matches.value_of("num").unwrap_or("-1").parse::<f64>().unwrap();
    if num < 0.0{
        num = f64::INFINITY;
    }

    // Get items in `--for`
    let mut items: Vec<String> = Vec::new();
    match matches.value_of("for") {
        Some(x) => {
            if x.contains("\n"){
                items = x.split("\n").map(String::from).collect();
            } else if x.contains(","){
                items = x.split(",").map(String::from).collect();
            }
            else{
                items = x.split(" ").map(String::from).collect();
            }
            num = items.len() as f64;
        },
        None => {}
    }

    // Amount to increment counter by
    let count_by = matches.value_of("count_by").unwrap_or("1").parse::<f64>().unwrap();

    // Counter offset
    let offset = matches.value_of("offset").unwrap_or("0").parse::<f64>().unwrap();

    // Delay time
    let every = parse_duration(matches.value_of("every").unwrap_or("1us")).unwrap();

    // --until-contains
    let mut has_matched = false;
    let mut has_until_contains = false;
    let mut until_contains = "";
    if matches.is_present("until_contains"){
        has_until_contains = true;
        until_contains = matches.value_of("until_contains").unwrap();
    }

    // --until-match
    let mut has_until_match = false;
    let until_match_re;
    match matches.value_of("until_match") {
        Some(match_str) => {
            until_match_re = Regex::new(match_str).unwrap();
            has_until_match = true;
        },
        None => { until_match_re = Regex::new("").unwrap() }
    }

    // --until-time
    let mut has_until_time = false;
    let mut until_time = parse_rfc3339_weak("9999-01-01 01:01:01").unwrap();
    match matches.value_of("until_time") {
        Some(match_str) => {
            match parse_rfc3339_weak(match_str) {
                Ok(time) => { until_time = time; },
                // TODO here: Try to append current year and try again.
                Err(err) => { println!("Bad --until-time: {:?}", err); return; }
            }
            has_until_time = true;
        },
        None => {}
    }

    // --for-duration
    let has_for_duration;
    let mut for_duration = parse_duration("999999y").unwrap();
    match matches.value_of("for_duration") {
        Some(match_str) => {
            match parse_duration(match_str) {
                        Ok(duration) => { for_duration = duration;  },
                        Err(err) => { println!("Bad --for-duration: {:?}", err); return; }
            }
            has_for_duration = true;
        },
        None => { has_for_duration = false; }
    }

    // --until-error
    let mut has_until_error = false;
    let mut has_until_error_code = false;
    let mut until_error_code = 1;
    if matches.occurrences_of("until_error") > 0 {
        has_until_error = true;
        if matches.values_of("until_error").unwrap().next() != Some("any_error") {
            has_until_error_code = true;
            until_error_code = matches.value_of("until_error").unwrap_or("1").parse::<u32>().unwrap();
        }
    }

    // --until-success
    let mut has_until_success = false;
    if matches.occurrences_of("until_success") > 0 {
        has_until_success = true;
    }

    // Stdin
    let mut has_stdin = false;
    let mut full_stdin = "".to_string();
    if matches.occurrences_of("stdin") > 0 {
        has_stdin = true;
	    let stdin = io::stdin();
	    for linee in stdin.lock().lines() {
	        items.push(linee.unwrap().to_owned());
	    }

	    num = matches.value_of("num").unwrap_or(&items.len().to_string()).parse::<f64>().unwrap();
    }

    // Counters
    let mut count = 0.0;
    let mut adjusted_count = 0.0 + offset;
    let mut result;

    while count < num {

        // Time Start
        loop_start = Instant::now();

        // Set counters before execution
        env::set_var("COUNT", adjusted_count.to_string());
        env::set_var("ACTUALCOUNT", (count as i64).to_string());

        // Get iterated item
        match items.get(count as usize){
            Some(item) => { env::set_var("ITEM", item); },
            None => {}
        }

        // Main executor
        result = Exec::shell(&input_s).stdout(Redirection::Pipe).stderr(Redirection::Merge).capture().unwrap();

        // Print the results
        for line in result.stdout_str().lines() {
            println!("{}", line);

            // --until-contains
            // We defer loop breaking until the entire result is printed.
            if has_until_contains {
                if line.contains(until_contains){
                    has_matched=true;
                }
            }

            // --until-match
            if has_until_match {
                match until_match_re.captures(&line){
                    Some(_item) => { has_matched=true; }
                    None => {}
                }
            }

            // --until-error
            if has_until_error {
                if has_until_error_code {
                    // TODO: might want to print the error code when it doesn't match
                    if result.exit_status == ExitStatus::Exited(until_error_code) {
                        has_matched = true;
                    }
                } else if !result.exit_status.success() {
                    has_matched = true;
                }
            }

            // --until-success
            if has_until_success {
                if result.exit_status.success() {
                    has_matched = true;
                }
            }
        }

        // Finish if we matched
        if has_matched {
            break;
        }

        // Increment counters
        count = count + 1.0;
        adjusted_count = adjusted_count + count_by;

        // The main delay-until-next-iteration loop
        loop {

            // Finish if we're over our duration
            if has_for_duration {
                now = Instant::now();
                since = now.duration_since(program_start);
                match for_duration.checked_sub(since) {
                    None => return,
                    Some(_time) => { },
                }
            }

            // Finish if our time until has passed
            // In this location, the loop will execute at least once,
            // even if the start time is beyond the until time.
            if has_until_time{
                match SystemTime::now().duration_since(until_time) {
                    Ok(_t) => return,
                    Err(_e) => {  },
                }
            }

            // Delay until next iteration time
            now = Instant::now();
            since = now.duration_since(loop_start);
            match every.checked_sub(since) {
                None => break,
                Some(_time) => continue,
            }
        }
    }
}
