// mod character;
// mod core;
mod net;
mod player;
mod config;
// mod server;
mod world;

use actix::prelude::*;
use clap::Parser;
use crate::net::authenticator::Authenticator;
use crate::net::game_listener::GameListener;

#[derive(Parser, Debug)]
struct Options {
    #[arg(long)]
    dry_run: bool,
}

#[actix::main]
async fn main() {
    let options = Options::parse();

    let config = match config::Config::new() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error loading configuration: {}", e);
            return;
        }
    };
    config::CONFIG.set(config).unwrap();

    let network_config = match config::NetworkConfig::new() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error loading network configuration: {}", e);
            return;
        }
    };

    let authenticator = Authenticator::new(network_config.auth_key);
    let authenticator_addr = authenticator.start();

    let game_listener = GameListener::new(6400, authenticator_addr);
    _ = game_listener.start();

    tokio::signal::ctrl_c().await.unwrap();
}
