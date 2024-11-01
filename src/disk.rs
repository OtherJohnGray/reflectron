use crate::*;
use std::fmt;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

pub const DISK_INFO: &str = "
        lsblk -b -ndo NAME,SIZE,TYPE,WWN,SERIAL,MODEL,VENDOR | grep -v ^loop;
        echo '';
        for disk in $(lsblk -ndo NAME | grep -v ^loop); do
            echo \"Disk: $disk\";
            udevadm info --query=all --name=/dev/$disk | grep -E 'ID_|by-';
            echo '';
        done
    ";

#[derive(Debug, Serialize, Deserialize)]
pub struct DiskInfo {
    pub name: String,
    pub size: String,
    pub device_type: String,
    pub wwn: Option<String>,
    pub serial: Option<String>,
    pub model: Option<String>,
    pub vendor: Option<String>,
    pub additional_info: HashMap<String, String>,
}

impl fmt::Display for DiskInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Name: {}", self.name)?;
        writeln!(f, "Size: {}", self.size)?;
        writeln!(f, "Device Type: {}", self.device_type)?;
        
        if let Some(wwn) = &self.wwn {
            writeln!(f, "WWN: {}", wwn)?;
        }
        if let Some(serial) = &self.serial {
            writeln!(f, "Serial: {}", serial)?;
        }
        if let Some(model) = &self.model {
            writeln!(f, "Model: {}", model)?;
        }
        if let Some(vendor) = &self.vendor {
            writeln!(f, "Vendor: {}", vendor)?;
        }
        
        if !self.additional_info.is_empty() {
            writeln!(f, "Additional Information:")?;
            let mut keys: Vec<_> = self.additional_info.keys().collect();
            keys.sort(); // Sort keys for consistent output
            for key in keys {
                if let Some(value) = self.additional_info.get(key) {
                    writeln!(f, "  {}: {}", key, value)?;
                }
            }
        }
        Ok(())
    }
}


pub fn parse_output(output: &str) -> Vec<DiskInfo> {
    let mut disks: Vec<DiskInfo> = Vec::new();
    let mut lines = output.lines();
    let mut current_disk_name: Option<String> = None;
    
    while let Some(line) = lines.next() {
        if line.is_empty() {
            continue;
        }
        
        if line.starts_with("Disk: ") {
            current_disk_name = Some(line.trim_start_matches("Disk: ").to_string());
            continue;
        }
        
        // Handle attribute lines (previously starting with S: or E:)
        if line.starts_with("S: ") || line.starts_with("E: ") {
            if let Some(disk_name) = &current_disk_name {
                if let Some(disk) = disks.iter_mut().find(|d| d.name == *disk_name) {
                    let line = line.trim_start_matches("S: ").trim_start_matches("E: ");
                    if let Some((key, value)) = line.split_once('=') {
                        disk.additional_info.insert(
                            key.trim().to_string(),
                            value.trim().to_string()
                        );
                    }
                }
            }
            continue;
        }
        
        // Handle main disk info lines from lsblk
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 3 {
            continue;
        }

        let disk = DiskInfo {
            name: parts[0].to_string(),
            size: parts.get(1)
                .map(|&s| s.to_string())
                .unwrap_or_default(),
            device_type: parts.get(2)
                .map(|&s| s.to_string())
                .unwrap_or_default(),
            wwn: parts.get(3).map(|&s| s.to_string()),
            serial: parts.get(4).map(|&s| s.to_string()),
            model: parts.get(5).map(|&s| s.to_string()),
            vendor: parts.get(6).map(|&s| s.to_string()),
            additional_info: HashMap::new(),
        };
        
        disks.push(disk);
    }

    disks
}


pub fn create_zvols(machine_name: &str, disks: &[DiskInfo]) {
    for disk in disks {
        // Create a standardized disk ID from vendor, model, and serial
        let disk_id = format!("{}-{}-{}",
            disk.vendor.as_deref().unwrap_or("unknown"),
            disk.model.as_deref().unwrap_or("unknown"),
            disk.serial.as_deref().unwrap_or("unknown")
        ).replace(" ", "-")
         .to_lowercase();

        // Construct the full ZVOL path
        let zvol_path = format!("rpool/reflectron/{}/{}", machine_name, disk_id);

        // Create the ZVOL
        perform(
            &format!("Create ZVOL for disk {}", disk_id),
            Some(zfs(&["list", &zvol_path])),
            zfs(&[
                "create",
                "-s",                   // sparse
                "-b", "4K",             // assume 4k blocksize
                "-V", &disk.size,       // size in bytes, assume a multiple of 4k
                &zvol_path
            ]),
            true
        );
    }
}