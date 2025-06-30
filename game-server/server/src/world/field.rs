use actix::{Actor, Context};

pub struct Field {}

impl Actor for Field {
    type Context = Context<Self>;
}