mod vim;
use vim::VimVar;

use log::*;
use std::{
    ffi::OsStr,
    io,
    path::{Path, PathBuf},
};
use structopt::StructOpt;
use vimwiki::*;
use walkdir::WalkDir;

#[derive(Debug, StructOpt)]
struct Opt {
    /// Verbose mode (-v, -vv, -vvv, etc.)
    #[structopt(short, long, parse(from_occurrences))]
    verbose: usize,

    /// Quiet mode
    #[structopt(short, long)]
    quiet: bool,

    /// Timestamp for logging (sec, ms, ns, none)
    #[structopt(long)]
    log_timestamp: Option<stderrlog::Timestamp>,

    /// Path to html config file for html output (otherwise uses default settings)
    #[structopt(long)]
    html_config: Option<PathBuf>,

    /// If provided, will fail immediately when encountering an error instead
    /// of continuing
    #[structopt(long)]
    fail_fast: bool,

    /// Extensions of files to parse when loading from wikis or arbitrary
    /// directories
    #[structopt(long, default_value = "wiki")]
    ext: Vec<String>,

    /// Write output to FILE instead of stdout; if processing multiple files,
    /// can use `{}` as a filler for the file name such as `{}.html`
    #[structopt(short, long, parse(from_os_str))]
    output: Option<PathBuf>,

    /// Files (or directories) to process
    #[structopt(name = "FILE", parse(from_os_str))]
    files: Vec<PathBuf>,
}

fn main() {
    let opt = Opt::from_args();

    // Initialize logging based on verbosity, quietness, and timestamp attr
    stderrlog::new()
        .module(module_path!())
        .quiet(opt.quiet)
        .verbosity(opt.verbose)
        .timestamp(opt.log_timestamp.unwrap_or(stderrlog::Timestamp::Off))
        .init()
        .unwrap();

    let html_config = handle_result(
        FailType::new("Unable to load html config", opt.fail_fast, || {
            HtmlConfig::default()
        }),
        load_html_config(opt.html_config.as_deref()),
    );

    // If no specific files provided, we will process all wikis
    if opt.files.is_empty() {
        for wiki in html_config.wikis.iter() {
            let msg = format!(
                "Failed to process wiki at {}",
                wiki.path.to_string_lossy()
            );
            handle_result(
                FailType::new(msg.as_str(), opt.fail_fast, || ()),
                process_path(
                    html_config.clone(),
                    wiki.path.as_path(),
                    opt.output.as_deref(),
                    &opt.ext,
                ),
            );
        }

        // Otherwise, we process each of the files individually
    } else {
        for path in opt.files {
            let msg = format!(
                "Failed to process dir/file at {}",
                path.to_string_lossy()
            );
            handle_result(
                FailType::new(msg.as_str(), opt.fail_fast, || ()),
                process_path(
                    html_config.clone(),
                    path.as_path(),
                    opt.output.as_deref(),
                    &opt.ext,
                ),
            );
        }
    }
}

