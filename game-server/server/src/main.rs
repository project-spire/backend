mod character;
mod core;
mod item;
mod physics;
mod player;
mod world;
mod auth;
mod station;

use protocol;

use clap::Parser;
use crate::core::server::ServerRunOptions;

#[derive(Parser, Debug)]
struct Options {
    #[arg(long)]
    dry_run: bool,
}

#[tokio::main]
async fn main() {
    let options = Options::parse();
    
    core::config::Config::init();

    let server_options = ServerRunOptions { 
        dry_run: options.dry_run,
    };
    _ = core::server::run_server(server_options).await;
}
