use vimwiki_cli as cli;

fn main() {
    let opt = cli::load_opt_from_args();
    init_logging(&opt.common);
    if let Err(x) = cli::run(opt) {
        x.exit();
    }
}

fn init_logging(opt: &cli::CommonOpt) {
    stderrlog::new()
        .module("vimwiki_cli")
        .quiet(opt.quiet)
        .verbosity(opt.verbose)
        .timestamp(opt.timestamp.unwrap_or(stderrlog::Timestamp::Off))
        .init()
        .unwrap();
}
