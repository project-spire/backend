use std::time::Duration;

use tonic::{Request, Response, Status};
use tracing::{error, warn};

use crate::config::config;
use crate::error::Error;
use protocol::lobby::*;
use protocol::lobby::dev_auth_server::DevAuth;

#[tonic::async_trait]
impl DevAuth for Context {
    async fn get_dev_account(
        &self,
        request: Request<GetDevAccountRequest>
    ) -> Result<Response<GetDevAccountResponse>, Status> {
        check_dev_mode()?;
        let request = request.into_inner();

        let account_id: Option<i64> = sqlx::query_scalar!(
            "select account_id from dev_account where id=$1",
            &request.dev_id,
        )
        .fetch_optional(&self.db_pool)
        .await
        .map_err(Error::Database)?;

        let account_id = match account_id {
            Some(account_id) => account_id,
            None => self.create_dev_account(&request.dev_id).await?,
        };

        let response = GetDevAccountResponse { account_id };
        Ok(Response::new(response))
    }

    async fn get_dev_token(
        &self,
        request: Request<GetDevTokenRequest>
    ) -> Result<Response<GetDevTokenResponse>, Status> {
        check_dev_mode()?;
        let request = request.into_inner();

        let _exists: Option<bool> = sqlx::query_scalar!(
            "select true from dev_account where account_id = $1 limit 1",
            &request.account_id,
        )
        .fetch_one(&self.db_pool)
        .await
        .map_err(Error::Database)?;

        let expiration = Duration::from_secs(config().token_expiration_seconds);
        let token = match util::token::generate(request.account_id, &self.encoding_key, expiration) {
            Ok(token) => token,
            Err(e) => {
                error!("Failed to generate token: {}", e);
                return Err(Status::unauthenticated("Token error"));
            }
        };
        
        let response = GetDevTokenResponse { token };
        Ok(Response::new(response))
    }
}

impl Context {
    async fn create_dev_account(&self, dev_id: &str) -> Result<i64, Error> {
        let mut tx = self.db_pool.begin().await?;

        let account_id = util::id::generate();

        sqlx::query!(
            "insert into account (id) values ($1)",
            &account_id,
        )
        .execute(&mut *tx)
        .await?;

        sqlx::query!(
            "insert into dev_account (id, account_id) values ($1, $2)",
            &dev_id,
            &account_id,
        )
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok(account_id)
    }
}

fn check_dev_mode() -> Result<(), Status> {
    if config().dev_mode {
        return Ok(());
    }

    warn!("Requested dev API while dev mode not enabled!");
    Err(Status::unavailable("Dev API not enabled"))
}
