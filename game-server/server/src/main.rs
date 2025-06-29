// mod character;
mod db;
mod net;
mod player;
mod settings;
mod world;

use actix::prelude::*;
use clap::Parser;
use crate::net::authenticator::Authenticator;
use crate::net::game_listener::GameListener;
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

    let network_settings = match settings::NetworkSettings::new() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error loading network configuration: {}", e);
            return;
        }
    };

    let db_pool = match db::new_pool(&network_settings).await {
        Ok(p) => Arc::new(p),
        Err(e) => {
            eprintln!("Error creating DB pool: {}", e);
            return;
        }
    };

    let authenticator = Authenticator::new(network_settings.auth_key);
    let authenticator_addr = authenticator.start();

    let game_listener = GameListener::new(network_settings.game_listen_port, authenticator_addr);
    _ = game_listener.start();

    if options.dry_run {
        println!("Dry running done");
        return;
    }

    tokio::signal::ctrl_c().await.unwrap();
}
