use log::trace;
use serde::{Deserialize, Serialize};
use sha1::{Digest, Sha1};
use snafu::{ResultExt, Snafu};
use std::{
    path::{Path, PathBuf},
    sync::Arc,
};
use vimwiki::{
    collections::ElementForest, elements::Page, Language, ParseError,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ParsedFile {
    path: PathBuf,
    checksum: String,
    forest: Arc<ElementForest<'static>>,
}

impl ParsedFile {
    /// Returns a reference to the path to the parsed file
    pub fn path(&self) -> &Path {
        self.path.as_path()
    }

    /// Returns a cloned instance of arc for the forest
    pub fn forest(&self) -> Arc<ElementForest<'static>> {
        Arc::clone(&self.forest)
    }

    /// Consumes this instance and attempts to load a new instance.
    ///
    /// A `ParsedFile` maintains a SHA-1 checksum to verify if the data in
    /// the file has changed. If the checksum of the file has not changed, the
    /// file's contents are not parsed and the current instance is returned
    pub async fn reload(self) -> Result<Self, LoadFileError> {
        let (contents, checksum) =
            Self::load_file_with_checksum(self.path()).await?;

        if self.checksum == checksum {
            trace!("{:?} has not changed", self.path());
            Ok(self)
        } else {
            Self::parse_contents_with_checksum(self.path, contents, checksum)
                .await
        }
    }

    /// Loads a vimwiki page by reading the contents from the file at the
    /// specified path and then parsing it into our internal representation.
    pub async fn load(path: impl AsRef<Path>) -> Result<Self, LoadFileError> {
        let (contents, checksum) =
            Self::load_file_with_checksum(path.as_ref()).await?;

        Self::parse_contents_with_checksum(
            path.as_ref().to_path_buf(),
            contents,
            checksum,
        )
        .await
    }

    /// Loads contents of file at specified path, returning a tuple of the
    /// file's contents and a checksum of its contents, or an error if
    /// failing to read the file
    async fn load_file_with_checksum(
        path: impl AsRef<Path>,
    ) -> Result<(String, String), LoadFileError> {
        let contents = tokio::fs::read_to_string(path.as_ref()).await.context(
            ReadFailed {
                path: path.as_ref().to_path_buf(),
            },
        )?;

        let checksum = format!("{:x}", Sha1::digest(contents.as_bytes()));

        Ok((contents, checksum))
    }

    /// Parses the provided file contents and creates a new `ParsedFile`
    /// using the given path, checksum, and resulting parsed forest
    async fn parse_contents_with_checksum(
        path: PathBuf,
        contents: String,
        checksum: String,
    ) -> Result<Self, LoadFileError> {
        let page: Page = Language::from_vimwiki_str(&contents)
            .parse()
            .map_err(|x: ParseError| LoadFileError::ParseFailed {
                path: path.to_path_buf(),
                source: x.to_string(),
            })?;

        let forest = Arc::new(ElementForest::from(page.into_owned()));

        Ok(Self {
            path,
            checksum,
            forest,
        })
    }
}

/// Represents errors that can occur when attempt to load a file
#[derive(Debug, Snafu)]
pub enum LoadFileError {
    #[snafu(
        display("Could not read contents from {}: {}", path.display(), source), 
        visibility = "pub(super)",
    )]
    ReadFailed {
        path: PathBuf,
        source: tokio::io::Error,
    },
    #[snafu(display("Could not parse {}: {}", path.display(), source))]
    ParseFailed {
        path: PathBuf,
        #[snafu(source(false))]
        source: String,
    },
}
