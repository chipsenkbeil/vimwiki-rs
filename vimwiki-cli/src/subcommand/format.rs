use crate::{CommonOpt, FormatSubcommand};
use log::*;
use std::{collections::HashSet, ffi::OsStr, io, path::Path};
use vimwiki::*;
use walkdir::WalkDir;

pub fn format(
    cmd: FormatSubcommand,
    _opt: CommonOpt,
    config: VimwikiConfig,
) -> io::Result<()> {
    let extensions: HashSet<String> = cmd.extensions.into_iter().collect();

    for path in cmd.paths {
        // Need to make sure the path is legit
        let path = match path.canonicalize() {
            Ok(path) => path,
            Err(x) => {
                error!("{:?} failed to canonicalize: {}", path, x);
                return Err(x);
            }
        };

        // If path is to a file, we want to process it directly regardless of
        // the extension
        if path.is_file() {
            process_file(config.clone(), path.as_path(), cmd.inline)?;

        // Otherwise, we walk the directory
        } else {
            for entry in WalkDir::new(path)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.file_type().is_file())
            {
                if let Some(ext) =
                    entry.path().extension().and_then(OsStr::to_str)
                {
                    if extensions.contains(ext) {
                        process_file(config.clone(), entry.path(), cmd.inline)?;
                    } else {
                        debug!(
                            "{:?} :: skipped due to unrecognized extension ({})!",
                            entry.path(),
                            ext
                        );
                    }
                } else {
                    debug!(
                        "{:?} :: skipped due to lack of extension!",
                        entry.path(),
                    );
                }
            }
        }
    }

    Ok(())
}

fn process_file(
    config: VimwikiConfig,
    input_path: &Path,
    inplace: bool,
) -> io::Result<()> {
    trace!(
        "process_file(_, input_path = {:?}, inplace = {})",
        input_path,
        inplace
    );

    // Load the file's text
    let text = std::fs::read_to_string(input_path)?;

    debug!("{:?} :: file loaded!", input_path);

    // Convert file to a vimwiki page ast
    let page =
        Language::from_vimwiki_str(&text)
            .parse::<Page>()
            .map_err(|x| {
                io::Error::new(io::ErrorKind::InvalidData, x.to_string())
            })?;

    debug!("{:?} :: page parsed!", input_path);

    // Convert page back to vimwiki text
    let text = page.to_vimwiki_string(config).map_err(|x| {
        io::Error::new(io::ErrorKind::InvalidData, x.to_string())
    })?;

    debug!("{:?} :: vimwiki generated!", input_path);

    // If indicated, we replace the file's contents inline
    if inplace {
        info!("Writing to {:?}", input_path);
        std::fs::write(input_path, text)?;

    // Otherwise, print to stdout
    } else {
        println!("{}", text);
    }

    Ok(())
}
