use crate::{utils, CommonOpt, ConvertSubcommand};
use log::*;
use std::{
    ffi::OsStr,
    io,
    path::{Path, PathBuf},
};
use vimwiki::*;
use walkdir::WalkDir;

pub fn convert(cmd: ConvertSubcommand, _opt: CommonOpt) -> io::Result<()> {
    let html_config = handle_result(
        FailType::new("Unable to load html config", cmd.fail_fast, || {
            HtmlConfig::default()
        }),
        load_html_config(cmd.config.as_deref()),
    );

    // If specified, we load all wikis and process them
    if cmd.all {
        for wiki in html_config.wikis.iter() {
            let msg = format!(
                "Failed to process wiki at {}",
                wiki.path.to_string_lossy()
            );
            handle_result(
                FailType::new(msg.as_str(), cmd.fail_fast, || ()),
                process_path(
                    html_config.clone(),
                    wiki.path.as_path(),
                    cmd.output.as_deref(),
                    &cmd.extensions,
                ),
            );
        }
    }

    // Additionally, we process any directories & files provided adhoc
    for path in cmd.files {
        let msg =
            format!("Failed to process dir/file at {}", path.to_string_lossy());
        handle_result(
            FailType::new(msg.as_str(), cmd.fail_fast, || ()),
            process_path(
                html_config.clone(),
                path.as_path(),
                cmd.output.as_deref(),
                &cmd.extensions,
            ),
        );
    }

    Ok(())
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

        // TODO: output_path will currently produce a flat file structure for
        //       vimwiki files, even with {}.html
        //
        //       Instead, we need to do a couple of things:
        //       1. Augment output_path by calculating a relative path from the
        //          input_path and the entry.path() to see how far in from the
        //          input_path the entry is
        //       2. Take the relative path from #1 and join it to the output_path
        //          to produce the actual file path
        //       3. Replace the extension in the output path with html
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
        match utils::load_vimwiki_list() {
            Ok(wikis) => html_config.wikis = wikis,
            Err(x) => {
                error!("Failed to load vimwiki_list from vim/neovim: {}", x)
            }
        }
    }

    Ok(html_config)
}