mod ast;
mod css;
mod opt;
mod subcommand;
mod utils;

use ast::Ast;
use log::*;
use opt::*;
use structopt::StructOpt;

enum ExitCodes {
    FailedToLoadConfig = 1,
    FailedToLoadData = 2,
    SubcommandFailed = 3,
}

impl ExitCodes {
    pub fn exit(self) -> ! {
        std::process::exit(self as i32);
    }
}

fn main() {
    let opt = Opt::from_args();
    init_logging(&opt.common);

    let config = match utils::load_html_config(
        &opt.common,
        opt.subcommand.extra_paths(),
    ) {
        Ok(config) => config,
        Err(x) => {
            error!("Failed to load config: {}", x);
            ExitCodes::FailedToLoadConfig.exit();
        }
    };

    let ast = match Ast::load(
        &config,
        &opt.common.include,
        &opt.common.cache,
        opt.common.no_cache,
        opt.common.no_prune_cache,
    ) {
        Ok(ast) => ast,
        Err(x) => {
            error!("Failed to load data: {}", x);
            ExitCodes::FailedToLoadData.exit();
        }
    };

    let res = match opt.subcommand {
        Subcommand::Convert(cmd) => {
            subcommand::convert(cmd, opt.common, config, ast)
        }
        Subcommand::Serve(cmd) => {
            subcommand::serve(cmd, opt.common, config, ast)
        }
        Subcommand::Inspect(cmd) => {
            subcommand::inspect(cmd, opt.common, config, ast)
        }
    };

    if let Err(x) = res {
        error!("Subcommand failed: {}", x);
        ExitCodes::SubcommandFailed.exit();
    }
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
