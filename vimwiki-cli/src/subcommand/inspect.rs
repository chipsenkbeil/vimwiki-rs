use crate::{utils, CommonOpt, IndexOrName, InspectSubcommand};
use jsonpath_lib as jsonpath;
use serde::{Deserialize, Serialize};
use std::{ffi::OsStr, fs, io, path::PathBuf};
use vimwiki::{HtmlConfig, HtmlWikiConfig, Language, Page};
use walkdir::WalkDir;

pub fn inspect(cmd: InspectSubcommand, _opt: CommonOpt) -> io::Result<()> {
    let InspectSubcommand {
        config,
        merge,
        include,
        output,
        json_path,
    } = cmd;

    let config = utils::load_html_config(config.as_deref(), merge)?;
    let ast = load_ast(config, &include)?;
    let ast_json = serde_json::to_value(ast).map_err(io::Error::from)?;
    let values =
        jsonpath::select(&ast_json, json_path.as_str()).map_err(|x| {
            io::Error::new(io::ErrorKind::InvalidData, x.to_string())
        })?;

    if let Some(path) = output {
        serde_json::to_writer_pretty(fs::File::create(path)?, &values)
            .map_err(io::Error::from)
    } else {
        serde_json::to_writer_pretty(io::stdout(), &values)
            .map_err(io::Error::from)
    }
}

#[derive(Default, Serialize, Deserialize)]
struct Ast {
    wikis: Vec<Wiki>,
}

#[derive(Default, Serialize, Deserialize)]
struct Wiki {
    index: usize,
    name: Option<String>,
    path: PathBuf,
    files: Vec<WikiFile>,
}

#[derive(Serialize, Deserialize)]
struct WikiFile {
    path: PathBuf,
    data: Page<'static>,
}

fn load_ast(config: HtmlConfig, include: &[IndexOrName]) -> io::Result<Ast> {
    let mut ast = Ast::default();

    // Filter for wikis to process, defaulting to every wiki unless given a
    // filter of wikis to include
    let filter = |(idx, wiki): &(usize, &HtmlWikiConfig)| {
        include.is_empty()
            || include
                .iter()
                .any(|f| f.matches_either(*idx, wiki.name.as_deref()))
    };

    for (index, wiki) in config.wikis.iter().enumerate().filter(filter) {
        ast.wikis.push(Wiki {
            index,
            name: wiki.name.as_ref().cloned(),
            path: wiki.path.to_path_buf(),
            ..Default::default()
        });

        for entry in WalkDir::new(wiki.path.as_path())
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.file_type().is_file()
                    && e.path().extension().and_then(OsStr::to_str)
                        == Some(wiki.ext.as_str())
            })
        {
            let page_path = entry.path().to_path_buf();

            // Load the file and add it to the associated wiki
            let text = fs::read_to_string(page_path.as_path())?;
            let page: Page = Language::from_vimwiki_str(&text)
                .parse::<Page>()
                .map_err(|x| {
                    io::Error::new(io::ErrorKind::InvalidData, x.to_string())
                })?
                .into_owned();

            if let Some(wiki) = ast.wikis.get_mut(index) {
                wiki.files.push(WikiFile {
                    path: page_path,
                    data: page,
                });
            }
        }
    }

    Ok(ast)
}
