use crate::cli::Cli;

fn main() {
    let run_result = Cli::run();

    if let Err(e) = run_result {
        eprintln!("{e}");
    }
}

pub mod cli;
pub mod error;
pub mod io;
