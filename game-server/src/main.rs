mod calc;
mod character;
mod config;
mod db;
mod env;
mod handler;
mod net;
mod player;
mod world;

use std::process::exit;

use actix::prelude::*;
use clap::Parser;
use mimalloc::MiMalloc;
use rustls::crypto::aws_lc_rs;
use tracing::{error, info};

use crate::config::Config;
use crate::env::Env;
use crate::net::authenticator::Authenticator;
use crate::net::game_listener::GameListener;
use crate::net::gateway::{Gateway, NewZone};
use crate::world::zone::Zone;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

#[derive(Parser, Debug)]
struct Options {
    #[arg(long)]
    dry_run: bool,
}

#[actix::main]
async fn main() {
    tracing_subscriber::fmt::init();

    if let Err(e) = aws_lc_rs::default_provider().install_default() {
        error!("Failed to install crypto provider: {:?}", e);
        exit(1);
    }

    let options = Options::parse();
    if let Err(e) = Config::init() {
        error!("Failed to initialize configuration: {e}");
        exit(1);
    }
    if let Err(e) = Env::init() {
        error!("Failed to initialize environment: {e}");
        exit(1);
    }

    let db_pool = match db::connect().await {
        Ok(pool) => pool,
        Err(e) => {
            error!("Failed to connect to database: {}", e);
            exit(1);
        }
    };

    if let Err(e) = data::load_all(&Env::get().data_dir).await {
        error!("Failed to load data: {}", e);
        exit(1);
    }

    let default_zone = Zone::new(0).start();
    let gateway = Gateway::new(db_pool.clone()).start();
    let authenticator = Authenticator::new(gateway.clone()).start();
    let _game_listener = GameListener::new(authenticator).start();

    gateway.do_send(NewZone {
        id: 0,
        zone: default_zone.clone(),
    });

    if options.dry_run {
        info!("Dry running done");
        exit(0);
    }

    tokio::signal::ctrl_c().await.unwrap();
}
