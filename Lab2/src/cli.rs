use crate::error::Error;
use crate::logger::LogSettings;
use crate::{compiler, io};
use clap::Parser;
use log::LevelFilter;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author = "Denys Sherstiuk", version = "0.0.1")]
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

    #[arg(
        short = 'l',
        long,
        default_value_t = LevelFilter::Warn,
        help = "Встановити рівень логування (Error, Warn, Info, Debug, Trace)."
    )]
    pub log_level: LevelFilter,
}

impl Cli {
    pub fn run() -> Result<(), Error> {
        let context = Cli::parse();

        LogSettings::default()
            .with_output_file(context.output_file)
            .with_level(context.log_level)
            .setup()?;

        let code = io::read_code_file(&context.code_file)?;

        compiler::compile(&code, context.pretty);

        Ok(())
    }
}
