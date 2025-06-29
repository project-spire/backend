// mod character;
mod net;
mod player;
mod settings;
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
