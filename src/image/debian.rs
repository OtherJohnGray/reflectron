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
    perform(
        &format!("Run debootstrap in {}", image_path),
        None,
        pkexec(&[&debootstrap_path, "bookworm", &image_path]),
        true
    );

    // Prepare chroot
    perform("Mount proc",   None, pkexec(&["mount", "-t", "proc", "proc",  &format!("{}/proc",    &image_path)]), false);
    perform("Mount sys",    None, pkexec(&["mount", "-t", "sysfs", "sys",  &format!("{}/sys",     &image_path)]), false);
    perform("Mount dev",    None, pkexec(&["mount", "-B", "/dev",          &format!("{}/dev",     &image_path)]), false);
    perform("Mount devpts", None, pkexec(&["mount", "-t", "devpts", "pts", &format!("{}/dev/pts", &image_path)]), false);

    // prepare apt
    write_sources_list(&image_path);
    perform("Update apt", None, chroot(&image_path, &["apt", "update"]), true);

    perform(
        "Configure locales", 
        None,
        chroot(&image_path, &["bash", "-c", "echo 'en_US.UTF-8 UTF-8' > /etc/locale.gen"]),
        false
    );

    // Then generate the locales
    perform(
        "Generate locales",
        None,
        chroot(&image_path, &["locale-gen"]),
        false
    );

    // Then set the default locale
    perform(
        "Set default locale",
        None,
        chroot(&image_path, &["update-locale", "LANG=en_US.UTF-8", "LC_ALL=en_US.UTF-8"]),
        false
    );

    // install additional packages
    perform("Install packages", None, apt_install(&image_path, &["locales", "keyboard-configuration", "console-setup"]), true);


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

fn write_sources_list(image_path: &str) {
    let sources_list_content = r#"deb http://deb.debian.org/debian bookworm main contrib
deb-src http://deb.debian.org/debian bookworm main contrib

deb http://deb.debian.org/debian-security bookworm-security main contrib
deb-src http://deb.debian.org/debian-security/ bookworm-security main contrib

deb http://deb.debian.org/debian bookworm-updates main contrib
deb-src http://deb.debian.org/debian bookworm-updates main contrib

deb http://deb.debian.org/debian bookworm-backports main contrib
deb-src http://deb.debian.org/debian bookworm-backports main contrib"#;

    let sources_list_path = format!("{}/etc/apt/sources.list", image_path);
    
    let temp_file = tempfile::Builder::new().prefix("reflectron-").tempfile().unwrap_or_else(|e| halt(&format!("Failed to create temporary file: {}", e)));
    
    std::fs::write(temp_file.path(), sources_list_content)
        .unwrap_or_else(|e| halt(&format!("Failed to write to temporary file: {}", e)));

    let cp_command = pkexec(&[
        "cp",
        temp_file.path().to_str().unwrap(),
        &sources_list_path
    ]);

    perform(
        "Writing sources.list",
        None,
        cp_command,
        false
    );
}

pub fn apt_install(new_root: &str, args: &[&str]) -> Command {
    let mut apt_args = vec!["env", "DEBIAN_FRONTEND=noninteractive", "apt-get", "install", "-y"];
    apt_args.extend_from_slice(args);
    chroot(new_root, &apt_args)
}

