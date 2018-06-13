![Logo, with help from Linn Kristiansen](https://i.imgur.com/TQp8nu3.png)

# loop [![Build Status](https://travis-ci.org/Miserlou/Loop.svg)](https://travis-ci.org/Miserlou/Loop) [![crates.io](https://img.shields.io/crates/v/loop-rs.svg)](https://crates.io/crates/loop-rs)

UNIX's missing `loop` command. 

## Why?

Loops in bash are surprisingly complicated and fickle! I wanted a simple way to write controllable loops. This is also my first Rust project.

Some examples:

    $ loop ls 
    ./hello.txt
    ./hello.txt
    ./hello.txt
    ./hello.txt
    [ .. ]

Timed loops, which accept [humanized times](https://github.com/tailhook/humantime):

    $ loop 'date' --every 5s
    Thu May 17 10:51:03 EDT 2018
    Thu May 17 10:51:08 EDT 2018
    Thu May 17 10:51:13 EDT 2018
    [ .. ]

Limited loops:

    $ loop 'ls' --num 2
    ./hello.txt
    ./hello.txt
    $ 

Looping until conditions are met:

    $ loop 'echo $RANDOM' --until-contains "666"
    11235
    35925
    666
    $ 

Including regular expressions:

    $ loop 'date' --until-match "(\d{4})"
    Thu May 17 10:51:03 EDT 2018
    $ 

Looping for a certain duration:

    $ loop 'date' --for-duration 8s --every 2s
    Fri May 25 16:46:42 EDT 2018
    Fri May 25 16:46:44 EDT 2018
    Fri May 25 16:46:46 EDT 2018
    Fri May 25 16:46:48 EDT 2018
    $

Or until a certain date/time:

    $ loop 'date -u' --until-time '2018-05-25 20:50:00' --every 5s
    Fri May 25 20:49:49 UTC 2018
    Fri May 25 20:49:54 UTC 2018
    Fri May 25 20:49:59 UTC 2018
    $

Looping over a list of items:

    $ loop 'echo $ITEM' --for red,green,blue
    red
    green
    blue
    $ 

Counting by a value:

    $ loop 'echo $COUNT' --count-by 5
    0
    4
    9
    14
    [ .. ]

## Installation

### Linux

_TODO_

### OSX

If you're a Homebrew user:

    $ brew tap miserlou/loop https://github.com/Miserlou/Loop.git
    $ brew install loop --HEAD

### Rust Users

    $ cargo install loop-rs

### Building

    $ cargo build
    ./debug/loop

## Advanced Usage

Iterators can be floats!

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

The counter can be offset:

    $ loop 'echo $COUNT' --count-by 2 --offset 10
    10
    12
    14
    [ .. ]

`--for` can accept all sorts of lists:

    $ loop 'echo $ITEM' --for "`ls`"
    Cargo.lock
    Cargo.toml
    README.md
    src
    target
    $

