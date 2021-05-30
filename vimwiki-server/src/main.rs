use flexi_logger::{LevelFilter, LogSpecification, Logger};
use vimwiki_server::{Config, Opt, Program};

#[tokio::main]
async fn main() {
    let opt = Opt::load();
    let config = Config::load(&opt).expect("Failed to load config");

    // Define our logger where everything but our server is not logged and
    // our server's logging is defined by input configuration
    let mut spec = LogSpecification::default(LevelFilter::Off);
    spec.module("vimwiki_server", opt.log_level());

    // Finalize our logger and - if given a directory - mark as logging to
    // files within the specified directory rather than to stderr
    let mut logger = Logger::with(spec.finalize());
    if let Some(log_dir) = opt.log_dir.as_ref() {
        logger = logger.log_to_file().directory(log_dir);
    }

    logger
        .start()
        .unwrap_or_else(|e| panic!("Logger initialization failed with {}", e));

    Program::run(opt, config)
        .await
        .expect("Program failed unexpectedly");
}
