use tonic::{Request, Response, Status};
use tracing::{error, warn};
use front_protocol::lobby::{DevTokenRequest, DevTokenResponse};
use crate::config::config;
use crate::error::Error;
use crate::lobby_server::LobbyServer;
use crate::protocol::{dev_auth_server::DevAuth, DevAccountRequest, DevAccountResponse};

#[tonic::async_trait]
impl DevAuth for LobbyServer {
    async fn get_dev_account(
        &self,
        request: Request<DevAccountRequest>
    ) -> Result<Response<DevAccountResponse>, Status> {
        check_dev_mode()?;
        let request = request.into_inner();

        let account_id: Option<i64> = sqlx::query_scalar("SELECT account_id FROM dev_account WHERE id=$1")
            .bind(&request.dev_id)
            .fetch_optional(&self.db_pool)
            .await
            .map_err(Error::Database)?;

        let account_id = match account_id {
            Some(account_id) => account_id,
            None => self.create_dev_account(&request.dev_id).await?,
        };

        let response = DevAccountResponse { account_id };

        Ok(Response::new(response))
    }

    async fn get_token(
        &self,
        request: Request<DevTokenRequest>
    ) -> Result<Response<DevTokenResponse>, Status> {
        check_dev_mode()?;


        Ok(todo!())
    }
}

impl LobbyServer {
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
