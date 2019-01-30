![Logo, with help from Linn Kristiansen](https://i.imgur.com/TQp8nu3.png)

# loop [![Build Status](https://travis-ci.org/Miserlou/Loop.svg)](https://travis-ci.org/Miserlou/Loop) [![crates.io](https://img.shields.io/crates/v/loop-rs.svg)](https://crates.io/crates/loop-rs)

_"UNIX's missing `loop` command!"_

`loop` lets you write powerful, intuitive looping one-liners in your favorite shell! Finally, loops in Bash that make sense!

## Why?

Loops in bash are surprisingly complicated and fickle! I wanted a simple and intuitive way to write controllable loops that:

 * Run on controllable **timers**!
   - `$ loop 'ls' --every 10s`

 * Have **custom counters**!
   - `$ loop 'touch $COUNT.txt' --count-by 5`

 * Loop **until output matches** a condition!
   - `$ loop --until-contains 200 -- ./get_response_code.sh --site mysite.biz`

 * Loop **until output changes between invocations**!
   - `$ loop --until-changes date +%s`

 * Loop **until a certain time**!
   - `$ loop './poke_server' --for-duration 8h`

 * Loop **until a program succeeds** (or fails!)
    - `$ loop './poke_server' --until-success`

 * Iterate over the **standard input**!
    - `$ cat files_to_create.txt | loop 'touch $ITEM'`

 * Get a **summary** of the runs!
    - `$ loop 'ls' --for-duration 10min --summary`

 * ..and **much more!**

 And so `loop` was born!

<!-- START doctoc generated TOC please keep comment here to allow auto update -->
<!-- DON'T EDIT THIS SECTION, INSTEAD RE-RUN doctoc TO UPDATE -->

- [Installation](#installation)
  - [Linux](#linux)
  - [OSX](#osx)
  - [Rust Users](#rust-users)
    - [Building](#building)
- [Usage](#usage)
  - [Counters](#counters)
  - [Timed Loops](#timed-loops)
  - [Until Conditions](#until-conditions)
  - [Iterating Over Lists and Standard Inputs](#iterating-over-lists-and-standard-inputs)
- [Useful Examples](#useful-examples)
  - [Testing inputs to a program](#testing-inputs-to-a-program)
  - [Waiting for a website to appear online](#waiting-for-a-website-to-appear-online)
  - [Waiting for a file to be created](#waiting-for-a-file-to-be-created)
  - [Create a backup for all files in a directory](#create-a-backup-for-all-files-in-a-directory)
  - [Keep trying a failing script until it passes, up to 5 times](#keep-trying-a-failing-script-until-it-passes-up-to-5-times)
  - [Comparison with GNU Parallel](#comparison-with-gnu-parallel)
  - [More examples](#more-examples)
- [Contributing](#contributing)
- [License](#license)

<!-- END doctoc generated TOC please keep comment here to allow auto update -->

## Installation

### Linux

`loop` is available on Snapcraft for all distributions as `loop-rs`.

    $ snap install loop-rs --beta

_Issues related to this package are tracked [here](https://github.com/Miserlou/Loop/issues/4)._

There is also an AUR for Arch Linux users, but I don't maintain it, so use it at your own risk:

    $ yaourt -S loop

### OSX

If you're a Homebrew user:

    $ brew tap miserlou/loop https://github.com/Miserlou/Loop.git
    $ brew install loop --HEAD

### Rust Users

    $ cargo install loop-rs

#### Building

    $ cargo build
    ./debug/loop
    $ cargo run 'echo $COUNT'
    1
    2
    [ .. ]

## Usage

With no arguments, `loop` will simply repeatedly execute a command string as fast as it can until `^C` (control + C) is sent.

    $ loop 'echo hello'
    hello
    hello
    hello
    hello
    [ .. ]

You can also use double dashes ( ` -- ` ) to seperate arguments:

    $ loop -- echo hello
    hello
    hello
    hello
    hello
    [ .. ]

### Counters

`loop` places a counter value into the `$COUNT` environment variable.

    $ loop 'echo $COUNT'
    0
    1
    2
    [ .. ]

The amount this counter increments can be changed with `--count-by`:

    $ loop 'echo $COUNT' --count-by 2
    0
    2
    4
    6
    [ .. ]

The counter can be offset with `--offset`:

    $ loop 'echo $COUNT' --count-by 2 --offset 10
    10
    12
    14
    [ .. ]

And iterators can be floats!

    $ loop 'echo $COUNT' --count-by 1.1
    0
    1.1
    2.2
    [ .. ]

There's also an `$ACTUALCOUNT`:

    $ loop 'echo $COUNT $ACTUALCOUNT' --count-by 2
    0 0
    2 1
    4 2
    [ .. ]

You can get a summary of successes and failures (based on exit codes) with `--summary`:

    $ loop 'echo $COUNT' --num 3 --summary
    0
    1
    2
    Total runs:  3
    Successes:   3
    Failures:    0

or

    $ loop 'ls -foobarbatz' --num 3 --summary
    [ .. ]
    Total runs:  3
    Successes:   0
    Failures:    3 (-1, -1, -1)

If you only want the output of the last result, you can use `--only-last`:

    $ loop 'echo $COUNT' --count-by 2 --num 50 --only-last
    100

### Timed Loops

Loops can be set to timers which accept [humanized times](https://github.com/tailhook/humantime) from the microsecond to the year with `--every`:

    $ loop 'date' --every 5s
    Thu May 17 10:51:03 EDT 2018
    Thu May 17 10:51:08 EDT 2018
    Thu May 17 10:51:13 EDT 2018

Looping can be limited to a set duration with `--for-duration`:

    $ loop 'date' --for-duration 8s --every 2s
    Fri May 25 16:46:42 EDT 2018
    Fri May 25 16:46:44 EDT 2018
    Fri May 25 16:46:46 EDT 2018
    Fri May 25 16:46:48 EDT 2018
    $

Or until a certain date/time with `--until-time`:

    $ loop 'date -u' --until-time '2018-05-25 20:50:00' --every 5s
    Fri May 25 20:49:49 UTC 2018
    Fri May 25 20:49:54 UTC 2018
    Fri May 25 20:49:59 UTC 2018
    $

### Until Conditions

`loop` can iterate until output contains a string with `--until-contains`:

    $ loop 'echo $RANDOM' --until-contains "666"
    11235
    35925
    666
    $ 

`loop` can iterate until the output changes with `--until-changes`:

    $ loop --only-last -e 1s 'date +%s' --until-changes
    1548884135
    $

Or until a program succeeds with `--until-success`:

    $ loop 'if (( RANDOM % 2 )); then (echo "TRUE"; true); else (echo "FALSE"; false); fi' --until-success
    FALSE
    FALSE
    TRUE
    $

Or until it fails with `--until-error` (which also accepts an optional error code):

    $ loop 'if (( RANDOM % 2 )); then (echo "TRUE"; true); else (echo "FALSE"; false); fi' --until-error
    TRUE
    TRUE
    FALSE
    $

Or until it matches a regular expression with `--until-match`:

    $ loop 'date' --until-match "(\d{4})"
    Thu May 17 10:51:03 EDT 2018
    $ 

### Iterating Over Lists and Standard Inputs

Loops can iterate over all sorts of lists with `--for`:

    $ loop 'echo $ITEM' --for red,green,blue
    red
    green
    blue
    $ 

And can read from the standard input via pipes:

    $ cat /tmp/my-list-of-files-to-create.txt | loop 'touch $ITEM'
    $ ls
    hello.jpg 
    goodbye.jpg

This can be combined with various flags, such as `--until-changes`:

    $ printf "%s\n" 1 1 3 | loop --until-changes echo '$ITEM'
    1
    1
    3

    $ seq 10 | loop --until-changes echo '$ITEM'
    1
    2


You can also easily pipe lists to `loop`:

    $ ls | loop 'cp $ITEM $ITEM.bak'; ls
    hello.jpg
    hello.jpg.bak

..or via the keyboard with `-i`:

    $ loop 'echo $ITEM | tr a-z A-Z' -i
    hello
    world^D
    HELLO
    WORLD

`--for` can accept all sorts of lists:

    $ loop 'echo $ITEM' --for "`ls`"
    Cargo.lock
    Cargo.toml
    README.md
    src
    target
    $

## Useful Examples

Here are some handy things you can do with `loop`!

### Testing inputs to a program

If you have a lot of files and a program, but don't know which file is the one the program takes, you can loop over them until you find it:

    $ ls | loop './my_program $ITEM' --until-success;

Or, if you have a list of files but need to find the one which causes your program to fail:

    $ ls | loop './my_program $ITEM' --until-fail;

### Waiting for a website to appear online

If you've just kicked off a website deployment pipeline, you might want to run a process when the site starts returning 200 response codes. With `--every` and `--until-contains`, you can do this without flooding the site with requests:

    $ ./deploy.sh; loop 'curl -sw "%{http_code}" http://coolwebsite.biz' --every 5s --until-contains 200; ./announce_to_slack.sh

Or until a host is online:

    $ loop "ping -c 1 mysite.com" --until-success; ./do_next_thing

### Waiting for a file to be created

If you have a long-running process that creates a new file, you might want to kick off another program when that process outputs a new file, like so:

    $ ./create_big_file -o my_big_file.bin; loop 'ls' --until-contains 'my_big_file.bin'; ./upload_big_file my_big_file.bin

### Create a backup for all files in a directory

If you've got a whole list of files that you want to create backup copies of, you can do it like so:

    $ ls
    hello.jpg
    $ ls | loop 'cp $ITEM $ITEM.bak'
    $ ls
    hello.jpg
    hello.jpg.bak

### Keep trying a failing script until it passes, up to 5 times

_This is an [example from StackExchange](https://unix.stackexchange.com/questions/82598/how-do-i-write-a-retry-logic-in-script-to-keep-retrying-to-run-it-upto-5-times/)._

> I want to write logic in shell script which will retry it to run again after 15 sec upto 5 times based on "status code=FAIL" if it fails due to some issue. 

There are so many questions like this on StackExchange, which all end up with long threads of complicated answers. 

With `loop`, it's a simple one liner:

    loop './do_thing.sh' --every 15s --until-success --num 5 

Which will do the thing every 15 seconds until it succeeds, for a maximum of five times.

### Comparison with GNU Parallel

This [thread on Reddit](https://www.reddit.com/r/debian/comments/9ha2dj/ive_written_a_useful_system_utility_how_do_i_get/e6abuht/) with GNU Parallel author Ole Tange has some interesting side-by-side comparisons between `loop` and `parallel`.

### More examples

Got any more useful examples? Send a pull request!

## Contributing

This project is still young, so there is still plenty to be done. Contributions are more than welcome!

Please file tickets for discussion before submitting patches. Pull requests should target `master` and should leave Loop in a "shippable" state if merged.

If you are adding a non-trivial amount of new code, please include a functioning test in your PR. The test suite will be run by [Travis CI](https://travis-ci.org/Miserlou/Zappa) once you open a pull request. Please include the GitHub issue or pull request URL that has discussion related to your changes as a comment in the code ([example](https://github.com/Miserlou/Zappa/blob/fae2925431b820eaedf088a632022e4120a29f89/zappa/zappa.py#L241-L243)). This greatly helps for project maintainability, as it allows us to trace back use cases and explain decision making. Similarly, please make sure that you meet all of the requirements listed in the [pull request template](https://raw.githubusercontent.com/Miserlou/Zappa/master/.github/PULL_REQUEST_TEMPLATE.md).

Please feel free to work on any open ticket, especially any ticket marked with the "help-wanted" label!

## License

(c) Rich Jones, 2018. MIT License.
