use diesel_async::pooled_connection::deadpool::PoolError;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Pool error: {0}")]
    Pool(#[from] PoolError),
    
    #[error("Query error: {0}")]
    Query(#[from] diesel::result::Error),
}