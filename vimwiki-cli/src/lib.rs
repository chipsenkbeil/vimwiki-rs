mod ast;
mod css;
mod opt;
mod subcommand;
mod utils;

use ast::Ast;
use log::*;
use std::path::PathBuf;
use structopt::StructOpt;
use vimwiki::{HtmlConfig, VimwikiConfig};

pub use opt::*;

pub enum ExitCodes {
    FailedToLoadConfig = 1,
    FailedToLoadData = 2,
    SubcommandFailed = 3,
}

impl ExitCodes {
    pub fn exit(self) -> ! {
        std::process::exit(self as i32);
    }
}

/// Loads CLI options from CLI arguments
pub fn load_opt_from_args() -> Opt {
    Opt::from_args()
}

/// Runs the CLI using the provided options, returning success if completed
/// or an error containing the appropriate exit code to return via [`Exitcodes::exit`]
pub fn run(opt: Opt) -> Result<(), ExitCodes> {
    #[cfg(feature = "timekeeper")]
    let timekeeper = opt.common.timekeeper;

    #[cfg(feature = "timekeeper")]
    if timekeeper {
        vimwiki::timekeeper::enable();
    }

    let res = match opt.subcommand {
        Subcommand::Convert(cmd) => {
            let (config, ast) =
                load_html_config_and_ast(&opt.common, &cmd.extra_paths)?;
            subcommand::convert(cmd, opt.common, config, ast)
        }
        Subcommand::Format(cmd) => {
            let config = load_format_config(&opt.common)?;
            subcommand::format(cmd, opt.common, config)
        }
        Subcommand::Serve(cmd) => {
            let (config, ast) =
                load_html_config_and_ast(&opt.common, &cmd.extra_paths)?;
            subcommand::serve(cmd, opt.common, config, ast)
        }
        Subcommand::Inspect(cmd) => {
            let (config, ast) =
                load_html_config_and_ast(&opt.common, &cmd.extra_paths)?;
            subcommand::inspect(cmd, opt.common, config, ast)
        }
    };

    #[cfg(feature = "timekeeper")]
    if timekeeper {
        vimwiki::timekeeper::disable();
        vimwiki::timekeeper::print_report(true);
    }

    if let Err(x) = res {
        error!("Subcommand failed: {}", x);
        return Err(ExitCodes::SubcommandFailed);
    }

    Ok(())
}

fn load_format_config(opt: &CommonOpt) -> Result<VimwikiConfig, ExitCodes> {
    if let Some(path) = opt.config.as_ref() {
        utils::load_format_config(path).map_err(|x| {
            error!("Failed to load config: {}", x);
            ExitCodes::FailedToLoadConfig
        })
    } else {
        Ok(VimwikiConfig::default())
    }
}

fn load_html_config_and_ast(
    opt: &CommonOpt,
    extra_paths: &[PathBuf],
) -> Result<(HtmlConfig, Ast), ExitCodes> {
    let config = match utils::load_html_config(opt, extra_paths) {
        Ok(config) => config,
        Err(x) => {
            error!("Failed to load config: {}", x);
            return Err(ExitCodes::FailedToLoadConfig);
        }
    };

    let ast = match Ast::load(
        &config,
        &opt.include,
        &opt.cache,
        opt.no_cache,
        opt.no_prune_cache,
    ) {
        Ok(ast) => ast,
        Err(x) => {
            error!("Failed to load data: {}", x);
            return Err(ExitCodes::FailedToLoadData);
        }
    };

    Ok((config, ast))
}
