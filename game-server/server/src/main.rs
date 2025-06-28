mod character;
mod core;
mod player;
mod auth;
mod net;
pub mod config;
pub mod server;
mod world;

use protocol;

use clap::Parser;
use server::ServerRunOptions;

#[derive(Parser, Debug)]
struct Options {
    #[arg(long)]
    dry_run: bool,
}

#[tokio::main]
async fn main() {
    let options = Options::parse();
    
    config::Config::init();

    let server_options = ServerRunOptions { 
        dry_run: options.dry_run,
    };
    _ = server::run_server(server_options).await;
}
