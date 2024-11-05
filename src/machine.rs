use ssh2::Session;
use std::io::prelude::*;
use std::net::TcpStream;
use crate::*;
use crate::settings::Key;
use crate::disk::Disk;
use ron::ser::{to_string_pretty, PrettyConfig};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct Machine {
    pub name: String,
    pub disks: Vec<Disk>,
}

fn machines_db() -> sled::Tree {
    database().open_tree("machines").unwrap_or_else(|e| halt!("Could not open machines database tree: {}", e))
}

pub fn new(machine_name: &str, ip: &str, password: &str) {
    let db = machines_db();

    if db.contains_key(machine_name).unwrap_or_else(|e| halt!("Error checking if machine exists: {}", e)) {
        halt!("Machine {} already exists in the database", machine_name);
    }

    let zpool = settings::get(Key::DiskPool).unwrap_or_else(|| halt!("Reflectron property disk-pool has not been set. Use 'ref set disk-pool <poolname>' to set it, and retry this command."));
    let zvol_path = format!("{}/reflectron/{}", zpool, machine_name);
    if success_stauts(zfs(&["list", &zvol_path])) {
        halt!("ZFS dataset {} already exists - refusing to overwrite it by creating new machine {}", zvol_path, machine_name);
    }

    let disks = disk::parse_output(&get_disk_info(ip, password));
    let machine = Machine {
        name: machine_name.to_string(),
        disks,
    };

    let config = PrettyConfig::new()
        .struct_names(true)
        .compact_arrays(false);
        
    let machine_data = to_string_pretty(&machine, config)
        .unwrap_or_else(|e| halt!("Could not serialize data: {}", e));

    db.insert(
        machine_name.as_bytes(),
        machine_data.as_bytes()
    ).unwrap_or_else(|e| halt!("Could not insert data: {}", e));  
    db.flush().unwrap_or_else(|e| halt!("Error flushing database: {}", e));

    println!("Machine: {}", machine_name);
    println!("-------------------");
    let machine = get_machine(machine_name).unwrap_or_else(|| halt!("No data found for machine {}", machine_name));
    
    for disk in &machine.disks {
        println!("{}", disk);
        println!("-------------------");
    }

    // simulate disks as ZVOLs
    disk::create_zvols(&machine);
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
    channel.exec(crate::disk::DISK_INFO).unwrap_or_else(|e| halt!("SSH command failed: {}", e));

    let mut output = String::new();
    channel.read_to_string(&mut output).unwrap_or_else(|e| halt!("Could not read SSH command output: {}", e));
    output
}

pub fn get_machine(machine_name: &str) -> Option<Machine> {
    let db = machines_db();
    let bytes = db.get(machine_name.as_bytes()).unwrap_or_else(|e| halt!("Could not retreive data for machine {} : {}", machine_name, e))?;
    let string = String::from_utf8_lossy(&bytes);
    ron::from_str(&string).ok()
}