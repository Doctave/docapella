use thiserror::Error;

pub mod commands {
    pub mod build;
    pub mod dev;
    pub mod init;
}

pub mod file_gatherer;
mod builder;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("An IO error occurred: {0}")]
    IoError(#[from] std::io::Error),
    #[error("{0}")]
    General(String),
}
