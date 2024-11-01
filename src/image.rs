pub mod debian;

use std::fs;
use std::path::Path;
use std::process::exit;
use std::env;
use crate::*;


pub fn check_and_create_image_dir(image_name: &str) -> String {
    let base_path = "/opt/reflectron/images";
    let image_path = format!("{}/{}", base_path, image_name);

    // Get the binary name used to call the program
    let binary_name = env::args().next().unwrap_or_else(|| String::from("reflectron"));
    let binary_name = binary_name.split('/').last().unwrap_or("reflectron");

    // Check if the directory already exists
    if Path::new(&image_path).exists() {
        log!(
            "Directory {} already exists. Refusing to overwrite the existing image directory out of caution.\n\
            Use '{} image delete {}' to delete the image if you need to recreate it.",
            image_path, binary_name, image_name
        );
        exit(1);
    }

    // Create the directory
    match fs::create_dir_all(&image_path) {
        Ok(_) => {
            log!("Created directory: {}", image_path);
            image_path
        },
        Err(e) => {
            halt!("Failed to create directory {}: {}", image_path, e)
        }
    }
}


pub fn image_which(image: &str, program: &str) -> String {
    let paths = vec![
        "/usr/sbin/",
        "/usr/bin/",
        "/sbin/",
        "/bin/",
    ];

    for path in &paths {
        let program_path = format!("{}{}", path, program);
        if Path::new(&format!("{}{}", image, program_path)).exists() {
            return program_path;
        }
    }

    log!("Could not find program {} in system paths {} under {}. Please install it and try again.", program, &paths.join(" "), image);
    exit(1);
}


pub fn copy_config(image_path: &str) {
    let current_dir = env::current_dir().unwrap_or_else(|e| halt!("Could not find current directory: {}", e));
    let dir_path = current_dir.to_str().unwrap_or_else(|| halt!("Could not generate source path for config file copy"));
    let source_path = format!("{}/files/debian12/etc", dir_path);

    let cp_command = pkexec(&[ &which("cp"), "-R", &source_path, &format!("{}/", image_path)]);

    perform(
        "Copying config",
        None,
        cp_command,
        true
    );
}