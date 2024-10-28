mod base;
mod image;
mod sanoid;

use clap::Parser;
//use crate::base::*;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Parser, Debug)]
enum Command {
    /// Create a new machine
    New {
        /// Name of the machine
        #[arg(short, long)]
        machine_name: String,
        /// IP address
        #[arg(short, long)]
        ip: String,
        /// Password
        #[arg(short, long)]
        password: String,
    },
    /// Create an image for a specific distribution
    Image {
        /// Action to perform on the image
        #[command(subcommand)]
        action: ImageAction,
    },
}

#[derive(Parser, Debug)]
enum ImageAction {
    /// Create a new image
    Create {
        /// Distribution name
        distro: String,
        /// Enable backports (Debian only)
        #[arg(long, default_value_t = false)]
        backports: bool,
    },
}


fn main() {
    let cli = Cli::parse();

    match cli.command {
        Command::New { machine_name, ip, password } => {
            new(&machine_name, &ip, &password);
        }
        Command::Image { action } => {
            match action {
                ImageAction::Create { distro, backports } => {
                    create_image(&distro, backports);
                }
            }
        }
    }
}


fn new(machine_name: &str, ip: &str, password: &str) {
    println!("Creating a new machine...");
    println!("Machine Name: {}", machine_name);
    println!("IP: {}", ip);
    println!("Password: {}", password);
    // Add your new machine creation logic here
}


fn create_image(distro: &str, backports: bool) {
    println!("Creating image for distribution: {}", distro);
    match distro.to_lowercase().as_str() {
        "debian" => {
            if backports {
                println!("Backports enabled");
            }
            crate::image::debian::create(backports)
        }
        _ => println!("Unsupported distribution: {}\nOnly Debian 12 is supported at the current time.", distro),
    }
}