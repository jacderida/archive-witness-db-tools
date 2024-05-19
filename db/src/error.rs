use thiserror::Error;

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, Error)]
#[allow(missing_docs)]
pub enum Error {
    #[error(transparent)]
    ChronoError(#[from] chrono::ParseError),
    #[error(transparent)]
    CsvError(#[from] csv::Error),
    #[error("Could not obtain 'Asset Name' field")]
    CumulusAssetNameFieldNotFound,
    #[error("Could not convert from duration to the Postgres interval type")]
    DurationToPgIntervalConversionError,
    #[error("Could not obtain metadata from file command: {0}")]
    FileCommandError(String),
    #[error("The completed master-video-record template does not match the expected format")]
    InvalidMasterVideoRecordFormat,
    #[error("The completed video-record template does not match the expected format")]
    InvalidVideoRecordFormat,
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    MagickError(#[from] magick_rust::MagickError),
    #[error("Could not find master video with ID '{0}'")]
    MasterVideoNotFound(u32),
    #[error("A news broadcast cannot have both a network and an affiliate")]
    NewsBroadcastCannotHaveNetworkAndAffiliate,
    #[error("A news broadcast needs either a network or an affiliate")]
    NewsBroadcastDoesNotHaveNetworkOrAffiliate,
    #[error("Could not obtain NIST reference from path")]
    NistRefNotObtained,
    #[error("Could not convert NIST tape from CSV: {0}")]
    NistTapeConversionError(String),
    #[error("Could not convert NIST video from CSV: {0}")]
    NistVideoConversionError(String),
    #[error("Could not obtain path")]
    PathNotObtained,
    #[error("Could not find release with ID '{0}'")]
    ReleaseNotFound(u32),
    #[error(transparent)]
    SqlError(#[from] sqlx::Error),
    #[error(transparent)]
    VarError(#[from] std::env::VarError),
}
