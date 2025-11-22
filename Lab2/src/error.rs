use std::io;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Тип: I/O. {0}")]
    IO(IOError),

    #[error("Тип: Logging. {0}")]
    Log(LogError),
}

#[derive(Debug, Error)]
pub enum IOError {
    #[error("Файл з кодом не був знайдений. {0}")]
    CodeFileNotFound(io::Error),

    #[error("Помилка при читанні файлу з кодом. {0}")]
    FailedToReadCodeFile(io::Error),
}

#[derive(Error, Debug)]
pub enum LogError {
    #[error("Помилка IO. {0}")]
    IO(#[from] io::Error),

    #[error("Помилка ініціалізаціїї логеру. {0}")]
    SetLogger(log::SetLoggerError),
}
