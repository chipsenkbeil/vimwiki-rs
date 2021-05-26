use crate::{css, utils, CommonOpt, ConvertSubcommand};
use log::*;
use std::{ffi::OsStr, io, path::Path};
use vimwiki::*;
use walkdir::WalkDir;

pub fn convert(cmd: ConvertSubcommand, _opt: CommonOpt) -> io::Result<()> {
    let html_config = handle_result(
        FailType::new("Unable to load html config", cmd.fail_fast, || {
            HtmlConfig::default()
        }),
        utils::load_html_config(cmd.config.as_deref(), cmd.merge),
    );

    // Filter for wikis to process, defaulting to every wiki unless given a
    // filter of wikis to include
    let filter = |(idx, wiki): &(usize, &HtmlWikiConfig)| {
        cmd.include.is_empty()
            || cmd
                .include
                .iter()
                .any(|f| f.matches_either(*idx, wiki.name.as_deref()))
    };

    // Process all wikis that match the given filters if we aren't given
    // specific files/wikis to convert
    if cmd.files.is_empty() {
        for (_, wiki) in html_config.wikis.iter().enumerate().filter(filter) {
            let msg = format!(
                "Failed to process wiki at {}",
                wiki.path.to_string_lossy()
            );

            handle_result(
                FailType::new(msg.as_str(), cmd.fail_fast, || ()),
                process_path(
                    html_config.clone(),
                    wiki.path.as_path(),
                    cmd.stdout,
                    &cmd.extensions,
                ),
            );

            // If writing to a file, we want to make sure there is a css
            // file generated if necessary
            if !cmd.stdout && cmd.include_vimwiki_css {
                let css_path = wiki.path_html.join("style.css");
                debug!("Writing css to {:?}", css_path);
                std::fs::write(css_path, css::DEFAULT_STYLE_FILE)?;
            }
        }
    }

    // Additionally, we process any directories & files provided adhoc
    for path in cmd.files {
        // Need to make sure the path is legit
        let path = if let Ok(path) = path.canonicalize() {
            path
        } else {
            error!("Failed to canonicalize path: {:?}", path);
            if cmd.fail_fast {
                panic!();
            } else {
                continue;
            }
        };

        let msg =
            format!("Failed to process dir/file at {}", path.to_string_lossy());
        handle_result(
            FailType::new(msg.as_str(), cmd.fail_fast, || ()),
            process_path(
                html_config.clone(),
                path.as_path(),
                cmd.stdout,
                &cmd.extensions,
            ),
        );

        // If writing to a file, we want to make sure there is a css
        // file generated if necessary
        if !cmd.stdout && cmd.include_vimwiki_css {
            let wiki = html_config.runtime.to_tmp_wiki();
            let css_path = wiki.path_html.join("style.css");
            debug!("Writing css to {:?}", css_path);
            std::fs::write(css_path, css::DEFAULT_STYLE_FILE)?;
        }
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
        (FailType::Fast(msg), Err(x)) => {
            error!("{}: {}", msg, x);
            panic!()
        }
        (FailType::Continue(msg, make_data), Err(x)) => {
            error!("{}: {}", msg, x);
            make_data()
        }
    }
}

fn process_path(
    mut html_config: HtmlConfig,
    input_path: &Path,
    stdout: bool,
    exts: &[String],
) -> io::Result<()> {
    trace!(
        "process_path(_, input_path = {:?}, stdout = {}, exts = {:?})",
        input_path,
        stdout,
        exts
    );

    // See if we have a wiki that already contains this path and, if not, we
    // want to inject a temporary wiki whose path matches the input if a
    // directory or matches a file's parent directory
    if html_config.find_wiki_index_by_path(input_path).is_none() {
        debug!("Creating temporary wiki for {:?}", input_path);
        if input_path.is_dir() {
            html_config.wikis.push(HtmlWikiConfig {
                path: input_path.to_path_buf(),
                ..Default::default()
            })
        } else if let Some(parent) = input_path.parent() {
            html_config.wikis.push(HtmlWikiConfig {
                path: parent.to_path_buf(),
                ..Default::default()
            })
        }
    }

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

        // Figure out which wiki this page belongs to (if any)
        let wiki_index =
            html_config.find_wiki_index_by_path(page_path.as_path());
        debug!("{:?}: Wiki {:?}", page_path, wiki_index);

        html_config.map_runtime(|mut rt| {
            rt.page = page_path.to_path_buf();
            rt.wiki_index = wiki_index;
            rt
        });

        process_file(html_config, page_path.as_path(), stdout)?;
    }

    Ok(())
}

fn process_file(
    html_config: HtmlConfig,
    input_path: &Path,
    stdout: bool,
) -> io::Result<()> {
    trace!(
        "process_file(_, input_path = {:?}, stdout = {})",
        input_path,
        stdout
    );

    // Go ahead and figure out the necessary wiki if we need it so that we
    // don't need to clone our entire config later
    let maybe_wiki = if !stdout {
        html_config.find_wiki_by_path(input_path).cloned()
    } else {
        None
    };

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

    // If told to print to stdout, do so
    if stdout {
        println!("{}", html);

    // Otherwise, we generate files based on resolved output paths
    } else {
        let path = maybe_wiki
            .unwrap_or_default()
            .make_output_path(input_path, "html");

        debug!("Writing to {:?}", path);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, html)?;
    }

    Ok(())
}
