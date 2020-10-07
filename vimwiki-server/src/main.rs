use flexi_logger::{LogSpecification, Logger};
use vimwiki_server::{program, Config};

#[tokio::main]
async fn main() {
    let config = Config::load();

    // TODO: We need to parse the program arguments in the binary (here) versus
    //       the library as we need to get the verbosity level to use with
    //       the filter level here -v == info, -vv == debug, -vvv == trace
    Logger::with(LogSpecification::default(config.log_level()).finalize())
        .start()
        .unwrap_or_else(|e| panic!("Logger initialization failed with {}", e));

    program::run(config)
        .await
        .expect("Program failed unexpectedly");
}
