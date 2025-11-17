use crate::cli::Cli;
use colored::Colorize;

fn main() {
    let result = Cli::run();

    result.unwrap_or_else(|err| {
        eprintln!("{}. {err}", "Помилка".red().bold());

        std::process::exit(1);
    });
}

pub mod cli;
pub mod compiler;
pub mod error;
pub mod io;
pub mod utils;
