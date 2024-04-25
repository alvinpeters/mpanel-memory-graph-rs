use std::net::{AddrParseError, SocketAddr};

use getopts::Options;
use rand::{Rng, thread_rng};
use configparser::ini::Ini;
use std::env;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::string::ToString;
use std::time::Duration;
use sysinfo::{Disks, Networks};
use crate::error_handler::{ProgramError, ProgramResult};

const STATS_DESTINATION_DEFAULT: String = "110.232.115.0:21000".to_string();
const MIN_INTERVAL_DEFAULT: String = "5m".to_string();
const MAX_INTERVAL_DEFAULT: String = "9m".to_string();
const ROOT_PATH_DEFAULT: PathBuf = PathBuf::from("/");

pub(crate) struct ConfigBuilder {
    stats_destination: Option<String>,
    interface_name: Option<String>,
    min_interval: Option<String>,
    max_interval: Option<String>,
    root_path: Option<PathBuf>,
    // Values that will never leave the builder
    config: Option<String>,
}

pub(crate) struct Config {
    pub(crate) stats_destination: SocketAddr,
    pub(crate) interface_name: Option<String>,
    pub(crate) min_interval: u64,
    pub(crate) max_interval: u64,
    pub(crate) root_path: PathBuf,
}

impl ConfigBuilder {
    pub(crate) fn new() -> ConfigBuilder {
        ConfigBuilder {
            stats_destination: None,
            interface_name: None,
            min_interval: None,
            max_interval: None,
            root_path: None,
            config: None,
        }
    }

    /// Returns None if help is invoked.
    pub(crate) fn parse_args(mut self) -> ProgramResult<Option<ConfigBuilder>> {
        let arguments: Vec<String> = env::args().collect();
        let program_name = arguments[0].clone();

        let mut opts = Options::new();
        // Technically required
        opts.optopt("c", "config",
                    "path to config file, the CLI options take precedence",
                    "/path/to/config");
        opts.optopt("s", "stats",
                    "set statistics server socket", "HOST:PORT");
        opts.optopt("e", "network-interface",
                    "set internet network interface to get MAC address from", "eth0");
        opts.optopt("i", "min-interval",
                    "minimum interval between sending statistics, defaults to 5 minutes",
                    "5m");
        opts.optopt("x", "max-interval",
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
            Err(_e) => return ProgramResult::Err(ProgramError::ArgParseError(
                "Couldn't parse arguments".to_string())),
        };
        if matches.opt_present("h") {
            print_help(&program_name, opts);
            return ProgramResult::Ok(None)
        }
        self.config = matches.opt_str("c");
        self.stats_destination = matches.opt_str("s");
        self.root_path = match matches.opt_str("r") {
            Some(s) => Some(PathBuf::from(s)),
            None => None
        };
        self.min_interval = matches.opt_str("i");
        self.max_interval = matches.opt_str("j");
        ProgramResult::Ok(Some(self))
    }

    fn parse_config(mut self) -> ProgramResult<ConfigBuilder> {
        let config_path = if let Some(c) = self.config.as_deref().take() {
            Path::new(c)
        } else {
            return ProgramResult::Ok(self)
        };
        let mut ini = Ini::new();
        let default_section = env!("CARGO_PKG_NAME");
        ini.set_default_section(default_section);
        ini.set_comment_symbols(&['#']);
        let stats_destination_key = "stats-destination";
        let interface_name_key = "interface-name";
        let root_path_key = "root-path";
        let min_interval_key = "min-interval";
        let max_interval_key = "max-interval";
        if config_path.is_file() {
            ini.load(config_path).unwrap(); // TODO: Remove unwrap
            self.stats_destination = ini.get(default_section, stats_destination_key);
            self.interface_name = ini.get(default_section, interface_name_key);
            self.root_path = match ini.get(default_section, root_path_key) {
                Some(s) => Some(PathBuf::from(s)),
                None => None
            };
            self.min_interval = ini.get(default_section, min_interval_key);
            self.max_interval = ini.get(default_section, max_interval_key);
        } else {
            ini.set(default_section, stats_destination_key, self.stats_destination.clone());
            ini.set(default_section, interface_name_key, self.interface_name.clone());
            ini.set(default_section, root_path_key, match self.root_path.clone() {
                Some(p) => match p.into_os_string().into_string() {
                    Ok(s) => Some(s),
                    Err(e) => return ProgramResult::Err(ProgramError::ArgParseError(
                        format!("Unable to parse root path {:?} into string", e)
                    ))
                },
                None => None
            });
            ini.set(default_section, min_interval_key, self.min_interval.clone());
            ini.set(default_section, max_interval_key, self.max_interval.clone());
        }
        ProgramResult::Ok(self)
    }

