use std::fs::File;
use std::io::{BufRead, BufReader, Error, ErrorKind, Result};
use std::ops::Index;

use regex::Regex;
use mac_address::{get_mac_address, MacAddress};
use sysinfo::{System, RefreshKind, MemoryRefreshKind, Disks, Networks, MacAddr};

use crate::config::Config;
use crate::error_handler::{ProgramError, ProgramResult};

pub(crate) struct Stats {
    memory_info: System,
    disks_info: Disks,
    mac_addr: MacAddr,
}

impl Stats {
    pub(crate) fn get_stats(config: &mut Config) -> ProgramResult<Stats> {
        let memory_info = System::new_with_specifics(
            RefreshKind::new().with_memory(
                MemoryRefreshKind::everything()));
        let disks_info = Disks::new_with_refreshed_list();
        let mac_addr = if let Some((name, data))
            = Networks::new_with_refreshed_list().iter().next() {
            data.mac_address()
        } else {
            MacAddr::from("00:00:00:00:00:00")
        };

        ProgramResult::Ok(Stats {
            memory_info,
            disks_info,
            mac_addr
        })
    }

    pub(crate) fn get_and_serialise(&mut self) -> Vec<u8> {
        self.disks_info.refresh();
        let disk_used_megabytes = if let Some(s) = self.disks_info.iter().next() {
            s.total_space() - s.available_space()
        } else { 0 };
        self.memory_info.refresh_memory();
        let memory_used_bytes = self.memory_info.used_memory();
        let memory_used_string = memory_used_bytes.to_string();
        let disk_used_string = disk_used_megabytes.to_string();
        let mut output_string = format!("{memory_used_string} {disk_used_string}");

        let mac_address_string = self.mac_addr.to_string().replace(":", "");
        output_string = format!("{output_string} {mac_address_string}");
        println!("{}", output_string);
        output_string.into_bytes()
    }
}

fn get_memory_used_bytes(config: &Config) -> ProgramResult<u64> {
    ProgramResult::Ok(1)
}

fn get_disk_used_megabytes(config: &Config) -> ProgramResult<u64> {
    let statfs = nix::sys::statvfs::statvfs(config.root_path.as_str())?;
    let blocks_used = (statfs.blocks() - statfs.blocks_free()) as u64;
    // fragment_size is more consistent than block_size, which differ between machines.
    let disk_used = blocks_used * statfs.fragment_size() as u64;
    let disk_used_megabytes = disk_used / 1048576;
    ProgramResult::Ok(disk_used_megabytes)
}

#[cfg(test)]
mod tests {

}