mod calc;
mod character;
mod config;
mod handler;
mod net;
mod physics;
mod player;
mod task;
mod world;

use crate::net::authenticator::Authenticator;
use crate::net::game_listener::GameListener;
use crate::net::gateway::{Gateway, NewZone};
use crate::net::zone::Zone;
use actix::prelude::*;
use clap::Parser;
use mimalloc::MiMalloc;
use rustls::crypto::aws_lc_rs;
use std::path::PathBuf;
use std::process::exit;
use tracing::{error, info};

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

/// A game server
#[derive(clap::Parser, Debug)]
struct Args {
    /// Initialize only and exit
    #[arg(long, default_value_t = false)]
    dry_run: bool,

    /// Use local environment file
    #[arg(long)]
    local_env: Option<PathBuf>,
}

#[actix::main]
async fn main() {
    if let Ok(path) = std::env::current_dir() {
        println!("Running on \"{}\"", path.display());
    }

    let args = Args::parse();
    
    if let Err(e) = init(&args).await {
        error!("Failed to initialize: {}", e);
        exit(1);
    }

    if args.dry_run {
        info!("Dry running done");
        exit(0);
    }

    run();

    tokio::signal::ctrl_c().await.unwrap();
}

async fn init(args: &Args) -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    aws_lc_rs::default_provider()
        .install_default()
        .map_err(|_| {
            std::io::Error::new(
                std::io::ErrorKind::Other,
                "Failed to install default provider",
            )
        })?;

    config::init(&args.local_env)?;
    util::id::init(config!(net).node_id);
    db::init(
        &config!(db).user,
        &config!(db).password,
        &config!(db).host,
        config!(db).port,
        &config!(db).name,
    ).await?;

    data::init(&config!(app).data.dir).await?;

    Ok(())
}

fn run() {
    _ = Authenticator::from_registry();
    _ = GameListener::from_registry();
    _ = Gateway::from_registry();

    let default_zone = Zone::new(0).start();
    Gateway::from_registry().do_send(NewZone {
        id: 0,
        zone: default_zone.clone(),
    });
}
