use flexi_logger::{LevelFilter, LogSpecification, Logger};
use vimwiki_server::program;

#[tokio::main]
async fn main() {
    // TODO: We need to parse the program arguments in the binary (here) versus
    //       the library as we need to get the verbosity level to use with
    //       the filter level here -v == info, -vv == debug, -vvv == trace
    Logger::with(LogSpecification::default(LevelFilter::Error).finalize())
        .start()
        .unwrap_or_else(|e| panic!("Logger initialization failed with {}", e));
    program::run().await.expect("Program failed unexpectedly");
}
