mod config;
mod stats;
mod network_interfaces;
mod logs;
mod error_handler;

use config::Config;

use std::net::{ SocketAddr, UdpSocket};
use std::io::Result;
use std::os::unix::fs::MetadataExt;
use std::thread::sleep;
use sysinfo::{MemoryRefreshKind, Networks, RefreshKind};
use crate::config::ConfigBuilder;
use crate::error_handler::{ExitResult, ProgramResult};
use crate::stats::Stats;

fn main() -> ExitResult {
    let config = match ConfigBuilder::new().parse_args().unwrap() {
        Some(c) => c.set_defaults().build().unwrap(),
        None => return ExitResult::Ok
    };
    let mut stats = Stats::get_stats(&config).unwrap();

    loop {
        // Create socket to stats server
        let socket = UdpSocket::bind(
            SocketAddr::from(([0, 0, 0, 0], 0))).unwrap();
        let data = stats.get_and_serialise();
        socket.send_to(&data, config.stats_destination).unwrap();
        sleep(config.calculate_interval());
    }
    ExitResult::Ok
}
