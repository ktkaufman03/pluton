use std::num::TryFromIntError;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("IIO error: {0}")]
    IIOError(#[from] industrial_io::Error),

    #[error("Rust formatting error: {0}")]
    FmtError(#[from] std::fmt::Error),

    #[error("unable to parse attribute data")]
    ParsingError,

    #[error("invalid gain control mode: {0}")]
    InvalidGainControlMode(String),

    #[error("invalid sample rate")]
    InvalidSampleRate,

    #[error("unable to find device: {0}")]
    CantFindDevice(&'static str),

    #[error("unable to find channel: {0}")]
    CantFindChannel(&'static str),

    #[error("failed to convert integer attribute to desired format: {0}")]
    FailedDownconvIntAttr(#[from] TryFromIntError),
}
