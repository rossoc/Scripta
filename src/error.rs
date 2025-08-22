use std::path::{PathBuf, StripPrefixError};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Settings section not found")]
    SettingsNotFoundGeneric,
    #[error("Content section not found")]
    ContentNotFoundGeneric,
    #[error("File not found")]
    FileNotFound(#[from] std::io::Error),
    #[error("Missing layout in one of the notes")]
    MissingLayoutGeneric,
    #[error("Missing layout in note: {0}")]
    MissingLayout(String),
    #[error("Settings section not found in note: {0}")]
    SettingsNotFound(String),
    #[error("Content section not found in note: {0}")]
    ContentNotFound(String),
    #[error("Missing File {0}")]
    MissingFile(PathBuf),
    #[error("parser::compute_out, unreachable error: {0}")]
    SplitPrefixUnreachable(StripPrefixError),
}
