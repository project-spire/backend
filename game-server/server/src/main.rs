mod character;
mod database;
mod network;
mod player;
mod settings;
mod world;

use actix::prelude::*;
use clap::Parser;
use crate::database::DatabaseContext;
use crate::network::authenticator::Authenticator;
use crate::network::game_listener::GameListener;
use crate::network::gateway::Gateway;
use std::sync::Arc;

#[derive(Parser, Debug)]
struct Options {
    #[arg(long)]
    dry_run: bool,
}

#[actix::main]
async fn main() {
    let options = Options::parse();

    let settings = match settings::Settings::new() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error loading configuration: {}", e);
            return;
        }
    };
    settings::SETTINGS.set(settings).unwrap();

    let net_settings = match settings::NetworkSettings::new() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error loading network configuration: {}", e);
            return;
        }
    };

    let db_ctx = match DatabaseContext::new(&net_settings).await {
        Ok(ctx) => Arc::new(ctx),
        Err(e) => {
            eprintln!("Error creating database context: {}", e);
            return;
        }
    };

    let gateway = Gateway::new(db_ctx.clone()).start();
    let authenticator = Authenticator::new(net_settings.auth_key, gateway).start();
    let _game_listener = GameListener::new(net_settings.game_listen_port, authenticator).start();

    if options.dry_run {
        println!("Dry running done");
        return;
    }

    tokio::signal::ctrl_c().await.unwrap();
}
