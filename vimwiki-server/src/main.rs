use flexi_logger::{LevelFilter, LogSpecification, Logger};
use vimwiki_server::{Config, Program};

#[tokio::main]
async fn main() {
    let config = Config::load();

    // Define our logger where everything but our server is not logged and
    // our server's logging is defined by input configuration
    let mut spec = LogSpecification::default(LevelFilter::Off);
    spec.module("vimwiki_server", config.log_level());

    Logger::with(spec.finalize())
        .start()
        .unwrap_or_else(|e| panic!("Logger initialization failed with {}", e));

    Program::run(config)
        .await
        .expect("Program failed unexpectedly");
}
