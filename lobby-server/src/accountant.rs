use gel_derive::Queryable;
use tonic::{Request, Response, Status};
use uuid::Uuid;
use crate::lobby_server::LobbyServer;
use crate::protocol::accountant_server::Accountant;
use crate::protocol::{AccountResponse, DevAccountRequest};

#[derive(Debug, Queryable)]
pub struct Character {
    pub id: Uuid,
    pub name: String,
    pub race: Race,
}

#[tonic::async_trait]
impl Accountant for LobbyServer {
    async fn get_dev_account(
        &self,
        request: Request<DevAccountRequest>
    ) -> Result<Response<AccountResponse>, Status> {
        let req = request.into_inner();
    }
}
