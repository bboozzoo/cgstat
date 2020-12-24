use std::env;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;
use std::process;
use std::thread::sleep;
use std::time::Duration;

extern crate getopts;
use getopts::Options;

extern crate chrono;
use chrono::Utc;

const CGROUP_BASE_MOUNT: &str = "/sys/fs/cgroup";

fn find_key_val(r: &mut dyn BufRead, key: &str) -> io::Result<Option<u64>> {
    let pref = format!("{} ", key);
    for l in r.lines() {
        let sl = l?;
        if !sl.starts_with(&pref) {
            continue;
        }
        let just_val = &sl[pref.len()..];
        let nv: u64 = just_val.parse().map_err(|err| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("cannot convert '{}': {}", just_val, err),
            )
        })?;
        return Ok(Some(nv));
    }
    Ok(None)
}

fn read_stat_key(cgname: &Path, key: &str) -> io::Result<Option<u64>> {
    let f = File::open(cgname.join("memory.stat"))?;
    let mut f = BufReader::new(f);
    find_key_val(&mut f, key)
}

fn format_usage(opts: &Options) -> String {
    opts.usage("Usage: cgstat [-d DURATION]")
}

/// Represents an error that occurred while parsing options.
#[derive(Debug, PartialEq)]
enum OptionsError {
    /// Usage was requested via an explicit flag. Carries the formatted usage
    /// string.
    Usage(String),
    /// Invaid option occurred.
    Invalid(String),
}

#[derive(Debug, PartialEq)]
struct CgstatOptions {
    /// Sampling interval.
    interval: Duration,
    // Cgroup name.
    cg_name: String,
}

impl Default for CgstatOptions {
    fn default() -> CgstatOptions {
        CgstatOptions {
            interval: Duration::new(1, 0),
            cg_name: String::new(),
        }
    }
}

fn parse_options(cmdline_opts: &Vec<String>) -> Result<CgstatOptions, OptionsError> {
    let mut opt = Options::new();
    opt.optflag("h", "help", "Show help");
    opt.optopt("d", "duration", "Sample inerval (float)", "DURATION");

    let matches = opt
        .parse(cmdline_opts)
        .map_err(|err| OptionsError::Invalid(format!("error parsing arguments: {}", err)))?;

    if matches.opt_present("h") {
        return Err(OptionsError::Usage(format_usage(&opt)));
    }

    let mut cgopts = CgstatOptions::default();

    if let Some(intv_str) = matches.opt_str("d") {
        cgopts.interval = intv_str
            .parse::<f32>()
            .map_err(|err| OptionsError::Invalid(format!("cannot parse interval: {}", err)))
            .map(|v| Duration::from_secs_f32(v))?;
    }

    if matches.free.len() != 1 {
        return Err(OptionsError::Invalid(String::from("no cgroup name")));
    }
    // TODO: remove leading / if present
    let cg_name = if Path::new(&matches.free[0]).is_absolute() {
        &matches.free[0][1..]
    } else {
        &matches.free[0]
    };
    cgopts.cg_name = String::from(cg_name);

    Ok(cgopts)
}

fn main() -> Result<(), String> {
    let opts = match parse_options(&env::args().skip(1).collect()) {
        Ok(opts) => opts,
        Err(optserr) => match optserr {
            OptionsError::Usage(usage) => {
                eprint!("{}", usage);
                process::exit(0);
            }
            OptionsError::Invalid(err) => {
                eprintln!("error: {:?}", err);
                process::exit(1);
            }
        },
    };
    let cgroup_dir = Path::new(CGROUP_BASE_MOUNT)
        .join("memory")
        .join(opts.cg_name);
    loop {
        let rss = match read_stat_key(cgroup_dir.as_path(), "rss") {
            Ok(val) => match val {
                None => return Err("key not found in file".to_string()),
                Some(rss) => rss,
            },
            Err(err) => return Err(err.to_string()),
        };
        let now = Utc::now();
        println!("{},{}", now.to_rfc3339(), rss);
        sleep(opts.interval);
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::io::Cursor;
    use std::string::ToString;

    fn vec_of_strings<T: ToString>(inp: Vec<T>) -> Vec<String> {
        inp.iter().map(|x| x.to_string()).collect()
    }

    #[test]
    fn test_parse_options_usage() {
        let res = parse_options(&vec_of_strings(vec!["-h"]));
        assert!(matches!(res, Err(OptionsError::Usage(_))));
        let help_str = match res {
            Err(oerr) => match oerr {
                OptionsError::Usage(h) => h,
                _ => {
                    panic!("unexpected")
                }
            },
            _ => {
                panic!("unexpected")
            }
        };
        assert!(help_str.len() > 0);
    }

    #[test]
    fn test_parse_options_all() {
        let res = parse_options(&vec_of_strings(vec!["-d", "0.1", "/foo/bar"]));
        assert_eq!(
            Ok(CgstatOptions {
                interval: Duration::from_secs_f32(0.1),
                cg_name: String::from("foo/bar"),
            }),
            res
        );
    }

    #[test]
    fn test_find_key_val_happy_many() {
        let r = "
foo 1234
baz_baz 1111
bar 4443
baz 9999
";
        let res = find_key_val(&mut Cursor::new(r), "baz");
        assert!(res.is_ok());
        assert_eq!(Some(9999 as u64), res.unwrap());
    }

    #[test]
    fn test_find_key_val_happy_simple() {
        let r = "baz 1234";
        let res = find_key_val(&mut Cursor::new(r), "baz");
        assert!(res.is_ok());
        assert_eq!(Some(1234 as u64), res.unwrap());
    }
}
