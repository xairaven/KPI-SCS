use std::io;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Тип: I/O. {0}")]
    IO(IOError),
}

#[derive(Debug, Error)]
pub enum IOError {
    #[error("Файл з кодом не був знайдений. {0}")]
    CodeFileNotFound(io::Error),

    #[error("Не вдалося прочитати файл з кодом. {0}")]
    FailedToReadCodeFile(io::Error),

    #[error("Не вдалося записати результат в файл. {0}")]
    FailedToWriteIntoOutputFile(io::Error),
}
