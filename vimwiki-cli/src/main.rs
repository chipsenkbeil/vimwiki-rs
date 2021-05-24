mod subcommand;

mod opt;
use opt::*;

mod vim;
use vim::*;

use structopt::StructOpt;

fn main() {
    let opt = Opt::from_args();
    init_logging(&opt.common);

    match opt.subcommand {
        Subcommand::Convert(cmd) => subcommand::convert(cmd, opt.common),
        Subcommand::Serve(cmd) => subcommand::serve(cmd, opt.common),
    }
}

fn init_logging(opt: &CommonOpt) {
    stderrlog::new()
        .module(module_path!())
        .quiet(opt.quiet)
        .verbosity(opt.verbose)
        .timestamp(opt.log_timestamp.unwrap_or(stderrlog::Timestamp::Off))
        .init()
        .unwrap();
}
