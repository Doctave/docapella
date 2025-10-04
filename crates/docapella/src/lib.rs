use thiserror::Error;

pub mod commands {
    pub mod build;
    pub mod dev;
    pub mod init;
}

mod builder;
pub mod file_gatherer;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("An IO error occurred: {0}")]
    IoError(#[from] std::io::Error),
    #[error("{0}")]
    General(String),
    #[error("Fatal build error")]
    FatalBuildError(Vec<libdoctave::Error>),
}
