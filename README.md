![Logo, with help from Linn Kristiansen](https://i.imgur.com/TQp8nu3.png)

# loop [![Build Status](https://travis-ci.org/Miserlou/Loop.svg)](https://travis-ci.org/Miserlou/Loop) [![crates.io](https://img.shields.io/crates/v/loop-rs.svg)](https://crates.io/crates/loop-rs)

_"UNIX's missing `loop` command!"_

`loop` lets you write powerful, intuitive looping one-liners in your favorite shell! Finally, loops in Bash that make sense!

<!-- START doctoc generated TOC please keep comment here to allow auto update -->
<!-- DON'T EDIT THIS SECTION, INSTEAD RE-RUN doctoc TO UPDATE -->
- [loop](#loop--)
- [Why?](#why)
- [Installation](#installation)
  - [Linux](#linux)
  - [OSX](#osx)
  - [Rust Users](#rust-users)
    - [Building](#building)
    - [Publishing](#publishing)
- [Usage](#usage)
  - [Counters](#counters)
  - [Timed Loops](#timed-loops)
  - [Until Conditions](#until-conditions)
  - [Iterating Over Lists and Standard Inputs](#iterating-over-lists-and-standard-inputs)
- [License](#license)
<!-- END doctoc generated TOC please keep comment here to allow auto update -->

## Why?

Loops in bash are surprisingly complicated and fickle! I wanted a simple and intuitive way to write controllable loops that run on **timers**, have **custom counters**, loop **until output matches certain conditions**, and iterate over the **standard input**. And so `loop` was born!

## Installation

### Linux

_TODO! [See here](https://github.com/Miserlou/Loop/issues/4)._

### OSX

If you're a Homebrew user:

    $ brew tap miserlou/loop https://github.com/Miserlou/Loop.git
    $ brew install loop --HEAD

### Rust Users

    $ cargo install loop-rs

#### Building

    $ cargo build
    ./debug/loop

#### Publishing

    $ cargo build
    ./debug/loop

## Usage

### Counters

`loop` places the a counter value into the `$COUNT` enviornment variable.

    $ loop 'echo $COUNT'
    0
    1
    2
    [ .. ]

The amount this counter increments can be changed with `--count-by`:

    $ loop 'echo $COUNT' --count-by 2
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

`loop` can interate until output contains a string with `--until-contains`:

    $ loop 'echo $RANDOM' --until-contains "666"
    11235
    35925
    666
    $ 

Or until a program succeeds with `--until-success`:

    $ loop "if (( RANDOM % 2 )); then (echo "TRUE"; true); else (echo "FALSE"; false); fi" --until-success
    FALSE
    FALSE
    TRUE
    $

Or until it fails with `--until-error` (which also accepts an optional error code):

    $ loop "if (( RANDOM % 2 )); then (echo "TRUE"; true); else (echo "FALSE"; false); fi" --until-error
    TRUE
    TRUE
    FALSE
    $

Or until it matches a regular expression with `--until-error`:

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

    $ cat my-list-of-files-to-create.txt | loop 'touch $ITEM'
    $ ls
    hello.jpg 
    goodbye.jpg

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

## License
(c) Rich Jones, 2018. MIT License.
