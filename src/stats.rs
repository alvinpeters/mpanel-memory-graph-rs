use std::fs::File;
use std::io::{BufRead, BufReader, Error, ErrorKind, Result};
use std::ops::Index;

use regex::Regex;
use mac_address::{get_mac_address, MacAddress};
use crate::config::Config;

pub(crate) struct Stats {
    memory_used_bytes: u64,
    disk_used_megabytes: u64,
    mac_address: Option<MacAddress>
}

impl Stats {
    pub(crate) fn get_stats(config: &Config) -> Result<Stats> {
        Ok(Stats {
            memory_used_bytes: get_memory_used_bytes(config)?,
            disk_used_megabytes: get_disk_used_megabytes(config)?,
            mac_address: match get_mac_address() {
                Ok(m) => m,
                Err(e) => return Err(Error::new(ErrorKind::NotFound, e))
            }
        })
    }

    pub(crate) fn serialise(self) -> Vec<u8> {
        let memory_used_string = self.memory_used_bytes.to_string();
        let disk_used_string = self.disk_used_megabytes.to_string();
        let mut output_string = format!("{memory_used_string} {disk_used_string}");
        if let Some(mac_address) = self.mac_address {
            let mac_address_string = mac_address.to_string().replace(":", "");
            output_string = format!("{output_string} {mac_address_string}");
        }
        println!("{}", output_string);
        output_string.into_bytes()
    }
}

fn get_memory_used_bytes(config: &Config) -> Result<u64> {
    let meminfo = File::open(&config.meminfo_path)?;
    println!("hi");
    let mut reader = BufReader::new(meminfo);
    let regex = match Regex::new(r"^(?P<key>\S*):\s*(?P<value>\d*)\s*kB") {
        Ok(r) => r,
        Err(_) => return Err(Error::new(ErrorKind::Other, "Invalid regex"))
    };
    let mut memory_total: Option<u64> = None;
    let mut memory_free: Option<u64> = None;
    let mut swap_total: Option<u64> = None;
    let mut swap_free: Option<u64> = None;
    let mut buffers: Option<u64> = None;
    let mut cached: Option<u64> = None;
    for mut line_result in reader.lines() {
        let line = line_result?;
        let capture = regex.captures(&line).unwrap();
        let Some(k) = capture.get(1) else { continue };
        match k.as_str() {
            "MemTotal" => memory_total = Some(capture.get(2).unwrap().as_str().parse::<u64>().unwrap()),
            "MemFree" => memory_free = Some(capture.get(2).unwrap().as_str().parse::<u64>().unwrap()),
            "SwapTotal" => swap_total = Some(capture.get(2).unwrap().as_str().parse::<u64>().unwrap()),
            "SwapFree" => swap_free = Some(capture.get(2).unwrap().as_str().parse::<u64>().unwrap()),
            "Buffers" => buffers = Some(capture.get(2).unwrap().as_str().parse::<u64>().unwrap()),
            "Cached" => cached = Some(capture.get(2).unwrap().as_str().parse::<u64>().unwrap()),
            _ => continue
        };
        println!("{}", line);
        if memory_total.is_some() &&
            memory_free.is_some() &&
            swap_total.is_some() &&
            swap_free.is_some() &&
            buffers.is_some() &&
            cached.is_some() {
            break;
        }
    }
    let memory_used = memory_total.unwrap() - memory_free.unwrap();
    let swap_used = swap_total.unwrap() - swap_free.unwrap();
    let memory_used = memory_used - buffers.unwrap() - cached.unwrap() + swap_used;
    println!("{}", memory_used);
    // Convert to bytes
    Ok(memory_used * 1024)
}

fn get_disk_used_megabytes(config: &Config) -> Result<u64> {
    let statfs = nix::sys::statvfs::statvfs(config.root_path.as_str())?;
    let blocks_used = (statfs.blocks() - statfs.blocks_free()) as u64;
    // fragment_size is more consistent than block_size, which differ between machines.
    let disk_used = blocks_used * statfs.fragment_size() as u64;
    let disk_used_megabytes = disk_used / 1048576;
    Ok(disk_used_megabytes)
}

#[cfg(test)]
mod tests {

}