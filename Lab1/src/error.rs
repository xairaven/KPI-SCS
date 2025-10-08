use std::io;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Utility error: {0}")]
    Utility(CliError),
}

#[derive(Debug, Error)]
pub enum CliError {
    #[error("Code file not found. {0}")]
    CodeFileNotFound(io::Error),

    #[error("Failed to read code file. {0}")]
    FailedToReadCodeFile(io::Error),

    #[error("Failed to write into output file. {0}")]
    FailedToWriteIntoOutputFile(io::Error),
}
