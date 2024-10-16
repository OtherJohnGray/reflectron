mod base;
mod debian;
mod incus;
mod sanoid;

mod init;

use clap::Parser;

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
    }
}

fn init() {
    println!("Initializing the application...");
    // Add your initialization logic here
}

fn new(machine_name: &str, network_name: &str, ip: &str, password: &str) {
    println!("Creating a new machine...");
    println!("Machine Name: {}", machine_name);
    println!("Network Name: {}", network_name);
    println!("IP: {}", ip);
    println!("Password: {}", password);
    // Add your new machine creation logic here
}