use std::collections::HashMap;
use std::io::BufRead;
use std::ops::Index;

use sysinfo::{System, RefreshKind, MemoryRefreshKind, Disks, Networks, MacAddr};

use crate::config::Config;
use crate::error_handler::{ProgramError, ProgramResult};

pub(crate) struct Stats {
    memory_info: System,
    disks_info: Disks,
    mac_addr: String,
}

impl Stats {
    pub(crate) fn get_stats(config: &Config) -> ProgramResult<Stats> {
        let memory_info = System::new_with_specifics(
            RefreshKind::new().with_memory(
                MemoryRefreshKind::everything()));
        let disks_info = Disks::new_with_refreshed_list();
        let mac_addr_original = if let Some((name, data))
            = Networks::new_with_refreshed_list().iter().next() {
            data.mac_address()
        } else {
            MacAddr::UNSPECIFIED
        };
        let mac_addr = mac_addr_original.to_string().replace(":", "");
        ProgramResult::Ok(Stats {
            memory_info,
            disks_info,
            mac_addr
        })
    }

    pub(crate) fn get_and_serialise(&mut self) -> Vec<u8> {
        // Get used disk space
        self.disks_info.refresh();
        // Got to be a HashMap because of Apple/macOS wanting to be special.
        let mut disk_used_spaces = HashMap::new();
        let disks_info = Disks::new_with_refreshed_list();
        for disk in disks_info.iter() {
            disk_used_spaces.insert(disk.name(), disk.total_space() - disk.available_space());
        }
        // Sum it up.
        let mut used_space_sum = 0;
        for (_, used_space) in disk_used_spaces {
            used_space_sum += used_space;
        }
        // Divide to megabytes.
        let disk_used_megabytes = used_space_sum / 1048576;
        // Get memory
        self.memory_info.refresh_memory();
        let memory_used_bytes = self.memory_info.used_memory();
        // Combine strings.
        let output_string = memory_used_bytes.to_string()
            + " " + disk_used_megabytes.to_string().as_str()
            + " " + self.mac_addr.as_str();
        output_string.into_bytes()
    }
}

#[cfg(test)]
mod tests {

}