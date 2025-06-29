// mod character;
// mod core;
// mod player;
// mod auth;
mod net;
// mod config;
// mod server;
// mod world;

use actix::prelude::*;
use clap::Parser;
use net::{
    authenticator::Authenticator,
    game_listener::GameListener,
};
use protocol;
// use server::ServerRunOptions;

#[derive(Parser, Debug)]
struct Options {
    #[arg(long)]
    dry_run: bool,
}

#[actix::main]
async fn main() {
    // let options = Options::parse();
    // config::Config::init();

    // let server_options = ServerRunOptions {
    //     dry_run: options.dry_run,
    // };
    // _ = server::run_server(server_options).await;

    let authenticator_addr = Authenticator {}.start();
    _ = GameListener::new(6400, authenticator_addr).start();

    tokio::signal::ctrl_c().await.unwrap();
}
