use gel_derive::Queryable;
use tonic::{Request, Response, Status};
use tracing::error;
use uuid::Uuid;
use crate::data::character::Race;
use crate::lobby_server::LobbyServer;
use crate::protocol::{accountant_server::Accountant, AccountResponse, DevAccountRequest};

// #[derive(Debug, Queryable)]
// pub struct Character {
//     pub id: Uuid,
//     pub name: String,
//     pub race: Race,
// }

#[tonic::async_trait]
impl Accountant for LobbyServer {
    async fn get_dev_account(
        &self,
        request: Request<DevAccountRequest>
    ) -> Result<Response<AccountResponse>, Status> {
        let req = request.into_inner();

        let query = "
            SELECT DevAccount {
                id
            }
            FILTER .dev_id = <str>$0
            LIMIT 1;
        ";

        let account_id = match self.db_client.query_required_single::<Uuid, _>(
            query,
            &(req.dev_id,)
        ).await {
            Ok(id) => id,
            Err(e) => {
                error!("Failed to find dev account: {}", e);
                return Err(Status::invalid_argument("Dev account not found"))
            }
        };

        let resp = AccountResponse { id: Some(account_id.into()) };

        Ok(Response::new(resp))
    }
}
