use std::io::{BufRead, BufReader, Error, ErrorKind, Result};
use std::ops::Index;

use sysinfo::{System, RefreshKind, MemoryRefreshKind, Disks, Networks, MacAddr};

use crate::config::Config;
use crate::error_handler::{ProgramError, ProgramResult};

pub(crate) struct Stats {
    memory_info: System,
    disks_info: Disks,
    mac_addr: MacAddr,
}

impl Stats {
    pub(crate) fn get_stats(config: &Config) -> ProgramResult<Stats> {
        let memory_info = System::new_with_specifics(
            RefreshKind::new().with_memory(
                MemoryRefreshKind::everything()));
        let disks_info = Disks::new_with_refreshed_list();
        let mac_addr = if let Some((name, data))
            = Networks::new_with_refreshed_list().iter().next() {
            data.mac_address()
        } else {
            MacAddr::UNSPECIFIED
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
            // to megabytes
            (s.total_space() - s.available_space()) / 1048576
        } else { 0 };
        self.memory_info.refresh_memory();
        let memory_used_bytes = self.memory_info.used_memory();
        let memory_used_string = memory_used_bytes.to_string();
        let disk_used_string = disk_used_megabytes.to_string();
        let mut output_string = format!("{memory_used_string} {disk_used_string}");

        let mac_address_string = self.mac_addr.to_string().replace(":", "");
        output_string = format!("{output_string} {mac_address_string}");
        output_string.into_bytes()
    }
}

#[cfg(test)]
mod tests {

}