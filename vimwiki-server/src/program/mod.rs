pub(crate) mod config;
mod graphql;
use config::*;
mod state;
use state::*;
mod server;
mod stdin;

pub async fn run(config: Config) -> ProgramResult<()> {
    let program = Program::load(&config).await?;

    match config.mode {
        Mode::Stdin => stdin::run(program, config).await,
        Mode::Http => server::run(program, config).await,
    }

    Ok(())
}
