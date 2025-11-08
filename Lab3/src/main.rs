// Hide console window on Windows in release mode
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use crate::config::Config;
use crate::logs::Logger;

fn main() {
    let config = match Config::from_file() {
        Ok(value) => value,
        Err(err) => {
            eprintln!("Error. {err}");
            std::process::exit(1);
        },
    };

    if let Err(err) = Logger::default()
        .with_file_title(&config.log_file_title)
        .with_format(&config.log_format)
        .with_level(config.log_level)
        .setup()
    {
        eprintln!("Error. {err}");
        std::process::exit(1);
    }

    log::info!("Starting application.");
    log::info!("Config loaded: {config:#?}");
    log::info!("Logger initialized.");
}

pub mod config;
pub mod errors;
pub mod logs;
