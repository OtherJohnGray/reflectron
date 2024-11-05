pub mod disk;
pub mod image;
pub mod machine;
pub mod settings;

use std::fs::OpenOptions;
use std::path::Path;
use chrono::Local;
use std::path::PathBuf;
use std::thread;
use std::time::Duration;
use std::process::{Command, Stdio};
use std::io::{BufReader, BufRead, Write};
use lazy_static::lazy_static;


#[macro_export]
macro_rules! halt {
    ($($arg:tt)*) => {{
        let error_message = &format!("ERROR: {}", format!($($arg)*));
        eprintln!("{}", error_message);
        write_logfile(error_message);
        std::process::exit(1)
    }}
}

#[macro_export]
macro_rules! log {
    ($($arg:tt)*) => {{
        let message = &format!($($arg)*);
        println!("{}", message);
        write_logfile(message);
    }}
}


lazy_static! {
    static ref DATABASE: sled::Db = sled::open("/opt/reflectron/database")
            .unwrap_or_else(|e| halt!("Could not open database: {}", e));
}

pub fn database() -> &'static sled::Db {
    &DATABASE
}


pub fn write_logfile(message: &str) {
    let now = Local::now();
    let timestamp = now.format("%Y-%m-%d %H:%M:%S").to_string();
    let date = now.format("%Y-%m-%d").to_string();
    
    let log_entry = format!("[{}] {}\n", timestamp, message);
    
    let log_dir = PathBuf::from("/var/log/reflectron");
    let log_file = log_dir.join(format!("{}.log", date));
    
    match OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_file)
    {
        Ok(mut file) => {
            match file.write_all(log_entry.as_bytes()) {
                Ok(_) => {}
                Err(e) => {
                    eprintln!("ERROR: could not write to log file: {:?}: {}", log_file, e);
                    std::process::exit(1);
                }
            }
        }
        Err(e) => {
            eprintln!("ERROR: could not open log file: {:?}: {}", log_file, e);
            std::process::exit(1);
        }
    }
}

pub fn open_database() -> sled::Db {
    sled::open("/opt/reflectron/database").unwrap_or_else(|e| halt!("Could not open database: {}", e))
}

pub fn pkexec(args: &[&str]) -> Command {
    let pkexec_path = which("pkexec");
    let mut pkexec = Command::new(pkexec_path);
    pkexec.args(args);
    pkexec
}

pub fn chroot(new_root: &str, args: &[&str]) -> Command {
    let env_path = which("env");
    let chroot_path = which("chroot");
    let mut chroot_args = vec![&env_path, "-i", &chroot_path, new_root];
    chroot_args.extend_from_slice(args);
    pkexec(&chroot_args)
}

pub fn zfs(args: &[&str]) -> Command {
    let zfs_path = which("zfs");
    let mut zfs_args = vec![&zfs_path[..]];
    zfs_args.extend_from_slice(args);
    pkexec(&zfs_args)
}

pub fn success_stauts(mut command: Command) -> bool {
    match command.output() {
        Ok(output) => {
            output.status.success()
        },
        Err(e) => {
            halt!("Error running check command '{}': {}", command.cmdline(), e);
        }
    }
}

pub fn perform(description: &str, check: Option<Command>, mut operation: Command, stream_output: bool) {
    if let Some(check_cmd) = check {
        if success_stauts(check_cmd) {
            log!("{} was already done, skipping.", description);
        }
    }

    if stream_output {
        operation.stdout(Stdio::piped());
        operation.stderr(Stdio::piped());

        let mut child = match operation.spawn() {
            Ok(child) => child,
            Err(e) => {
                halt!("Failed to spawn command '{}': {}", operation.cmdline(), e);
            }
        };

        let stdout = child.stdout.take().expect("Failed to capture stdout");
        let stderr = child.stderr.take().expect("Failed to capture stderr");

        let stdout_handle = thread::spawn(move || {
            let mut reader = BufReader::new(stdout);
            let mut buffer = Vec::new();
            while reader.read_until(b'\n', &mut buffer).unwrap_or(0) > 0 {
                print!("{}", String::from_utf8_lossy(&buffer));
                buffer.clear();
            }
        });

        let stderr_handle = thread::spawn(move || {
            let reader = BufReader::new(stderr);
            for line in reader.lines() {
                if let Ok(line) = line {
                    eprintln!("{}", line);
                }
            }
        });

        // Wait for the command to finish
        match child.wait() {
            Ok(status) => {
                // Wait for output threads to finish
                stdout_handle.join().expect("Stdout thread panicked");
                stderr_handle.join().expect("Stderr thread panicked");

                if status.success() {
                    log!("{} succeeded.", description);
                } else {
                    halt!("Command '{}' failed with exit code: {:?}", operation.cmdline(), status.code());
                }
            },
            Err(e) => {
                halt!("Failed to wait for command '{}': {}", operation.cmdline(), e);
            }
        }
    } else {
        // Original non-streaming behavior
        match operation.output() {
            Ok(output) => {
                if output.status.success() {
                    log!("{} succeeded.", description);
                } else {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    halt!("Command '{}' failed:\nSTDOUT: {}\nSTDERR: {}", operation.cmdline(), stdout, stderr);
                }
            },
            Err(e) => {
                halt!("Command '{}' failed: {}", operation.cmdline(), e);
            }
        }
    }

}


trait CommandExt {
    fn cmdline(&self) -> String;
}

impl CommandExt for Command {
    fn cmdline(&self) -> String {
        format!("{} {}",
            self.get_program().to_str().unwrap_or(""),
            self.get_args().map(|s| s.to_str().unwrap_or("")).collect::<Vec<_>>().join(" ")
        )
    }
}


pub fn wait(mut command: Command, sleep: u64) {
    loop {
        match command.output() {
            Ok(output) => {
                if output.status.success() {
                    return;
                } else {
                    thread::sleep(Duration::from_secs(sleep));
                }
            },
            Err(e) => {
                halt!("Command '{}' failed: {}", command.cmdline(), e);
            }
        }
    }
}


pub fn get(mut command: Command) -> String {
    match command.output() {
        Ok(output) => {
            if output.status.success() {
                String::from_utf8_lossy(&output.stdout).into_owned()
            } else {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);
                halt!("Command '{}' failed:\nSTDOUT: {}\nSTDERR: {}", command.cmdline(), stdout, stderr);
            }
        },
        Err(e) => {
            halt!("Command '{}' failed: {}", command.cmdline(), e);
        }
    }
}

pub fn which(program: &str) -> String {
    let paths = vec![
        "/usr/sbin/",
        "/usr/bin/",
        "/sbin/",
        "/bin/",
    ];

    for path in &paths {
        let program_path = format!("{}{}", path, program);
        if Path::new(&program_path).exists() {
            return program_path;
        }
    }

    log!("Could not find program {} in system paths {}. Please install it and try again.", program, &paths.join(" "));
    std::process::exit(1);
}