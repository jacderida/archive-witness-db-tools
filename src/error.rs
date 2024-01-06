use thiserror::Error;

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, Error)]
#[allow(missing_docs)]
pub enum Error {
    #[error("A 404 response was returned for {0}")]
    ArchiveFileNotFoundError(String),
    #[error("Error response when downloading file: {0}")]
    ArchiveDownloadFailed(u16),
    #[error("Could not read field '{0}' as a string")]
    CouldNotReadStringField(String),
    #[error(transparent)]
    CsvError(#[from] csv::Error),
    #[error(transparent)]
    DateParsingError(#[from] chrono::ParseError),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    LavaTorrent(#[from] lava_torrent::LavaTorrentError),
    #[error("Release ID {0} does not have a torrent")]
    NoTorrentForRelease(i32),
    #[error("Cannot parse path segments from torrent URL")]
    PathSegmentsParseError,
    #[error("The top level directory for the release could not be obtained")]
    ReleaseDirectoryNotObtained,
    #[error(transparent)]
    ReqwestError(#[from] reqwest::Error),
    #[error(transparent)]
    SqlxError(#[from] sqlx::Error),
    #[error(transparent)]
    TemplateError(#[from] indicatif::style::TemplateError),
    #[error("Cannot retrieve torrent files")]
    TorrentFilesError,
    #[error(transparent)]
    UrlParseError(#[from] url::ParseError),
    #[error(transparent)]
    Utf8Error(#[from] std::string::FromUtf8Error),
    #[error(transparent)]
    VarError(#[from] std::env::VarError),
}
