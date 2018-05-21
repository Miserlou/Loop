#[macro_use]
extern crate clap;
extern crate humantime;
extern crate subprocess;

use std::env;
use std::f64;
use std::io::{BufReader, BufRead};
use std::time::{Instant};

use clap::App;
use humantime::parse_duration;
use subprocess::{Exec, ExitStatus};

fn main() {

    // Load the CLI
    let yaml = load_yaml!("cli.yml");
    let matches = App::from_yaml(yaml).get_matches();

    // Main input command
    let input = matches.value_of("INPUT").unwrap();
    let input_s: String = input.to_string();

    // Number of iterations
    let mut num = matches.value_of("num").unwrap_or("-1").parse::<f64>().unwrap();
    if num < 0.0{
        num = f64::INFINITY;
    }

    // Get items in `--for`
    let mut items: Vec<&str> = Vec::new();
    match matches.value_of("for") {
        Some(x) => {
            if x.contains("\n"){
                items = x.split("\n").collect();
            } else if x.contains(","){
                items = x.split(",").collect();
            }
            else{
                items = x.split(",").collect();
            }
            num = items.len() as f64;
        },
        None => {}
    }

    // Amount to increment counter by
    let mut count_by = matches.value_of("count_by").unwrap_or("1").parse::<f64>().unwrap();

    // Counter offset
    let mut offset = matches.value_of("offset").unwrap_or("0").parse::<f64>().unwrap();

    // Delay time
    let every = parse_duration(matches.value_of("every").unwrap_or("1us")).unwrap();

    // --until-*
    let mut has_matched = false;
    let mut has_until_contains = false;
    let mut until_contains = "";
    if matches.is_present("until_contains"){
        has_until_contains = true;
        until_contains = matches.value_of("until_contains").unwrap();
    }

    // Counters
    let mut count = 0.0;
    let mut adjusted_count = 0.0 + offset;

    // Time
    let mut start = Instant::now();
    let mut now = Instant::now();
    let mut since;

    // Executor/readers
    let mut executor;
    let mut buf_reader;
    let mut line;

    while count < num {

        // Time Start
        start = Instant::now();

        // Set counters before execution
        env::set_var("COUNT", adjusted_count.to_string());
        env::set_var("ACTUALCOUNT", (count as i64).to_string());

        // Get iterated item
        match items.get(count as usize){
            Some(item) => { env::set_var("ITEM", item); },
            None => {}
        }

        // Main executor
        executor = Exec::shell(&input_s).stream_stdout().unwrap();
        buf_reader = BufReader::new(executor);

        // Print the results
        for (_i, rline) in buf_reader.lines().enumerate() {
            line = rline.unwrap();
            println!("{}", line);
            if has_until_contains{
                if line.contains(until_contains){
                    has_matched=true;
                }
            }
        }

        // Finish if we matched
        if has_matched {
            return;
        }

        // Increment counters
        count = count + 1.0;
        adjusted_count = adjusted_count + count_by;

        // Delay until next iteration time
        loop {
            now = Instant::now();
            since = now.duration_since(start);
            match every.checked_sub(since) {
                None => break,
                Some(time) => continue,
            }
        }
    }
}
