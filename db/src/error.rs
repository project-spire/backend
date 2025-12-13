use diesel_async::pooled_connection::deadpool::{BuildError, PoolError};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Build error: {0}")]
    Build(#[from] BuildError),

    #[error("Pool error: {0}")]
    Pool(#[from] PoolError),
    
    #[error("Query error: {0}")]
    Query(#[from] QueryError),
}

pub type QueryError = diesel::result::Error;
