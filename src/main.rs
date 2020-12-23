use std::io::{self, BufRead, BufReader};
use std::env;
use std::fs::File;
use std::path::Path;
use std::time::Duration;
use std::thread::sleep;
use std::process;

extern crate getopts;
use getopts::Options;

extern crate chrono;
use chrono::Utc;


const cgroup_base_mount: &str = "/sys/fs/cgroup";

fn read_stat_key(cgname: &Path, key: &str) -> io::Result<u64> {
    let f = File::open(cgname.join("memory.stat"))?;
    let f = BufReader::new(f);
    let mut pref = String::from(key);
    pref.push_str(" ");
    for l in f.lines() {
        let sl = l.unwrap();
        if !sl.starts_with(&pref) {
            continue;
        }
        let just_val = &sl[pref.len()..];
        let nv =
            u64::from_str_radix(just_val, 10).expect(&format!("cannot convert '{}'", just_val));
        return Ok(nv);
    }
    Err(io::Error::new(io::ErrorKind::NotFound, "key not found"))
}

fn format_usage(opts : &Options) -> io::Result<()> {
    print!("{}", opts.usage("Usage: cgstat [-d DURATION]"));
    Ok(())
}

struct CgstatOptions {
    interval : Duration,
    cg_name: String,
}

fn parse_options() -> Result<CgstatOptions, String> {
    let mut opt = Options::new();
    opt.optflag("h", "help", "Show help");
    opt.optopt("d", "duration", "Sample inerval", "DURATION");

    let cmdline_opts : Vec<String> = env::args().skip(1).collect();
    let matches = opt.parse(cmdline_opts)
        .map_err(|err| format!("error parsing arguments: {}", err))?;

    if matches.opt_present("h") {
        format_usage(&opt);
        process::exit(0);
    }

    let mut cgopts = CgstatOptions{
        interval: Duration::new(1, 0),
        cg_name: String::new(),
    };

    if let Some(intv_str) = matches.opt_str("d") {
        match intv_str.parse() {
            Ok(intv) => { cgopts.interval = Duration::from_secs_f32(intv) },
            Err(err) => { return Err(format!("cannot parse interval: {}", err)) }
        };
    }

    if matches.free.len() != 1 {
        return Err(String::from("no cgroup name"));
    }
    cgopts.cg_name = String::from(&matches.free[0]);

    Ok(cgopts)
}

fn main() -> io::Result<()> {
    let opts = match parse_options() {
        Ok(opts) => { opts }
        Err(err) => {
            eprintln!("error: {:?}", err);
            process::exit(1);
        }
    };
    let cgroup_dir = Path::new(cgroup_base_mount).join("memory").join(opts.cg_name);
    loop {
        let rss = read_stat_key(cgroup_dir.as_path(), "rss").expect("failed to read");
        let now = Utc::now();
        println!("{},{}", now.to_rfc3339(), rss);
        sleep(opts.interval);
    }
}
