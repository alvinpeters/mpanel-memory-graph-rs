use std::error::Error;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::process::{ExitCode, Termination};



pub(crate) enum ProgramResult<T> {
    Ok(T),
    Err(ProgramError),
}

// Same as above but without data in Ok()
pub(crate) enum ExitResult {
    Ok,
    Err(ProgramError),
}

#[derive(Debug)]
pub(crate) enum ProgramError {
    // Simple error, usually gets matched as unknown error. Please don't use.
    Error,
    StdError,
    IoError(std::io::Error),
    // String parameter for missing values
    MissingValueError(String),
    MinGreaterThanMaxDurationError(u64, u64),
    ArgParseError(String),
    InvalidArgError(Vec<String>),
    MissingInterfaceError,
    NoInterfaceError(String),
}

impl<T> ProgramResult<T> {
    pub(crate) fn unwrap(self) -> T {
        match self {
            ProgramResult::Ok(t) => t,
            ProgramResult::Err(e) => panic!("{e}")
        }
    }
}

impl Termination for ExitResult {
    fn report(self) -> ExitCode {
        match self {
            ExitResult::Ok => ExitCode::SUCCESS,
            ExitResult::Err(e) => e.into(),
        }
    }
}

impl<T> Into<ExitResult> for ProgramResult<T> {
    fn into(self) -> ExitResult {
        match self {
            ProgramResult::Ok(_) => ExitResult::Ok,
            ProgramResult::Err(e) => ExitResult::Err(e),
        }
    }
}

impl Into<ExitCode> for ProgramError {
    fn into(self) -> ExitCode {
        match self {
            ProgramError::Error => ExitCode::FAILURE,
            ProgramError::IoError(_e) => ExitCode::from(1),
            ProgramError::MissingValueError(_) => ExitCode::FAILURE,
            _ => ExitCode::FAILURE
        }
    }
}

impl Display for ProgramError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            ProgramError::IoError(e) => write!(f, "IO error: {e}"),

            _ => write!(f, "Unknown error"),
        }
    }
}

impl Error for ProgramError {}
