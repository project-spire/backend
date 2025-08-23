mod calc;
mod character;
mod data;
mod db;
mod net;
mod player;
mod protocol;
mod config;
mod world;
mod timestamp;
mod env;

use actix::prelude::*;
use clap::Parser;
use tracing::{info, error};
use crate::db::DbContext;
use crate::env::Env;
use crate::net::authenticator::Authenticator;
use crate::net::game_listener::GameListener;
use crate::net::gateway::{Gateway, NewZone};
use crate::config::Config;
use crate::world::zone::Zone;

#[derive(Parser, Debug)]
struct Options {
    #[arg(long)]
    dry_run: bool,
}

#[actix::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let options = Options::parse();
    if let Err(e) = Config::init() {
        error!("Failed to initialize configuration: {e}");
        return;
    }
    if let Err(e) = Env::init() {
        error!("Failed to initialize environment: {e}");
        return;
    }

    let db_ctx = match DbContext::new().await {
        Ok(ctx) => ctx,
        Err(e) => {
            error!("Error creating database context: {}", e);
            return;
        }
    };

    if let Err(e) = data::load::load_all(&Env::get().data_dir).await {
        error!("Error loading data: {}", e);
        return;
    }

    let default_zone = Zone::new(0).start();
    let gateway = Gateway::new(db_ctx.clone()).start();
    let authenticator = Authenticator::new(gateway.clone()).start();
    let _game_listener = GameListener::new(authenticator).start();
    
    gateway.do_send(NewZone { id: 0, zone: default_zone.clone() });

    if options.dry_run {
        info!("Dry running done");
        return;
    }

    tokio::signal::ctrl_c().await.unwrap();
}
