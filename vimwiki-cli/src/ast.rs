use crate::IndexOrName;
use log::*;
use serde::{Deserialize, Serialize};
use sha1::{Digest, Sha1};
use std::collections::HashSet;
use std::{
    ffi::OsStr,
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
};
use vimwiki::{HtmlConfig, HtmlWikiConfig, Language, Page};
use walkdir::WalkDir;

#[derive(Default, Serialize, Deserialize)]
pub struct Ast {
    pub wikis: Vec<Wiki>,
}

impl Ast {
    pub fn load(
        config: &HtmlConfig,
        include: &[IndexOrName],
        cache: &Path,
        no_cache: bool,
        no_prune_cache: bool,
    ) -> io::Result<Self> {
        load_ast(config, include, cache, no_cache, no_prune_cache)
    }

    /// Loads a file by either loading it from an external cache file or
    /// manually parsing it (and updating cache)
    ///
    /// If you want to load from the ast, use [`Self::find_file_by_path`]
    /// first prior to this option
    pub fn load_file(
        &mut self,
        path: &Path,
        cache: &Path,
        no_cache: bool,
    ) -> io::Result<&WikiFile> {
        let file = WikiFile::load(path, cache, no_cache)?;

        // Figure out where to put the file
        if let Some(wiki) = self
            .wikis
            .iter_mut()
            .find(|w| path.starts_with(w.path.as_path()))
        {
            wiki.files.push(file);
        }

        self.find_file_by_path(path).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                "Loaded file is now missing",
            )
        })
    }

    /// Finds first file that matches a loaded wiki file path
    pub fn find_file_by_path<P: AsRef<Path>>(
        &self,
        path: P,
    ) -> Option<&WikiFile> {
        self.wikis
            .iter()
            .find_map(|w| w.files.iter().find(|f| f.path == path.as_ref()))
    }
}

#[derive(Default, Serialize, Deserialize)]
pub struct Wiki {
    pub index: usize,
    pub name: Option<String>,
    pub path: PathBuf,
    pub files: Vec<WikiFile>,
}

#[derive(Serialize, Deserialize)]
pub struct WikiFile {
    pub path: PathBuf,
    pub checksum: String,
    pub data: Page<'static>,
}

impl WikiFile {
    pub fn load(path: &Path, cache: &Path, no_cache: bool) -> io::Result<Self> {
        load_wiki_file(path, cache, no_cache)
    }
}

fn load_ast(
    config: &HtmlConfig,
    include: &[IndexOrName],
    cache: &Path,
    no_cache: bool,
    no_prune_cache: bool,
) -> io::Result<Ast> {
    trace!(
        "load_ast(_, include = {:?}, cache = {:?}, no_cache = {}, no_prune_cache = {})",
        include,
        cache,
        no_cache,
        no_prune_cache,
    );

    let mut ast = Ast::default();

    // If working with the cache, create the directory for it to make sure
    // it is available
    if !no_cache {
        fs::create_dir_all(cache)?;
    }

    // Filter for wikis to process, defaulting to every wiki unless given a
    // filter of wikis to include
    let filter = |(idx, wiki): &(usize, &HtmlWikiConfig)| {
        include.is_empty()
            || include
                .iter()
                .any(|f| f.matches_either(*idx, wiki.name.as_deref()))
    };

    for (index, wiki) in config.wikis.iter().enumerate().filter(filter) {
        trace!(
            "Loading wiki @ index = {} | name = {:?} from {:?}",
            index,
            wiki.name,
            wiki.path
        );
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
            let file = load_wiki_file(entry.path(), cache, no_cache)?;
            if let Some(wiki) = ast.wikis.get_mut(index) {
                wiki.files.push(file);
            }
        }
    }

    // Prune cache of any file not listed
    if !no_prune_cache && !no_cache {
        let checksums: HashSet<&str> = ast
            .wikis
            .iter()
            .flat_map(|w| w.files.as_slice())
            .map(|f| f.checksum.as_str())
            .collect();
        trace!("Pruning cache down to {} files", checksums.len());

        let iter = WalkDir::new(cache)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file());

        for entry in iter {
            match entry.file_name().to_str() {
                Some(name) if !checksums.contains(name) => {
                    trace!("Removing cache file {}", name);
                    fs::remove_file(entry.path())?;
                }
                None => {
                    trace!("Removing corrupt cache file @ {:?}", entry.path());
                    fs::remove_file(entry.path())?;
                }
                _ => {}
            }
        }
    }

    Ok(ast)
}

fn load_wiki_file(
    path: &Path,
    cache: &Path,
    no_cache: bool,
) -> io::Result<WikiFile> {
    trace!(
        "load_wiki_file(path = {:?}, cache = {:?}, no_cache = {})",
        path,
        cache,
        no_cache
    );

    // Load the file contents and calculate the checksum to see how it
    // compares to our cached version
    let text = fs::read_to_string(path)?;
    trace!("{:?} :: text loaded", path);

    let checksum = format!("{:x}", Sha1::digest(text.as_bytes()));
    trace!("{:?} :: checksum = {}", path, checksum);

    let cached_page: Option<Page> = if !no_cache {
        let cached_page_path = cache.join(checksum.as_str());
        trace!("{:?} :: checking cache at {:?}", path, cached_page_path);

        // If a checksum file exists for the current checksum, then we can
        // just load that as it should match what we want
        if cached_page_path.exists() {
            let cached_page: io::Result<Page> =
                fs::File::open(cached_page_path.as_path())
                    .map(io::BufReader::new)
                    .and_then(|reader| {
                        serde_json::from_reader(reader).map_err(io::Error::from)
                    });

            match cached_page {
                Ok(page) => {
                    trace!("{:?} :: loaded from cache", path);
                    Some(page)
                }
                Err(x) => {
                    error!("{:?} :: cache corrupted: {}", path, x);
                    if let Err(x) = fs::remove_file(cached_page_path) {
                        error!(
                            "{:?} :: failed to remove corrupted cache: {}",
                            path, x
                        );
                    }
                    None
                }
            }
        } else {
            trace!("{:?} :: no cache found", path);
            None
        }
    } else {
        trace!("{:?} :: skipping cache", path);
        None
    };

    let has_cached_page = cached_page.is_some();

    // Only parse a page fresh if checksum is different
    let page: Page = if let Some(page) = cached_page {
        page
    } else {
        Language::from_vimwiki_str(&text)
            .parse::<Page>()
            .map(Page::into_owned)
            .map_err(|x| {
                io::Error::new(io::ErrorKind::InvalidData, x.to_string())
            })?
    };

    // Update our cache with the new file; old files get cleaned later
    if !has_cached_page {
        match fs::File::create(cache.join(checksum.as_str())) {
            Ok(file) => {
                let mut writer = io::BufWriter::new(file);
                match serde_json::to_writer_pretty(&mut writer, &page) {
                    Ok(()) => {
                        if let Err(x) = writer.flush() {
                            trace!(
                                "{:?} :: failed to write cache: {}",
                                path,
                                x
                            );
                        } else {
                            trace!("{:?} :: wrote cache", path);
                        }
                    }
                    Err(x) => {
                        error!("{:?} :: failed to write cache: {}", path, x);
                    }
                }
            }
            Err(x) => {
                error!("{:?} :: open cache for write failed: {}", path, x);
            }
        }
    }

    Ok(WikiFile {
        path: path.to_path_buf(),
        checksum,
        data: page,
    })
}
