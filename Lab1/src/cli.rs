use crate::error::Error;
use crate::{compiler, io};
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author = "Sherstiuk Denys", version = "0.1.0")]
pub struct Cli {
    #[arg(short = 'c', long, help = "Файл з кодом.")]
    pub code_file: PathBuf,

    #[arg(
        short = 'o',
        long,
        help = "Назва вихідного файлу. Якщо не вказана, результат буде виведений в консоль."
    )]
    pub output_file: Option<PathBuf>,

    #[arg(short = 'p', action, long, help = "Красивий вивід.")]
    pub pretty: bool,
}

impl Cli {
    pub fn run() -> Result<(), Error> {
        let context = Cli::parse();

        let code = io::read_code_file(&context.code_file)?;

        let output = compiler::compile(&code, context.pretty);

        let output_destination = io::define_output_destination(context.output_file);

        io::write_output(&output, output_destination)
    }
}
