use tracing::error;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Database error: {0}")]
    Database(#[from] db::Error),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Unknown enum value: {0}")]
    UnknownEnumValue(#[from] prost::UnknownEnumValue)
}

impl From<Error> for tonic::Status {
    fn from(value: Error) -> Self {
        match value {
            Error::Database(e) => {
                error!("Database error: {:?}", e);
                tonic::Status::internal("Database error")
            },
            Error::Validation(msg) => tonic::Status::unauthenticated(msg),
            Error::NotFound(msg) => tonic::Status::not_found(msg),
            Error::UnknownEnumValue(_) => tonic::Status::invalid_argument("")
        }
    }
}
