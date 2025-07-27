mod character;
mod data {
    pub use game_data::*;
    
    include!(concat!(env!("OUT_DIR"), "/gen/data.rs"));
}
mod db;
mod game_handler;
mod network;
mod player;
mod settings;
mod world;
mod timestamp;

use actix::prelude::*;
use clap::Parser;
use tracing::{info, error};
use tracing_subscriber::fmt;
use crate::db::DbContext;
use crate::network::authenticator::Authenticator;
use crate::network::game_listener::GameListener;
use crate::network::gateway::{Gateway, NewZone};
use crate::world::zone::Zone;

#[derive(Parser, Debug)]
struct Options {
    #[arg(long)]
    dry_run: bool,
}

#[actix::main]
async fn main() {
    fmt::init();

    let options = Options::parse();

    let settings = match settings::Settings::new() {
        Ok(c) => c,
        Err(e) => {
            error!("Error loading configuration: {}", e);
            return;
        }
    };
    settings::SETTINGS.set(settings).unwrap();

    let net_settings = match settings::NetworkSettings::new() {
        Ok(c) => c,
        Err(e) => {
            error!("Error loading network configuration: {}", e);
            return;
        }
    };

    let db_ctx = match DbContext::new(&net_settings).await {
        Ok(ctx) => ctx,
        Err(e) => {
            error!("Error creating database context: {}", e);
            return;
        }
    };

    let default_zone = Zone::new(0).start();
    let gateway = Gateway::new(db_ctx.clone()).start();
    let authenticator = Authenticator::new(net_settings.token_key, gateway.clone()).start();
    let _game_listener = GameListener::new(net_settings.port, authenticator).start();
    
    gateway.do_send(NewZone { id: 0, zone: default_zone.clone() });

    if options.dry_run {
        info!("Dry running done");
        return;
    }

    tokio::signal::ctrl_c().await.unwrap();
}
