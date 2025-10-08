use crate::error::Error;
use crate::io;
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author = "Alex Kovalov", version = "0.0.1")]
pub struct Cli {
    #[arg(short = 'c', long, help = "Code file.")]
    pub code_file: PathBuf,

    #[arg(
        short = 'o',
        long,
        help = "Output file name. If not provided, output will be printed to console."
    )]
    pub output_file: Option<PathBuf>,
}

impl Cli {
    pub fn run() -> Result<(), Error> {
        let context = Cli::parse();

        let code = io::read_code_file(&context.code_file)?;

        // TODO: Implement code processing here
        let output = code.clone();

        let output_destination = io::define_output_destination(context.output_file);

        io::write_output(&output, output_destination)
    }
}
