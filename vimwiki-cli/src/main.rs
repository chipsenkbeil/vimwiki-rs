mod ast;
mod css;
mod opt;
mod subcommand;
mod utils;

use ast::Ast;
use opt::*;
use structopt::StructOpt;

fn main() {
    let opt = Opt::from_args();
    init_logging(&opt.common);

    let config =
        utils::load_html_config(opt.common.config.as_deref(), opt.common.merge)
            .expect("Failed to load config");
    let ast = Ast::load(
        &config,
        &opt.common.include,
        &opt.common.cache,
        opt.common.no_cache,
        opt.common.no_prune_cache,
    )
    .expect("Failed to load data");

    match opt.subcommand {
        Subcommand::Convert(cmd) => {
            subcommand::convert(cmd, opt.common, config, ast)
        }
        Subcommand::Serve(cmd) => {
            subcommand::serve(cmd, opt.common, config, ast)
        }
        Subcommand::Inspect(cmd) => {
            subcommand::inspect(cmd, opt.common, config, ast)
        }
    }
    .expect("Command failed unexpectedly");
}

fn init_logging(opt: &CommonOpt) {
    stderrlog::new()
        .module(module_path!())
        .quiet(opt.quiet)
        .verbosity(opt.verbose)
        .timestamp(opt.timestamp.unwrap_or(stderrlog::Timestamp::Off))
        .init()
        .unwrap();
}
