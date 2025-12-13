use tracing::error;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Database connection error: {0}")]
    DatabaseConnection(#[from] db::Error),
    
    #[error("Database query error: {0}")]
    DatabaseQuery(#[from] diesel::result::Error),

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
            Error::DatabaseConnection(e) => {
                error!("Database connection error: {:?}", e);
                tonic::Status::internal("Database connection error")
            },
            Error::DatabaseQuery(e) => {
                error!("Database query error: {:?}", e);
                tonic::Status::internal("Database query error")
            },
            Error::Validation(msg) => tonic::Status::unauthenticated(msg),
            Error::NotFound(msg) => tonic::Status::not_found(msg),
            Error::UnknownEnumValue(_) => tonic::Status::invalid_argument("")
        }
    }
}
