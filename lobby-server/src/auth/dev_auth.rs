use tonic::{Request, Response, Status};
use tracing::{error, warn};
use front_protocol::lobby::{DevTokenRequest, DevTokenResponse};
use crate::config::config;
use crate::db;
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

        let tx = self.db_pool.begin().await?;

        let account_id = match sqlx::query("SELECT id FROM accounts WHERE platform=? and platform_id=?")
            .bind()
            .bind(&request.dev_id)
            .execute(&self.db_pool)
            .await {
            Ok(row) => {
                row.
            }
            Err(sqlx::Error::RowNotFound) => {
                Self::create_dev_account(&tx, &request.dev_id).await?
            }
            Err(e) => return Err(e.into()),
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
    async fn create_dev_account(
        tx: &db::Transaction<'_>,
        dev_id: &str,
    ) -> Result<u64, db::Error> {

    }
}

fn check_dev_mode() -> Result<(), Status> {
    if config().dev_mode {
        return Ok(());
    }

    warn!("Requested dev API while dev mode not enabled!");
    Err(Status::unavailable("Dev API not enabled"))
}
