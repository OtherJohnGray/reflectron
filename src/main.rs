mod base;
mod image;
mod incus;
mod sanoid;

use clap::Parser;
use crate::base::*;
use crate::incus::*;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Parser, Debug)]
enum Command {
    /// Initialize the application
    Init,
    /// Create a new machine
    New {
        /// Name of the machine
        #[arg(short, long)]
        machine_name: String,
        /// Name of the network
        #[arg(short, long)]
        network_name: String,
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
    },
}


fn main() {
    let cli = Cli::parse();

    match cli.command {
        Command::Init => {
            init();
        }
        Command::New { machine_name, network_name, ip, password } => {
            new(&machine_name, &network_name, &ip, &password);
        }
        Command::Image { action } => {
            match action {
                ImageAction::Create { distro } => {
                    create_image(&distro);
                }
            }
        }
    }
}

fn init() {
    log(&format!("Initializing Reflectron bridge and VM",));
    let reflectron_bridge = create_bridge("reflectron-bridge");
    let profile = create_profile("reflectron-vm");
    let reflectron = create_debian_vm("reflectron", &profile);
    let reflectron_nic = attach_bridge(&reflectron_bridge, &reflectron, "00:16:3e:ff:55:01");
    start_vm(&reflectron);
    configure_nic(reflectron_nic, "reflectron-nic", "10.254.0.1");
}

fn new(machine_name: &str, network_name: &str, ip: &str, password: &str) {
    println!("Creating a new machine...");
    println!("Machine Name: {}", machine_name);
    println!("Network Name: {}", network_name);
    println!("IP: {}", ip);
    println!("Password: {}", password);
    // Add your new machine creation logic here
}

fn create_image(distro: &str) {
    println!("Creating image for distribution: {}", distro);
    match distro.to_lowercase().as_str() {
        "debian" => crate::image::debian::create(),
        // Add more distributions here as needed
        _ => println!("Unsupported distribution: {}\nOnly Debian 12 is supported at the current time.", distro),
    }
}