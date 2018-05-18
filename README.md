# loop [![crates.io](https://img.shields.io/crates/v/loop-rs.svg)](https://crates.io/crates/loop-rs)

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

Counting by a value:

    $ loop 'echo $COUNT' --count-by 5
    0
    4
    9
    14
    [ .. ]

Timed loops:

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

## Installation

Not published yet.

### Building

    cargo build
    ./debug/loop

## Advanced Usage

Iterators can be floats!

    $ loop 'echo $COUNT' --count-by 1.1
    0
    1.1
    2.2
    [ .. ]

There's also an `$ACTUALCOUNT`:

Iterators can be floats!

    $ loop 'echo $COUNT $ACTUALCOUNT' --count-by 2
    0 0
    2 1
    4 2
    [ .. ]

