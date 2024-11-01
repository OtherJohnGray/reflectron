use ssh2::Session;
use std::io::prelude::*;
use std::net::TcpStream;
use crate::*;


pub fn new(machine_name: &str, ip: &str, password: &str) {
    let disks = disk::parse_output(&get_disk_info(ip, password));
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
        let disk: disk::DiskInfo = bincode::deserialize(&value).unwrap_or_else(|e| halt!("Could not deserialize disk: {}", e));
        println!("{}", disk);
        println!("-------------------");
    }

    // simulate disks as ZVOLs
    disk::create_zvols(machine_name, &disks);
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