    pub(crate) fn set_defaults(mut self) -> ConfigBuilder {
        self.stats_destination = self.stats_destination.or(Some(STATS_DESTINATION_DEFAULT));
        self.interface_name = self.interface_name.or_else(|| {
            let networks = Networks::new_with_refreshed_list();
            // Get the first network interface and hope it's the internet interface
            if let Some((interface_name, _)) = networks.iter().next() {
                Some(interface_name.to_owned())
            } else {
                None
            }
        });
        self.root_path = self.root_path.or_else(|| {
            let mut disks = Disks::new_with_refreshed_list();
            // Get the first network interface and hope it's the internet interface
            if let Some(disk) = disks.iter().next() {
                Some(disk.mount_point().to_path_buf())
            } else {
                None
            }
        });
        self.min_interval = self.min_interval.or(Some(MIN_INTERVAL_DEFAULT));
        self.max_interval = self.max_interval.or(Some(MAX_INTERVAL_DEFAULT));
        self
    }

    /// Build the Config struct. Applies default values as well if missing.
    pub(crate) fn build(mut self) -> ProgramResult<Config> {
        let stats_destination = match self.stats_destination {
            Some(s) => match s.parse::<SocketAddr>() {
                Ok(addr) => addr,
                Err(_e) => return ProgramResult::Err(ProgramError::ArgParseError(
                    "stats destination".to_string()
                ))
            },
            None => return ProgramResult::Err(ProgramError::MissingValueError(
                "status destination".to_string()
            ))
        };
        let root_path = self.root_path.unwrap();
        let interface_name = self.interface_name;
        let mut max_interval = None;
        let min_interval = match self.min_interval {
            Some(min_s) => duration_str::parse(min_s).unwrap().as_secs(),
            None => {
                let min = duration_str::parse(MIN_INTERVAL_DEFAULT).unwrap().as_secs(); //TODO: unwrap
                if let Some(max_s) = self.max_interval {
                let max = duration_str::parse(max_s).unwrap().as_secs();
                // Determine minimum interval
                max_interval = Some(max.clone());
                if max >= min {
                    min
                } else {
                    // Set to same as current max
                    max
                }
            } else {
                min
            }}
        };
        let max_interval = max_interval.unwrap_or_else(|| {
            let max = duration_str::parse(MAX_INTERVAL_DEFAULT).unwrap().as_secs(); //TODO: unwrap
            if min_interval <= max {
                max
            } else {
                min_interval.clone()
            }
        });
        // Last check to see if the durations are valid
        if ! min_interval <= max_interval {
            return ProgramResult::Err(
                ProgramError::MinGreaterThanMaxDurationError(min_interval, max_interval));
        }
        ProgramResult::Ok(Config {
            stats_destination,
            interface_name,
            min_interval,
            max_interval,
            root_path,
        })
    }
}

impl Config {
    pub(crate) fn calculate_interval(&self) -> Duration {
        // No point on calculating the interval if they're the same
        if self.min_interval == self.min_interval {
            return Duration::from_secs(self.max_interval);
        };
        let random_interval_secs = thread_rng().gen_range(
            self.min_interval..=self.max_interval);
        Duration::from_secs(random_interval_secs)
    }
}

fn print_help(program_name: &str, opts: Options) {
    let brief = format!("Usage: {} FILE [options]", program_name);
    print!("{}", opts.usage(&brief));
}
