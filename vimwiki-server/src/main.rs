use flexi_logger::{LogSpecification, Logger};
use vimwiki_server::{program, Config};

#[tokio::main]
async fn main() {
    let config = Config::load();

    Logger::with(LogSpecification::default(config.log_level()).finalize())
        .start()
        .unwrap_or_else(|e| panic!("Logger initialization failed with {}", e));

    program::run(config)
        .await
        .expect("Program failed unexpectedly");
}
