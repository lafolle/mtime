Rust port of [multitime](https://github.com/ltratt/multitime) (simple mode) with some custom changes.

[Development in progress]

```
mtime 0.0
lafolle
Rust port of mtime

USAGE:
    mtime [FLAGS] [OPTIONS] <cmd>...

FLAGS:
    -h, --help       Prints help information
    -q, --quiet      Do not emit output of cmd to stdout
    -V, --version    Prints version information

OPTIONS:
    -d, --initial-delay <initial-delay>    Waits for this many seconds before executing first run of command
    -n, --numruns <numruns>                Number of times the command will run
    -s, --sleep <sleep>                    Sleeps randomly (uniform distribution) for [0..sleep] duration between
                                           executing commands

ARGS:
    <cmd>...    Command to be executed
```

[![asciicast](https://asciinema.org/a/356348.svg)](https://asciinema.org/a/356348)
