use ssh2::Session;
use std::io::prelude::*;
use std::net::TcpStream;
use std::fmt;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use crate::*;

#[derive(Debug, Serialize, Deserialize)]
struct DiskInfo {
    name: String,
    size: String,
    device_type: String,
    wwn: Option<String>,
    serial: Option<String>,
    model: Option<String>,
    vendor: Option<String>,
    additional_info: HashMap<String, String>,
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

pub fn new(machine_name: &str, ip: &str, password: &str) {
    let disks = parse_output(&get_disk_info(ip, password));
    let db = sled::open("disk_info_db").unwrap_or_else(|e| halt!("Could not open database: {}", e));
    for disk in &disks {
        let disk_data = bincode::serialize(&disk).unwrap_or_else(|e| halt!("Could not serialize data: {}", e));
        db.insert(disk.name.as_bytes(), disk_data).unwrap_or_else(|e| halt!("Could not insert data: {}", e));
    }

    // print stored data
    println!("Machine: {}", machine_name);
    println!("-------------------");
    for item in db.iter() {
        let (_, value) = item.unwrap_or_else(|e| halt!("Could not access data: {}", e));
        let disk: DiskInfo = bincode::deserialize(&value).unwrap_or_else(|e| halt!("Could not deserialize disk: {}", e));
        println!("{}", disk);
        println!("-------------------");
    }

    // simulate disks as ZVOLs
    create_zvols(machine_name, &disks);
}


pub fn get_disk_info(ip: &str, password: &str) -> String {
    // SSH connection and command execution
    let tcp = TcpStream::connect(ip).unwrap_or_else(|e| halt!("Could not open TCP connection to remote machine {} : {}", ip, e));
    let mut sess = Session::new().unwrap_or_else(|e| halt!("Could not create SSH session: {}", e));
    sess.set_tcp_stream(tcp);
    sess.handshake().unwrap_or_else(|e| halt!("Could not perform SSH handshake with remote machine at {} : {}", ip, e));

    sess.userauth_password("root", password).unwrap_or_else(|e| halt!("Authentication failed for SSH user root@{} : {}", ip, e));

    println!("Connected to remote server. Getting disk info...");

    let mut channel = sess.channel_session().unwrap_or_else(|e| halt!("Could not open SSH channtel to {} : {}", ip, e));
    channel.exec("
        lsblk -b -ndo NAME,SIZE,TYPE,WWN,SERIAL,MODEL,VENDOR | grep -v ^loop;
        echo '';
        for disk in $(lsblk -ndo NAME | grep -v ^loop); do
            echo \"Disk: $disk\";
            udevadm info --query=all --name=/dev/$disk | grep -E 'ID_|by-';
            echo '';
        done
    ").unwrap_or_else(|e| halt!("SSH command failed: {}", e));

    let mut output = String::new();
    channel.read_to_string(&mut output).unwrap_or_else(|e| halt!("Could not read SSH command output: {}", e));
    output
}


fn parse_output(output: &str) -> Vec<DiskInfo> {
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


fn create_zvols(machine_name: &str, disks: &[DiskInfo]) {
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