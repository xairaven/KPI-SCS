use crate::error::{CliError, Error};
use crate::io::OutputDestination::{Console, File};

pub fn read_code_file(path: &std::path::PathBuf) -> Result<String, Error> {
    std::fs::read_to_string(path).map_err(|e| {
        let cli_error = match e.kind() {
            std::io::ErrorKind::NotFound => CliError::CodeFileNotFound(e),
            _ => CliError::FailedToReadCodeFile(e),
        };

        Error::Utility(cli_error)
    })
}

pub enum OutputDestination {
    Console,
    File(std::path::PathBuf),
}

pub fn define_output_destination(
    output_file: Option<std::path::PathBuf>,
) -> OutputDestination {
    match output_file {
        Some(path) => File(path),
        None => Console,
    }
}

pub fn write_output(
    result: &str, output_destination: OutputDestination,
) -> Result<(), Error> {
    match output_destination {
        Console => {
            println!("{result}");
        },
        File(path) => {
            std::fs::write(&path, result).map_err(|e| {
                let cli_error = CliError::FailedToWriteIntoOutputFile(e);
                Error::Utility(cli_error)
            })?;
        },
    }

    Ok(())
}
