use std::fs::OpenOptions;
use chrono::Local;
use std::path::PathBuf;
use std::thread;
use std::time::Duration;
use std::process::{Command, Stdio};
use std::io::{BufReader, BufRead, Write};

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

pub fn log(message: &str) {
    println!("{}", message);
    write_logfile(message);
}

pub fn halt(message: &str) -> ! {
    let error_message = &format!("ERROR: {}", message);
    eprintln!("{}", error_message);
    write_logfile(error_message);
    std::process::exit(1)
}

pub fn perform(description: &str, check: Option<Command>, mut operation: Command, stream_output: bool) {
    if let Some(mut check_cmd) = check {
        match check_cmd.output() {
            Ok(output) => {
                if output.status.success() {
                    log(&format!("{} was already done, skipping.", description));
                    return
                } 
            },
            Err(e) => {
                halt(&format!("Check command '{}' failed: {}", check_cmd.cmdline(), e));
            }
        }
    }

    if stream_output {
        operation.stdout(Stdio::piped());
        operation.stderr(Stdio::piped());

        let mut child = match operation.spawn() {
            Ok(child) => child,
            Err(e) => {
                halt(&format!("Failed to spawn command '{}': {}", operation.cmdline(), e));
            }
        };

        let stdout = child.stdout.take().expect("Failed to capture stdout");
        let stderr = child.stderr.take().expect("Failed to capture stderr");

        let stdout_handle = thread::spawn(move || {
            let reader = BufReader::new(stdout);
            for line in reader.lines() {
                if let Ok(line) = line {
                    println!("{}", line);
                }
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
                    log(&format!("{} succeeded.", description));
                } else {
                    halt(&format!("Command '{}' failed with exit code: {:?}", operation.cmdline(), status.code()));
                }
            },
            Err(e) => {
                halt(&format!("Failed to wait for command '{}': {}", operation.cmdline(), e));
            }
        }
    } else {
        // Original non-streaming behavior
        match operation.output() {
            Ok(output) => {
                if output.status.success() {
                    log(&format!("{} succeeded.", description));
                } else {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    halt(&format!("Command '{}' failed:\nSTDOUT: {}\nSTDERR: {}", operation.cmdline(), stdout, stderr));
                }
            },
            Err(e) => {
                halt(&format!("Command '{}' failed: {}", operation.cmdline(), e));
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
                halt(&format!("Command '{}' failed: {}", command.cmdline(), e));
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
                halt(&format!("Command '{}' failed:\nSTDOUT: {}\nSTDERR: {}", command.cmdline(), stdout, stderr));
            }
        },
        Err(e) => {
            halt(&format!("Command '{}' failed: {}", command.cmdline(), e));
        }
    }
}