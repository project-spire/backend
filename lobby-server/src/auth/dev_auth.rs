use gel_derive::Queryable;
use gel_errors::Error;
use gel_tokio::QueryExecutor;
use tonic::{Request, Response, Status};
use tracing::{error, warn};
use uuid::Uuid;
use front_protocol::lobby::{DevTokenRequest, DevTokenResponse};
use crate::config::Config;
use crate::data::character::Race;
use crate::lobby_server::LobbyServer;
use crate::protocol::{dev_auth_server::DevAuth, DevAccountRequest, DevAccountResponse};

// #[derive(Debug, Queryable)]
// pub struct Character {
//     pub id: Uuid,
//     pub name: String,
//     pub race: Race,
// }

#[tonic::async_trait]
impl DevAuth for LobbyServer {
    async fn get_dev_account(
        &self,
        request: Request<DevAccountRequest>
    ) -> Result<Response<DevAccountResponse>, Status> {
        check_dev_mode()?;
        let request = request.into_inner();

        let account_id = match self.db_client.transaction(|mut tx| {
            let dev_id = request.dev_id.clone();

            async move {
                let account_id = tx.query_single::<Uuid, _>(
                    "SELECT DevAccount { id }
                    FILTER .dev_id = <str>$0
                    LIMIT 1;",
                    &(&dev_id,)
                ).await?;

                if let Some(account_id) = account_id {
                    return Ok(account_id);
                }

                let account_id = tx.query_single::<Uuid, _>(
                    "INSERT DevAccount {
                        dev_id := <str>$0
                    };",
                    &(&dev_id,)
                ).await?;

                Ok(account_id.unwrap())
            }
        }).await {
            Ok(id) => id,
            Err(e) => {
                error!("Failed to execute dev account query: {:?}", e);
                return Err(Status::internal("DB error"));
            }
        };

        let response = DevAccountResponse { account_id: Some(account_id.into()) };

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

fn check_dev_mode() -> Result<(), Status> {
    if Config::get().dev_mode {
        return Ok(());
    }

    warn!("Requested dev API while dev mode not enabled!");
    Err(Status::unavailable("Dev API not enabled"))
}
