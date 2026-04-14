use thiserror::Error;

#[derive(Error, Debug)]
pub enum Rdf4jError {
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    #[error("Server returned {status}: {body}")]
    ServerError { status: u16, body: String },

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
