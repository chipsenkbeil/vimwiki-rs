use super::{graphql, Program};
use std::{
    collections::HashSet,
    path::{Path, PathBuf},
};
use tokio::sync::Mutex;

/// Represents a wiki and its associated files
#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct Wiki {
    pub(super) index: usize,
    pub(super) name: Option<String>,
    pub(super) path: PathBuf,
    pub(super) files: HashSet<PathBuf>,
}

impl Wiki {
    #[inline]
    pub fn as_index(&self) -> usize {
        self.index
    }

    #[inline]
    pub fn as_name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    #[inline]
    pub fn as_path(&self) -> &Path {
        self.path.as_path()
    }

    #[inline]
    pub fn files(&self) -> impl Iterator<Item = &Path> {
        self.files.iter().map(PathBuf::as_path)
    }
}

#[async_graphql::Object]
impl Wiki {
    async fn index(&self) -> u32 {
        self.index as u32
    }

    async fn name(&self) -> Option<&String> {
        self.name.as_ref()
    }

    async fn path(&self) -> String {
        self.path.to_string_lossy().to_string()
    }

    /// Returns all pages within the wiki
    async fn pages(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> Vec<graphql::elements::Page> {
        let mut pages = Vec::new();
        for path in self.files() {
            // TODO: Is this needed, or would the wiki have absolute paths?
            let full_path = self.path.join(path);

            if let Some(page) = ctx
                .data_unchecked::<Mutex<Program>>()
                .lock()
                .await
                .graphql_page(full_path)
            {
                pages.push(page);
            }
        }
        pages
    }

    /// Returns the page with the specific path within the wiki
    async fn page(
        &self,
        ctx: &async_graphql::Context<'_>,
        path: String,
    ) -> Option<graphql::elements::Page> {
        ctx.data_unchecked::<Mutex<Program>>()
            .lock()
            .await
            .graphql_page(self.path.join(path))
    }
}
