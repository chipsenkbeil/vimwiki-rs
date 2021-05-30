use crate::{css, Ast, CommonOpt, ConvertSubcommand};
use log::*;
use std::{ffi::OsStr, io, path::Path};
use vimwiki::*;
use walkdir::WalkDir;

pub fn convert(
    cmd: ConvertSubcommand,
    opt: CommonOpt,
    config: HtmlConfig,
    mut ast: Ast,
) -> io::Result<()> {
    // Process all wikis that match the given filters if we aren't given
    // specific files/wikis to convert
    if cmd.files.is_empty() {
        for (_, wiki) in
            config.wikis.iter().enumerate().filter(|(idx, wiki)| {
                opt.filter_by_wiki_idx_and_name(*idx, wiki.name.as_deref())
            })
        {
            process_path(
                config.clone(),
                &mut ast,
                wiki.path.as_path(),
                opt.cache.as_path(),
                opt.no_cache,
                cmd.stdout,
                &wiki.ext,
            )?;

            // If writing to a file, we want to make sure there is a css
            // file generated if necessary
            if !cmd.stdout && cmd.include_vimwiki_css {
                let css_path =
                    wiki.path_html.join(HtmlWikiConfig::default_css_name());
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
            panic!();
        };

        process_path(
            config.clone(),
            &mut ast,
            path.as_path(),
            opt.cache.as_path(),
            opt.no_cache,
            cmd.stdout,
            &HtmlWikiConfig::default_ext(),
        )?;

        // If writing to a file, we want to make sure there is a css
        // file generated if necessary
        if !cmd.stdout && cmd.include_vimwiki_css {
            let wiki = config.runtime.to_tmp_wiki();
            let css_path = wiki.path_html.join("style.css");
            debug!("Writing css to {:?}", css_path);
            std::fs::write(css_path, css::DEFAULT_STYLE_FILE)?;
        }
    }

    Ok(())
}

fn process_path(
    mut config: HtmlConfig,
    ast: &mut Ast,
    input_path: &Path,
    cache: &Path,
    no_cache: bool,
    stdout: bool,
    ext: &str,
) -> io::Result<()> {
    trace!(
        "process_path(_, input_path = {:?}, stdout = {}, ext = {})",
        input_path,
        stdout,
        ext
    );

    // See if we have a wiki that already contains this path and, if not, we
    // want to inject a temporary wiki whose path matches the input if a
    // directory or matches a file's parent directory
    if config.find_wiki_index_by_path(input_path).is_none() {
        debug!("Creating temporary wiki for {:?}", input_path);
        if input_path.is_dir() {
            config.wikis.push(HtmlWikiConfig {
                path: input_path.to_path_buf(),
                ..Default::default()
            })
        } else if let Some(parent) = input_path.parent() {
            config.wikis.push(HtmlWikiConfig {
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
                && e.path().extension().and_then(OsStr::to_str) == Some(ext)
        })
    {
        let mut config = config.clone();
        let page_path = entry.path().to_path_buf();

        // Figure out which wiki this page belongs to (if any)
        let wiki_index = config.find_wiki_index_by_path(page_path.as_path());
        debug!("{:?}: Wiki {:?}", page_path, wiki_index);

        config.map_runtime(|mut rt| {
            rt.page = page_path.to_path_buf();
            rt.wiki_index = wiki_index;
            rt
        });

        process_file(
            config,
            ast,
            page_path.as_path(),
            cache,
            no_cache,
            stdout,
        )?;
    }

    Ok(())
}

fn process_file(
    config: HtmlConfig,
    ast: &mut Ast,
    input_path: &Path,
    cache: &Path,
    no_cache: bool,
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
        config.find_wiki_by_path(input_path).cloned()
    } else {
        None
    };

    // If we already have a file loaded at this path, use it
    let html = if let Some(file) = ast.find_file_by_path(input_path) {
        trace!("{:?} :: loaded from cache!", input_path);

        file.data.to_html_page(config).map_err(|x| {
            io::Error::new(io::ErrorKind::InvalidData, x.to_string())
        })?

    // Otherwise, we need to load the file
    } else {
        let file = ast.load_file(input_path, cache, no_cache)?;

        file.data.to_html_page(config).map_err(|x| {
            io::Error::new(io::ErrorKind::InvalidData, x.to_string())
        })?
    };
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
