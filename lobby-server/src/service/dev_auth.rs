use crate::config::config;
use crate::error::Error;
use diesel::dsl::{exists, select};
use diesel::prelude::*;
use diesel_async::{AsyncConnection, RunQueryDsl};
use jsonwebtoken::EncodingKey;
use protocol::lobby::*;
use protocol::lobby::dev_auth_server::DevAuth;
use std::time::Duration;
use diesel_async::scoped_futures::ScopedFutureExt;
use tonic::{Request, Response, Status};
// use tower::ServiceExt;
use tracing::{error, warn};
use util::id::Id;

pub struct Server {
    pub encoding_key: EncodingKey,
}

impl Server {
    pub fn new() -> Self {
        let encoding_key = EncodingKey::from_secret(&config().token_key);

        Self { encoding_key }
    }
}

#[tonic::async_trait]
impl DevAuth for Server {
    async fn get_dev_account(
        &self,
        request: Request<GetDevAccountRequest>
    ) -> Result<Response<GetDevAccountResponse>, Status> {
        use data::schema::dev_account::dsl::*;

        check_dev_mode()?;
        let request = request.into_inner();

        let mut conn = db::conn().await.map_err(Error::DatabaseConnection)?;

        let dev_account_id: Id = match dev_account
            .select(account_id)
            .filter(id.eq(&request.dev_id))
            .first(&mut conn)
            .await {
            Ok(aid) => aid,
            Err(db::QueryError::NotFound) => {
                create_dev_account(&mut conn, &request.dev_id).await?
            },
            Err(e) => return Err(Error::DatabaseQuery(e).into()),
        };

        let response = GetDevAccountResponse { account_id: dev_account_id };
        Ok(Response::new(response))
    }

    async fn get_dev_token(
        &self,
        request: Request<GetDevTokenRequest>
    ) -> Result<Response<GetDevTokenResponse>, Status> {
        check_dev_mode()?;
        let request = request.into_inner();

        // Check if the requested dev account exists.
        {
            use data::schema::dev_account::dsl::*;

            let mut conn = db::conn().await.map_err(Error::DatabaseConnection)?;

            let dev_account_exists = select(exists(
                dev_account.filter(account_id.eq(request.account_id))
            )).get_result::<bool>(&mut conn)
                .await
                .map_err(Error::DatabaseQuery)?;

            if !dev_account_exists {
                return Err(Status::invalid_argument("No such dev account"));
            }
        }

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

#[derive(Insertable)]
#[diesel(table_name = data::schema::account)]
#[diesel(check_for_backend(diesel::pg::Pg))]
struct NewAccount {
    id: Id,
}

#[derive(Insertable)]
#[diesel(table_name = data::schema::dev_account)]
#[diesel(check_for_backend(diesel::pg::Pg))]
struct NewDevAccount<'a> {
    id: &'a str,
    account_id: Id,
}

async fn create_dev_account(conn: &mut db::Connection, dev_id: &str) -> Result<i64, Error> {
    let new_account_id = util::id::global();

    conn.transaction::<(), Error, _>(|conn| async move {
        use data::schema::account::dsl::*;
        use data::schema::dev_account::dsl::*;

        let new_account = NewAccount {
            id: new_account_id,
        };

        diesel::insert_into(account)
            .values(&new_account)
            .execute(conn)
            .await?;

        let new_dev_account = NewDevAccount {
            id: dev_id,
            account_id: new_account_id,
        };

        diesel::insert_into(dev_account)
            .values(&new_dev_account)
            .execute(conn)
            .await?;

        Ok(())
    }.scope_boxed()).await?;

    Ok(new_account_id)
}

fn check_dev_mode() -> Result<(), Status> {
    if config().dev_mode {
        return Ok(());
    }

    warn!("Requested dev API while dev mode not enabled!");
    Err(Status::unavailable("Dev API not enabled"))
}
