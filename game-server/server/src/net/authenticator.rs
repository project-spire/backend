use actix::{Actor, Context};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    aid: String, // account_id
    cid: String, // character_id
    prv: String, // privilege
}

pub struct Authenticator {}

impl Actor for Authenticator {
    type Context = Context<Self>;
}