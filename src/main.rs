mod error;
mod utils;
mod entry;
mod config;
mod process;
use clap::{Parser, Subcommand};
use config::{generate_config, ConfigManager};
use crate::error::RemError;
use home::home_dir;
use std::path::PathBuf;

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
    },
    Remove {
        id: u32,

        #[arg(short = 'k', long = "keep-next-notif")]
        keep_next_notif: bool,
    },
    Toggle {
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

fn add(configman: &mut ConfigManager, name: String, interval: &str, message: String) -> Result<(), RemError> {
    let seconds = utils::get_seconds(interval)?;

    configman.add_entry(name, seconds, message);

    Ok(())
}

fn remove(configman: &mut ConfigManager, id: u32, _keep_next_notif: bool) {
    if id as usize >= configman.config.entries.len() {
        eprintln!("index {} not exists", id);
        return;
    }
    configman.remove_entry(id);
}

fn toggle(configman: &mut ConfigManager, id: u32) {
    if id as usize >= configman.config.entries.len() {
        eprintln!("index {} not exists", id);
        return;
    }
    configman.toggle_entry(id);
}

fn list(configman: &ConfigManager, verbose: bool) {
    for i in 0..configman.config.entries.len() {
        println!("{}. {}", i, configman.config.entries[i].name);
        if verbose {
            println!(
                "\tinterval: {}\n\tmessage: {}",
                configman.config.entries[i].interval,
                configman.config.entries[i].message,
            )
        }
    }
}

fn main() -> Result<(), RemError> {
    let args = Args::parse();

    let config_path = expand_path(&args.config);
    if let Err(e) = generate_config(&config_path) {
        eprintln!("error while generating config: {}", e);
        std::process::exit(1);
    }

    let mut configman = ConfigManager::open(config_path)?;

    match &args.command {
        Commands::Start {} => println!("started!"),
        Commands::Add { name, interval, message } => add(&mut configman, name.to_string(), interval, message.to_string())?,
        Commands::Remove { id, keep_next_notif } => remove(&mut configman, *id, *keep_next_notif),
        Commands::Toggle { id } => toggle(&mut configman, *id),
        Commands::List { verbose } => list(&configman, *verbose),
    }
     Ok(())
}
