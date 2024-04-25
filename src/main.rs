mod config;
mod stats;

use config::Config;

use std::net::{ SocketAddr, UdpSocket};
use std::io::Result;
use std::os::unix::fs::MetadataExt;
use std::thread::sleep;
use crate::stats::Stats;

fn main() -> Result<()> {
    // Does the initial waiting if not started now.
    let config = match Config::parse_args()? {
        Some(c) => c,
        None => return Ok(()) // Help called
    };
    // Create socket to stats server
    let socket = UdpSocket::bind(
        SocketAddr::from(([0, 0, 0, 0], 0)))?;
    loop {
        let stats = Stats::get_stats(&config)?;
        socket.send_to(&stats.serialise(), config.stats_destination)?;
        sleep(config.calculate_interval());
    }
}
