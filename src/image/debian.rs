use std::path::Path;
use std::process::{Command, exit};
use crate::base::*;
use crate::image::check_and_create_image_dir;

pub fn create() {
    let image_path = check_and_create_image_dir("debian12");

    // Check if debootstrap is installed
    let debootstrap_path = which_debootstrap().unwrap_or_else(|| {
        log("debootstrap is not installed. Please install it and try again.");
        exit(1);
    });

    // Run debootstrap
    let mut debootstrap = Command::new("pkexec");
    debootstrap.args(&[&debootstrap_path, "bookworm", &image_path]);
    perform(
        &format!("Run debootstrap in {}", image_path),
        None,
        debootstrap,
        true
    );

}


fn which_debootstrap() -> Option<String> {
    let paths = vec![
        "/usr/sbin/debootstrap",
        "/sbin/debootstrap",
        "/usr/local/sbin/debootstrap",
    ];

    for path in paths {
        if Path::new(path).exists() {
            return Some(path.to_string());
        }
    }

    // If not found in standard locations, try to find it in PATH
    if let Ok(output) = Command::new("which").arg("debootstrap").output() {
        if output.status.success() {
            return String::from_utf8(output.stdout).ok().map(|s| s.trim().to_string());
        }
    }

    None
}