use std::fmt::Formatter;
use strum::ParseError;

pub type AppResult<T> = std::result::Result<T, AppError>;


pub enum AppError {
    FileError(String),
    CsvError(String),
    ParseError(String),
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::FileError(err) => { write!(f, "FileError: {}", err) }
            AppError::CsvError(err) => { write!(f, "CsvError: {}", err) }
            AppError::ParseError(err) => { write!(f, "CsvError: {}", err) }
        }
    }
}

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self { AppError::FileError(err.to_string()) }
}

impl From<csv::Error> for AppError {
    fn from(err: csv::Error) -> Self { AppError::CsvError(err.to_string()) }
}

impl From<ParseError> for AppError {
    fn from(err: ParseError) -> Self { AppError::ParseError(err.to_string()) }
}