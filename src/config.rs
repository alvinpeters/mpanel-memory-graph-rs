use std::net::SocketAddr;
use std::io::{ErrorKind, Result};
use std::io::Error;


use getopts::Options;
use rand::{Rng, thread_rng};
use std::env;
use std::ops::Not;
use std::str::FromStr;
use std::thread::sleep;
use std::time::Duration;


pub(crate) struct Config {
    pub(crate) stats_destination: SocketAddr,
    pub(crate) min_interval: Duration,
    // No point subtracting the interval every calculation
    pub(crate) max_interval: Duration,
    pub(crate) meminfo_path: String,
    pub(crate) root_path: String,
}

impl Config {
    pub(crate) fn parse_args() -> Result<Option<Config>> {
        let arguments: Vec<String> = env::args().collect();
        let program_name = arguments[0].clone();

        let mut opts = Options::new();
        // Technically required
        opts.optopt("s", "stats",
                    "set statistics server socket", "HOST:PORT");
        opts.optopt("i", "min-interval",
                    "minimum interval between sending statistics, defaults to 5 minutes",
                    "5m");
        opts.optopt("i", "max-interval",
                   "maximum interval between sending statistics, defaults to 9 minutes",
                    "9m");
        opts.optflag("n", "now",
                     "send statistics now, defaults to waiting the interval");
        opts.optopt("m", "meminfo",
                    "path to meminfo, defaults to /proc/meminfo", "PATH");
        opts.optopt("r", "root",
                    "path to root for disk usage check, defaults to /", "PATH");
        opts.optflag("h", "help",
                     "print this help menu");
        let matches = match opts.parse(&arguments[1..]) {
            Ok(m) => m,
            Err(f) => return Err(Error::new(ErrorKind::InvalidInput, f)),
        };
        if matches.opt_present("h") {
            print_help(&program_name, opts);
            return Ok(None)
        }
        let stats_destination = match matches.opt_str("s") {
            Some(s) => match SocketAddr::from_str(&s) {
                Ok(a) => a,
                Err(_) => return Err(
                    Error::new(ErrorKind::InvalidInput, "Invalid destination socket"))
            },
            None => return Err(Error::new(ErrorKind::InvalidInput, "No destination provided"))
        };
        let min_interval = match matches.opt_str("i") {
            Some(s) => match duration_str::parse(s) {
                Ok(d) => d,
                Err(_) => return Err(
                    Error::new(ErrorKind::InvalidInput, "Invalid duration"))
            },
            None => Duration::from_secs(300) // 5 minutes * 60
        };
        let max_interval = match matches.opt_str("i") {
            Some(s) => match duration_str::parse(s) {
                Ok(d) => d,
                Err(_) => return Err(
                    Error::new(ErrorKind::InvalidInput, "Invalid duration"))
            },
            None => Duration::from_secs(560) // 5 minutes * 60
        };
        if min_interval > max_interval {
            return Err(Error::new(ErrorKind::InvalidInput, "Invalid durations"))
        };
        let meminfo_path = matches.opt_str("m")
            .unwrap_or(String::from("/proc/meminfo"));
        let root_path = matches.opt_str("r")
            .unwrap_or(String::from("/"));
        if matches.opt_present("n").not() {
            sleep(min_interval)
        }
        Ok(Some(Config {
            stats_destination,
            min_interval,
            max_interval,
            meminfo_path,
            root_path,
        }))
    }

    pub(crate) fn calculate_interval(&self) -> Duration {
        // No point on calculating the interval if they're the same
        if self.min_interval == self.min_interval {
            return self.max_interval;
        };
        let min_interval_secs = self.min_interval.as_secs();
        let max_interval_secs = self.max_interval.as_secs();
        let random_interval_secs = thread_rng().gen_range(min_interval_secs..max_interval_secs);
        Duration::from_secs(random_interval_secs)
    }
}

fn print_help(program_name: &str, opts: Options) {
    let brief = format!("Usage: {} FILE [options]", program_name);
    print!("{}", opts.usage(&brief));
}
