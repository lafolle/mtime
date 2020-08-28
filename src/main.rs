use clap::{App, AppSettings, Arg};
use libc::{getrusage, rusage, RUSAGE_CHILDREN};
use prettytable::format::TableFormat;
use prettytable::{cell, row, table};
use rand::distributions::{Distribution, Uniform};
use std::io::{self, Write};
use std::mem;
use std::process::Command;
use std::thread;
use std::time::{Duration, Instant};

mod stats;
use stats::{max, mean, median, min, std_dev};

struct RunMetrics {
    wall_clock_dur: Duration,
    rusage: RUsage,
}

#[derive(Debug)]
struct RUsage {
    user_tv_usec: i64,
    system_tv_usec: i64,
}

fn get_rusage() -> RUsage {
    unsafe {
        let mut usage: rusage = mem::MaybeUninit::uninit().assume_init();
        let success = getrusage(RUSAGE_CHILDREN, &mut usage);
        assert_eq!(0, success);
        RUsage {
            user_tv_usec: usage.ru_utime.tv_sec * 1e6 as i64 + usage.ru_utime.tv_usec as i64,
            system_tv_usec: usage.ru_stime.tv_sec * 1e6 as i64 + usage.ru_stime.tv_usec as i64,
        }
    }
}

// mean, stddev, min, median, max.
fn stats(usages: Vec<RunMetrics>) {
    let real_usages: Vec<i64> = usages
        .iter()
        .map(|ru| ru.wall_clock_dur.as_micros() as i64)
        .collect();
    let user_usages: Vec<i64> = usages.iter().map(|ru| ru.rusage.user_tv_usec).collect();
    let system_usages: Vec<i64> = usages.iter().map(|ru| ru.rusage.system_tv_usec).collect();

    let mut table = table!(
        ["", "Mean", "Std.Dev.", "Min", "Median", "Max"],
        [
            "real",
            format!("{:.3}", mean(&real_usages) / 1e6),
            format!("{:.3}", median(&real_usages) / 1e6),
            format!("{:.3}", std_dev(&real_usages) / 1e6),
            format!("{:.3}", *min(&real_usages).unwrap() as f64 / 1e6),
            format!("{:.3}", *max(&real_usages).unwrap() as f64 / 1e6)
        ],
        [
            "user",
            format!("{:.3}", mean(&user_usages) / 1e6),
            format!("{:.3}", median(&user_usages) / 1e6),
            format!("{:.3}", std_dev(&user_usages) / 1e6),
            format!("{:.3}", *min(&user_usages).unwrap() as f64 / 1e6),
            format!("{:.3}", *max(&user_usages).unwrap() as f64 / 1e6)
        ],
        [
            "sys",
            format!("{:.3}", mean(&system_usages) / 1e6),
            format!("{:.3}", median(&system_usages) / 1e6),
            format!("{:.3}", std_dev(&system_usages) / 1e6),
            format!("{:.3}", *min(&system_usages).unwrap() as f64 / 1e6),
            format!("{:.3}", *max(&system_usages).unwrap() as f64 / 1e6)
        ]
    );

    let mut format = TableFormat::new();
    format.column_separator('\t');
    table.set_format(format);
    table.printstd();
}

// Simple mode supported.
fn run(cmd: Vec<&str>, num_runs: u32, sleep_dur: u32) {
    let between = Uniform::from(0..sleep_dur + 1);
    let mut rng = rand::thread_rng();

    let command_program = cmd[0];

    let mut command = Command::new(command_program);
    command.args(&cmd[1..]);

    let mut i = 0;
    let mut rusage_per_run: Vec<RunMetrics> = Vec::with_capacity(num_runs as usize);
    let mut last_cum_rusage: RUsage = get_rusage();
    while i < num_runs {
        let start = Instant::now();
        let output = command.output().unwrap();
        io::stdout().write_all(&output.stdout).unwrap();
        io::stderr().write_all(&output.stderr).unwrap();
        let wall_clock_dur = start.elapsed();

        let cum_usage = get_rusage();

        let cmd_user_tv_usec = cum_usage.user_tv_usec - last_cum_rusage.user_tv_usec;
        let cmd_sys_tv_usec = cum_usage.system_tv_usec - last_cum_rusage.system_tv_usec;

        rusage_per_run.push(RunMetrics {
            wall_clock_dur,
            rusage: RUsage {
                user_tv_usec: cmd_user_tv_usec,
                system_tv_usec: cmd_sys_tv_usec,
            },
        });

        last_cum_rusage = RUsage {
            user_tv_usec: cum_usage.user_tv_usec,
            system_tv_usec: cum_usage.system_tv_usec,
        };

        if i != num_runs - 1 && sleep_dur > 0 {
            let rand_secs = between.sample(&mut rng);
            let sleep_dur = Duration::from_secs(rand_secs as u64);
            thread::sleep(sleep_dur);
        }

        i += 1;
    }
    stats(rusage_per_run);
}

struct Config {
    
}

fn main() {
    // Parse CLI args.
    let matches = App::new("mtime")
        .setting(AppSettings::TrailingVarArg)
        .version("0.0")
        .author("lafolle")
        .about("Rust port of multitime")
        .arg(
            Arg::with_name("numruns")
                .short("n")
                .long("numruns")
                .takes_value(true)
                .required(false)
                .help("Number of times the command will run"),
        )
        .arg(
            Arg::with_name("sleep")
                .short("s")
                .long("sleep")
                .takes_value(true)
                .required(false)
                .help("Sleeps randomly (uniform distribution) for [0..sleep] duration between executing commands")
        )
        .arg(
            Arg::with_name("cmd")
                .required(true)
                .multiple(true)
                .help("Command to be executed"),
        )
        .get_matches();

    let num_runs = matches
        .value_of("numruns")
        .unwrap_or("1")
        .parse::<u32>()
        .unwrap();
    let sleep_dur = matches
        .value_of("sleep")
        .unwrap_or("0")
        .parse::<u32>()
        .unwrap();
    let cmd: Vec<&str> = matches.values_of("cmd").unwrap().collect();

    run(cmd, num_runs, sleep_dur);
}
