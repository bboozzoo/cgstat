use std::io::{self, BufRead, BufReader};
use std::env;
use std::fs::File;
use std::path::Path;
use std::time::Duration;
use std::thread::sleep;

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

fn usage() -> io::Result<()> {
    Ok(())
}

struct CgstatOptions {
    interval : Duration,
}

fn parse_options() -> io::Result<CgstatOptions> {
    let mut opt = Options::new();
    opt.optflag("h", "help", "Show help");
    opt.optopt("d", "duration", "Sample inerval", "DURATION");

    Ok(CgstatOptions{
        interval: Duration::new(1, 0),
    })
}

fn main() -> io::Result<()> {
    let opts = parse_options().expect("failed to parse options");
    let cg = env::args().skip(1).next().expect("no arg");
    let cgroup_dir = Path::new(cgroup_base_mount).join("memory").join(cg);
    loop {
        let rss = read_stat_key(cgroup_dir.as_path(), "rss").expect("failed to read");
        let now = Utc::now();
        println!("{},{}", now.to_rfc3339(), rss);
        sleep(opts.interval);
    }
    Ok(())
}
