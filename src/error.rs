use thiserror::Error;

#[derive(Error, Debug)]
pub enum UdpStackError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Serialization error: {0}")]
    Serialization(#[from] bincode::Error),
    #[error("Address parse error: {0}")]
    AddrParse(#[from] std::net::AddrParseError),
    #[error("Other: {0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, UdpStackError>;
