mod graphql;
mod opt;
use opt::*;
mod server;

pub async fn run() {
    use clap::Clap;
    let opt = Opt::parse();
    match opt.mode {
        ModeOpt::Stdin => unimplemented!(),
        ModeOpt::Http => server::run("http://localhost:8000").await,
    }
}
