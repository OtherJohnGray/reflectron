use crate::base::*;
use crate::incus::*;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Read;

#[derive(Debug, Serialize, Deserialize)]
struct Config {
    bridge_name: String,
    machine_name: String,
    mac: String,
    nic_name: String,
    ip: String,
}

fn parse_config(file_path: &str) -> Config {
    let mut file = File::open(file_path).unwrap_or_else(|e| halt(&format!("Could not open config file {} : {}", file_path, e)));
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap_or_else(|e| halt(&format!("Could not read file {} : {}", file_path, e)));
    serde_yaml::from_str(&contents).unwrap_or_else(|e| halt(&format!("Could not parse config file {} : {}", file_path, e)))
}

fn main() {
    let config = parse_config("/opt/reflectron/config.yaml");
    let reflectron_bridge = create_bridge(&config.bridge_name);
    let profile = create_profile("reflectron-vm");
    let reflectron = create_debian_vm(&config.machine_name, &profile);
    let reflectron_nic = attach_bridge(&reflectron_bridge, &reflectron, &config.mac);
    start_vm(&reflectron);
    configure_nic(reflectron_nic, &config.nic_name, &config.ip);

}

