use crate::base::*;
use std::fs::File;
use std::io::Write;
use std::process::Command;
use std::path::Path;


fn incus(args: &[&str]) -> Command {
    let mut cmd = Command::new("incus");
    cmd.args(args);
    cmd
}

#[derive(Debug)]
pub struct Bridge {
    name: String,
    // address: String
}

#[derive(Debug)]
pub struct Instance {
    name: String
}

#[derive(Debug)]
pub struct Nic {
    #[allow(dead_code)]
    mac: String
}

#[derive(Debug)]
pub struct Profile {
    name: String
}

pub fn create_profile(name: &str) -> Profile {
    perform(
        &format!("Create Incus profile {}", name),
        Some(incus(&["profile", "show", name])), 
        incus(&["profile", "create", name])
    );
    let cpu_count_string = get( Command::new("nproc") );
    let cpu_count = cpu_count_string.trim_end().parse::<u64>().unwrap_or_else(|e| halt(&format!("unable to parse output of ncpu of '{}' : {}", cpu_count_string, e)));
    let cpu_limit = (cpu_count + 1 / 2).to_string(); 
    perform(
        &format!("Set memory and CPU limits for Incus profile {}", name),
        None,
        incus(&["profile", "set", name, &format!("limits.cpu={}", cpu_limit), "limits.memory=4GB"])
    );
    Profile {name: name.to_owned()}
}

pub fn create_bridge(name: &str) -> Bridge {

    let file_path = format!("/etc/systemd/network/{}.netdev", name);
    let path = Path::new(&file_path);

    // Create the file, or truncate it if it already exists
    match File::create(path) {
        Ok(mut file) => {
            let content = format!(
                "[NetDev]\n\
                Name={}\n\
                Kind=bridge\n",
                name
            );
            file.write_all(content.as_bytes())
            .unwrap_or_else(|e| halt(&format!("Could not write to file {} : {}", file_path, e)));
            log(&format!("Wrote bridge config file {}", file_path));
        },
        Err(e) => {
            halt(&format!("Could not open file {} : {}", file_path, e));
        }
    }
    let mut restart = Command::new("systemctl");
    restart.args(&["restart", "systemd-networkd"]);
    perform(
        "Restart networking",
        None,
        restart
    );
    Bridge {name: name.to_owned()}
}

pub fn create_debian_vm(name: &str, profile: &Profile) -> Instance {
    perform(
        &format!("Create Debian VM {}", name),
        Some(incus(&["config", "show", name])),
        incus(&["create", "images:debian/12", name, "--vm", "--profile", &profile.name])
    );
    Instance {name: name.to_owned()}
}

pub fn attach_bridge(bridge: &Bridge, vm: &Instance, mac: &str) -> Nic {
    perform(
        &format!("Attach bridge {} to {}", bridge.name, vm.name),
        Some(incus(&["config", "device", "get", &vm.name, &bridge.name, "name"])),
        incus(&["config", "device", "add", &vm.name, &bridge.name, "nic", "nictype=bridged", &format!("parent={}", &bridge.name), &format!("hwaddr={}", &mac)])
    );
    Nic {mac: mac.to_owned()}
}

pub fn start_vm(instance: &Instance){
    perform(
        &format!("Start VM {}", instance.name),
        Some(incus(&["exec", &instance.name, "ls"])),
        incus(&["start", &instance.name]),
    );
    wait(incus(&["exec", &instance.name, "ls"]), 1);
}

#[allow(dead_code)]
pub fn push_file(instance: &Instance, path: &str) {
    let source_path = format!("/opt/builder/files{}", path);
    if !Path::new(&source_path).exists() {
        halt(&format!("Source file '{}' does not exist", source_path));
    }
    perform(
        &format!("Push file {} to {}", path, instance.name),
        Some(incus(&["file", "get", &instance.name, path])),
        incus(&["file", "push", &source_path, &format!("{}:{}", instance.name, path)])
    );
}

pub fn configure_nic(nic: Nic, name: &str, ip: &str) {
    print!("{:?}{:?}{:?}", nic, name, ip);
}