enum FailType<'a, T, F: FnOnce() -> T> {
    Fast(&'a str),
    Continue(&'a str, F),
}

impl<'a, T, F: FnOnce() -> T> FailType<'a, T, F> {
    pub fn new(msg: &'a str, fail_fast: bool, f: F) -> Self {
        if fail_fast {
            Self::Fast(msg)
        } else {
            Self::Continue(msg, f)
        }
    }
}

fn handle_result<T, F: FnOnce() -> T, E: std::error::Error>(
    ft: FailType<'_, T, F>,
    x: Result<T, E>,
) -> T {
    match (ft, x) {
        (_, Ok(x)) => x,
        (FailType::Fast(msg), Err(x)) => panic!("{}: {}", msg, x),
        (FailType::Continue(msg, make_data), Err(x)) => {
            error!("{}: {}", msg, x);
            make_data()
        }
    }
}

fn process_path(
    html_config: HtmlConfig,
    input_path: &Path,
    output_path: Option<&Path>,
    exts: &[String],
) -> io::Result<()> {
    trace!(
        "process_path(_, {:?}, {:?}, {:?})",
        input_path,
        output_path,
        exts
    );
    // Walk through all entries in directory (or singular file), processing
    // each file as it is encountered that has a valid file extension
    for entry in WalkDir::new(input_path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.file_type().is_file()
                && exts.iter().any(|ext| {
                    e.path().extension().and_then(OsStr::to_str)
                        == Some(ext.as_str())
                })
        })
    {
        let mut html_config = html_config.clone();
        let page_path = entry.path().to_path_buf();

        // Figure out which wiki this page belongs to, if any
        // TODO: Do we need to worry about wikis nested in other wikis?
        let wiki_index =
            html_config
                .wikis
                .iter()
                .enumerate()
                .find_map(|(idx, wiki)| {
                    if page_path.starts_with(wiki.path.as_path()) {
                        Some(idx)
                    } else {
                        None
                    }
                });
        debug!("{:?}: Wiki {:?}", page_path, wiki_index);

        html_config.map_runtime(|mut rt| {
            rt.page = page_path.to_path_buf();
            rt.wiki_index = wiki_index;
            rt
        });

        process_file(html_config, page_path.as_path(), output_path)?;
    }

    Ok(())
}

fn process_file(
    html_config: HtmlConfig,
    input_path: &Path,
    output_path: Option<&Path>,
) -> io::Result<()> {
    trace!("process_file(_, {:?}, {:?})", input_path, output_path);

    let text = std::fs::read_to_string(input_path)?;
    trace!("{:?} :: text loaded!", input_path);

    let page: Page = Language::from_vimwiki_str(text.as_str())
        .parse()
        .map_err(|x: ParseError<'_>| {
            io::Error::new(io::ErrorKind::InvalidData, x.to_string())
        })?;
    trace!("{:?} :: page parsed!", input_path);

    let html = page.to_html_page(html_config).map_err(|x| {
        io::Error::new(io::ErrorKind::InvalidData, x.to_string())
    })?;
    trace!("{:?} :: html generated!", input_path);

    // If we are given an output path to use, then process it as destination
    if let Some(out_path) = output_path {
        let out_path_str = out_path.to_string_lossy();
        let path = if out_path_str.contains("{}") {
            let name = input_path
                .file_stem()
                .ok_or_else(|| {
                    io::Error::new(
                        io::ErrorKind::Other,
                        "Input is not a file with a name that can be used for {}"
                    )
                })?
                .to_string_lossy();
            PathBuf::from(out_path_str.replace("{}", name.as_ref()))
        } else {
            out_path.to_path_buf()
        };

        debug!("Writing to {:?}", path);
        std::fs::write(path, html)?;

    // Otherwise, output to stdout by default
    } else {
        println!("{}", html);
    }

    Ok(())
}

fn load_html_config<'a, I: Into<Option<&'a Path>>>(
    path: I,
) -> io::Result<HtmlConfig> {
    let maybe_path = path.into();
    trace!("load_html_config({:?})", maybe_path);

    let mut html_config: HtmlConfig = if let Some(path) = maybe_path {
        let config_string = std::fs::read_to_string(path)?;
        toml::from_str(config_string.as_str())?
    } else {
        HtmlConfig::default()
    };

    // If html config has no wikis, attempt to load wikis from vim
    if html_config.wikis.is_empty() {
        // We attempt to load and parse our wiki content now, and if it fails
        // then we report over stderr and continue
        match load_vimwiki_list() {
            Ok(wikis) => html_config.wikis = wikis,
            Err(x) => {
                error!("Failed to load vimwiki_list from vim/neovim: {}", x)
            }
        }
    }

    Ok(html_config)
}

/// Loads g:vimwiki_list from vim/neovim and then attempts to convert it into
/// a structured html wiki config
fn load_vimwiki_list() -> std::io::Result<Vec<HtmlWikiConfig>> {
    trace!("load_vimwiki_list()");
    let vimwiki_list_json = VimVar::get_global("vimwiki_list")?;
    serde_json::from_value(vimwiki_list_json).map_err(Into::into)
}
