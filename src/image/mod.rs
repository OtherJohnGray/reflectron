pub mod debian;

use std::fs;
use std::path::Path;
use std::process::exit;
use std::env;
use crate::base::*;


pub fn check_and_create_image_dir(image_name: &str) -> String {
    let base_path = "/opt/reflectron/images";
    let image_path = format!("{}/{}", base_path, image_name);

    // // Get the binary name used to call the program
    // let binary_name = env::args().next().unwrap_or_else(|| String::from("reflectron"));
    // let binary_name = binary_name.split('/').last().unwrap_or("reflectron");

    // // Check if the directory already exists
    // if Path::new(&image_path).exists() {
    //     log(&format!(
    //         "Directory {} already exists. Refusing to overwrite the existing image directory out of caution.\n\
    //         Use '{} image delete {}' to delete the image if you need to recreate it.",
    //         image_path, binary_name, image_name
    //     ));
    //     exit(1);
    // }

    // // Create the directory
    // match fs::create_dir_all(&image_path) {
    //     Ok(_) => {
    //         log(&format!("Created directory: {}", image_path));
    //         image_path
    //     },
    //     Err(e) => {
    //         halt(&format!("Failed to create directory {}: {}", image_path, e));
    //     }
    // }
    image_path
}

