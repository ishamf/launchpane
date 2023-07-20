use std::{io, time::SystemTimeError, sync::PoisonError};

#[cfg(target_family = "unix")]
use nix::errno::Errno;

use prisma_client_rust::QueryError;
use serde::Serialize;
use tokio::task::JoinError;

#[derive(Debug, Serialize)]
pub enum AppCommandError {
    ClientError(ClientError),
    QueryError(QueryError),
    TauriError(String),
    IoError(String),
    SystemTimeError(String),
    JoinError(String),
    PoisonError(String),

    #[cfg(target_family = "unix")]
    NixError(i32),
}

#[derive(Debug, Serialize)]
pub enum ClientError {
    CommandNotFound,
    InvalidCommandId,
}

impl From<QueryError> for AppCommandError {
    fn from(err: QueryError) -> Self {
        Self::QueryError(err)
    }
}

impl From<tauri::Error> for AppCommandError {
    fn from(err: tauri::Error) -> Self {
        Self::TauriError(err.to_string())
    }
}

impl From<io::Error> for AppCommandError {
    fn from(err: io::Error) -> Self {
        Self::IoError(err.to_string())
    }
}

impl From<SystemTimeError> for AppCommandError {
    fn from(err: SystemTimeError) -> Self {
        Self::SystemTimeError(err.to_string())
    }
}

#[cfg(target_family = "unix")]
impl From<Errno> for AppCommandError {
    fn from(err: Errno) -> Self {
        Self::NixError(err as i32)
    }
}

impl From<JoinError> for AppCommandError {
    fn from(err: JoinError) -> Self {
        Self::JoinError(err.to_string())
    }
}

impl<T> From<PoisonError<T>> for AppCommandError {
    fn from(err: PoisonError<T>) -> Self {
        Self::PoisonError(err.to_string())
    }
}