mod calc;
mod character;
mod config;
mod env;
mod handler;
mod net;
mod physics;
mod player;
mod world;

use std::process::exit;

use actix::prelude::*;
use clap::Parser;
use mimalloc::MiMalloc;
use rustls::crypto::aws_lc_rs;
use tracing::{error, info};

use crate::config::{config, Config};
use crate::env::{env, Env};
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
    let options = Options::parse();

    match init().await {
        Ok(_) => {}
        Err(e) => {
            error!("Failed to initialize: {}", e);
            exit(1);
        }
    }

    if options.dry_run {
        info!("Dry running done");
        exit(0);
    }

    start();

    tokio::signal::ctrl_c().await.unwrap();
}

async fn init() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    aws_lc_rs::default_provider()
        .install_default()
        .map_err(|_| std::io::Error::new(
            std::io::ErrorKind::Other, 
            "Failed to install default provider"
        ))?;

    Config::init()?;
    Env::init()?;

    db::init(
        &config().db_user,
        &config().db_password,
        &config().db_host,
        config().db_port,
        &config().db_name
    ).await?;
    data::load_all(&env().data_dir).await?;

    Ok(())
}

fn start() {
    let _ = Authenticator::from_registry();
    let _ = GameListener::from_registry();
    let _ = Gateway::from_registry();

    let default_zone = Zone::new(0).start();
    Gateway::from_registry().do_send(NewZone {
        id: 0,
        zone: default_zone.clone(),
    });
}

