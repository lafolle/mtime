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

mod config;
mod stats;
use config::Config;
use stats::{stats, ContextStat};

pub struct RunMetrics {
    pub wall_clock_dur: Duration,
    pub rusage: RUsage,
}

pub struct RUsage {
    pub user_tv_usec: i64,
    pub system_tv_usec: i64,
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

fn display(ctx_stats: ContextStat) {
    let mut table = table!(
        ["", "Mean", "Std.Dev.", "Min", "Median", "Max"],
        [
            "real",
            format!("{:.3}", ctx_stats.wall.mean),
            format!("{:.3}", ctx_stats.wall.std_dev),
            format!("{:.3}", ctx_stats.wall.min),
            format!("{:.3}", ctx_stats.wall.median),
            format!("{:.3}", ctx_stats.wall.max)
        ],
        [
            "user",
            format!("{:.3}", ctx_stats.user.mean),
            format!("{:.3}", ctx_stats.user.std_dev),
            format!("{:.3}", ctx_stats.user.min),
            format!("{:.3}", ctx_stats.user.median),
            format!("{:.3}", ctx_stats.user.max)
        ],
        [
            "sys",
            format!("{:.3}", ctx_stats.sys.mean),
            format!("{:.3}", ctx_stats.sys.std_dev),
            format!("{:.3}", ctx_stats.sys.min),
            format!("{:.3}", ctx_stats.sys.median),
            format!("{:.3}", ctx_stats.sys.max)
        ]
    );

    let mut format = TableFormat::new();
    format.column_separator('\t');
    table.set_format(format);
    table.printstd();
}

// Simple mode supported.
fn run(cfg: Config) {

    if cfg.initial_delay != 0 {
        let s = Duration::from_secs(cfg.initial_delay as u64);
        thread::sleep(s);
    }

    let between = Uniform::from(0..cfg.sleep_dur + 1);
    let mut rng = rand::thread_rng();

    let command_program = &cfg.cmd[0];

    let mut command = Command::new(command_program);
    command.args(&cfg.cmd[1..]);

    let mut i = 0;
    let mut rusage_per_run: Vec<RunMetrics> = Vec::with_capacity(cfg.num_runs as usize);
    let mut last_cum_rusage: RUsage = get_rusage();
    while i < cfg.num_runs {
        let start = Instant::now();
        let output = command.output().unwrap();
        if !cfg.quiet_stdout {
            io::stdout().write_all(&output.stdout).unwrap();
        }
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

        if i != cfg.num_runs - 1 && cfg.sleep_dur > 0 {
            let rand_secs = between.sample(&mut rng);
            let sleep_dur = Duration::from_secs(rand_secs as u64);
            thread::sleep(sleep_dur);
        }

        i += 1;
    }
    let ctx_stats = stats(rusage_per_run);
    display(ctx_stats);
}

fn config() -> Config {
    let matches = App::new("mtime")
        .setting(AppSettings::TrailingVarArg)
        .version("0.0")
        .author("lafolle")
        .about("Rust port of mtime")
        .arg(
            Arg::with_name("quiet")
                .short("q")
                .long("quiet")
                .takes_value(false)
                .required(false)
                .help("Do not emit output of cmd to stdout")
        )
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
            Arg::with_name("initial-delay")
                .short("d")
                .long("--initial-delay")
                .takes_value(true)
                .required(false)
                .help("Waits for this many seconds before executing first run of command")
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
    let quiet_stdout = matches.is_present("quiet");
    let initial_delay = matches
        .value_of("initial-delay")
        .unwrap_or("0")
        .parse::<u32>()
        .unwrap();
    let cmd: Vec<String> = matches
        .values_of("cmd")
        .unwrap()
        .collect::<Vec<&str>>()
        .iter()
        .map(|s| s.to_string())
        .collect();

    Config {
        cmd,
        num_runs,
        sleep_dur,
        initial_delay,
        quiet_stdout,
    }
}

fn main() {
    let cfg = config();
    run(cfg);
}
