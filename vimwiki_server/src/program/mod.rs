mod config;
mod graphql;
use config::*;
mod state;
use state::*;
mod server;
mod stdin;

pub async fn run() {
    use clap::Clap;
    let config = Config::parse();
    let program = Program::load(&config).await;

    match config.mode {
        Mode::Stdin => stdin::run(program, config).await,
        Mode::Http => server::run(program, config).await,
    }
}
