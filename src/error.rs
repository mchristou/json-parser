use thiserror::Error;

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Unexcpected characters")]
    UnexcpectedCharacters(),

    #[error("Invalid input: {0}")]
    InvalidInput(String),
}
