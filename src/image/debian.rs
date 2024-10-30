use std::env;
use std::process::Command;
use crate::*;
use crate::image::*;


pub fn create(backports: bool) {
    let current_dir = env::current_dir().unwrap_or_else(|e| halt(&format!("Could not find current directory: {}", e)));
    let check_dir = current_dir.join("files/debian12/etc/apt");
    if !check_dir.is_dir() {
        halt("reflectron setup needs to be run from the root of the reflectron project git repository.");
    }

    let image_path = check_and_create_image_dir("debian12");

    // Check if debootstrap is installed
    let debootstrap_path = which("debootstrap");

    // Run debootstrap
    perform(
        &format!("Run debootstrap in {}", image_path),
        None,
        pkexec(&[&debootstrap_path, "bookworm", &image_path]),
        true
    );

    // Copy files
    copy_config(&image_path);

    // Prepare chroot
    perform("Mount proc",   None, pkexec(&[&which("mount"), "-t", "proc", "proc",  &format!("{}/proc",    &image_path)]), false);
    perform("Mount sys",    None, pkexec(&[&which("mount"), "-t", "sysfs", "sys",  &format!("{}/sys",     &image_path)]), false);
    perform("Mount dev",    None, pkexec(&[&which("mount"), "-B", "/dev",          &format!("{}/dev",     &image_path)]), false);
    perform("Mount devpts", None, pkexec(&[&which("mount"), "-t", "devpts", "pts", &format!("{}/dev/pts", &image_path)]), false);

    // prepare apt
    perform("Update apt", None, chroot(&image_path, &[&which("apt"), "update"]), true);

    // generate locale
    perform("Install locales .deb package", None, apt_install(&image_path, backports, &["locales"]), true);
    copy_config(&image_path);
    perform(
        "Generate locales",
        None,
        chroot(&image_path, &[&image_which(&image_path, "locale-gen")]),
        true
    );

    // Then set the default locale
    perform(
        "Set default locale",
        None,
        chroot(&image_path, &[&image_which(&image_path, "update-locale"), "LANG=en_US.UTF-8", "LC_ALL=en_US.UTF-8"]),
        true
    );

    // install additional packages
    perform(
        "Install packages",
        None,
        apt_install(
            &image_path,
            backports,
            &[
                "keyboard-configuration",
                "console-setup",
                "linux-headers-amd64",
                "linux-image-amd64",
                "zfs-initramfs",
                "dosfstools",
                ]
        ),
        true
    );

    // enable services
    perform("Enable zfs", None, chroot(&image_path, &[&image_which(&image_path, "systemctl"), "enable", "zfs.target"]), true);
    perform("Enable zfs-import-cache", None, chroot(&image_path, &[&image_which(&image_path, "systemctl"), "enable", "zfs-import-cache"]), true);
    perform("Enable zfs-mount", None, chroot(&image_path, &[&image_which(&image_path, "systemctl"), "enable", "zfs-mount"]), true);
    perform("Enable zfs-import", None, chroot(&image_path, &[&image_which(&image_path, "systemctl"), "enable", "zfs-import.target"]), true);


}


pub fn apt_install(new_root: &str, backports: bool, args: &[&str]) -> Command {
    let env = image_which(new_root, "env");
    let apt_get = image_which(new_root, "apt-get");
    let mut apt_args = if backports {
        vec![&env, "DEBIAN_FRONTEND=noninteractive", &apt_get, "install", "-y", "-t", "bookworm-backports"]
    } else {
        vec![&env, "DEBIAN_FRONTEND=noninteractive", &apt_get, "install", "-y"]
    };
    apt_args.extend_from_slice(args);
    chroot(new_root, &apt_args)
}

