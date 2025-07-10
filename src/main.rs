mod error;
mod utils;
mod entry;
mod config;
mod process;
use std::sync::{Arc, Mutex};
use clap::{Parser, Subcommand};
use error::RemError;
use home::home_dir;
use std::path::PathBuf;
use config::{generate_config, ConfigManager};
use process::Process;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short = 'c', long = "config", default_value_t = ("~/.config/rem.json").to_string())]
    config: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Start,
    Add {
        name: String,
        interval: String,
        
        #[arg(short = 'm', long = "message", default_value_t = ("take a break!").to_string())]
        message: String,

        #[arg(short = 'u', long = "urgency", default_value_t = 1)]
        urgency: u8,

        #[arg(short = 'i', long = "icon", default_value_t = ("").to_string())]
        icon: String,
    },
    Remove {
        id: u32,
    },
    List {
        #[arg(short = 'v', long = "verbose")]
        verbose: bool,
    }
}

fn expand_path(path_str: &str) -> PathBuf {
    if let Some(stripped) = path_str.strip_prefix("~/") {
        if let Some(home) = home_dir() {
            return home.join(stripped);
        }
    }
    PathBuf::from(path_str)
}

fn main() -> Result<(), RemError> {
    let args = Args::parse();

    let config_path = expand_path(&args.config);
    if let Err(e) = generate_config(&config_path) {
        eprintln!("error while generating config: {}", e);
        std::process::exit(1);
    }

    let mut process = Process::new(ConfigManager::open(config_path.clone())?);

    match &args.command {
        Commands::Start => {
            let proc = Arc::new(Mutex::new(process));
            Process::start(Arc::clone(&proc));
        },
        Commands::Add { name, interval, message, urgency, icon } => {
            let seconds = utils::get_seconds(interval)?;

            process.configman.add_entry(
                name.to_string(),
                seconds,
                message.to_string(),
                *urgency,
                icon.to_string(),
            );
        },
        Commands::Remove { id } => {
            if *id as usize >= process.configman.config.entries.len() {
                eprintln!("index {} not exists", id);
                std::process::exit(1);
            }
            process.configman.remove_entry(*id);
        },
        Commands::List { verbose } => {
            for i in 0..process.configman.config.entries.len() {
                println!("{}. {}", i, process.configman.config.entries[i].name);
                if *verbose {
                    println!(
                        "\tinterval: {}\n\tmessage: {}",
                        process.configman.config.entries[i].interval,
                        process.configman.config.entries[i].message,
                    )
                }
            }
        },
    }
     Ok(())
}
