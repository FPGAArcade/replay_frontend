use thiserror::Error;

#[derive(Error, Debug)]
#[allow(dead_code)]
pub enum InternalError {
    #[error("Io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Notify Error")]
    Notify(#[from] notify::Error),
    #[error("Generic error {text})")]
    GenericError { text: String },
    #[error("Png Error")]
    ArenaError(#[from] arena_allocator::ArenaError),
}

#[allow(dead_code)]
pub type InternalResult<T> = std::result::Result<T, InternalError>;
