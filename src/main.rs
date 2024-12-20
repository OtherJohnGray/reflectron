
use clap::Parser;
use reflectron::*;
use reflectron::settings::*;

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
    /// Set reflecton properties
    Set {
        /// Property to set
        #[command(subcommand)]
        action: SetAction
    },
    /// Get reflectron properties
    Get {
        /// Property to get
        #[command(subcommand)]
        action: GetAction
    },
    /// List reflectron properties
    Settings,
    
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

#[derive(Parser, Debug)]
enum SetAction {
    /// Set the ZPool to use for disk images
    DiskPool {
        /// pool name
        name: String,
    },
}

#[derive(Parser, Debug)]
enum GetAction {
    /// Set the ZPool to use for disk images
    DiskPool,
}


fn main() {
    let cli = Cli::parse();

    match cli.command {
        Command::New { machine_name, ip, password } => {
            machine::new(&machine_name, &ip, &password);
        }
        Command::Image { action } => {
            match action {
                ImageAction::Create { distro, backports } => {
                    create_image(&distro, backports);
                }
            }
        }
        Command::Set { action } => {
            match action {
                SetAction::DiskPool { name } => {
                    settings::set(Key::DiskPool, &name);
                }
            }
        }
        Command::Get { action } => {
            match action {
                GetAction::DiskPool => {
                    println!("{}", settings::get(Key::DiskPool).unwrap_or("Not set".to_owned()));
                }
            }
        }
        Command::Settings => {
            let settings = settings::list();
            if settings.is_empty() {
                println!("No settings found");
            } else {
                println!("Current settings:");
                for (key, value) in settings {
                    println!("  {}: {}", key, value);
                }
            }
        }
    }
}




fn create_image(distro: &str, backports: bool) {
    println!("Creating image for distribution: {}", distro);
    match distro.to_lowercase().as_str() {
        "debian" => {
            if backports {
                println!("Backports enabled");
            }
            image::debian::create(backports)
        }
        _ => println!("Unsupported distribution: {}\nOnly Debian 12 is supported at the current time.", distro),
    }
}