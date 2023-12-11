use thiserror::Error;

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, Error)]
#[allow(missing_docs)]
pub enum Error {
    #[error("Could not read field '{0}' as a string")]
    CouldNotReadStringField(String),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Utf8Error(#[from] std::string::FromUtf8Error),
}
