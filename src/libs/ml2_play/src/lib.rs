use std::io;

use tokio::sync::{mpsc, oneshot};

pub mod binary;
pub mod config;
pub mod data;
pub mod manager;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0:?}")]
    BrokenChannel(#[source] anyhow::Error),

    #[error(transparent)]
    Io(#[from] io::Error),

    #[error("{0:?}")]
    Toml(#[source] anyhow::Error),

    #[error(transparent)]
    UnknownError(#[from] anyhow::Error),
}

impl From<oneshot::error::RecvError> for Error {
    fn from(e: oneshot::error::RecvError) -> Self {
        Error::BrokenChannel(e.into())
    }
}

impl<T> From<mpsc::error::SendError<T>> for Error
where
    T: std::fmt::Debug + Send + Sync + 'static,
{
    fn from(e: mpsc::error::SendError<T>) -> Self {
        Error::BrokenChannel(e.into())
    }
}

impl From<toml::ser::Error> for Error {
    fn from(e: toml::ser::Error) -> Self {
        Error::Toml(anyhow::Error::new(e))
    }
}

impl From<toml::de::Error> for Error {
    fn from(e: toml::de::Error) -> Self {
        Error::Toml(anyhow::Error::new(e))
    }
}
