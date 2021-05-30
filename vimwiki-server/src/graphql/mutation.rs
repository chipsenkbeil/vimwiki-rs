use crate::data::*;
use log::trace;

pub struct Mutation;

#[async_graphql::Object]
impl Mutation {
    /// Imports/re-imports a wiki from the specified path
    async fn import_wiki(
        &self,
        path: String,
        index: u32,
        name: Option<String>,
        #[graphql(default = "wiki")] ext: String,
    ) -> async_graphql::Result<Wiki> {
        trace!(
            "import_wiki(path: {:?}, index: {}, name: {:?})",
            path,
            index,
            name
        );
        Wiki::load(
            index as usize,
            path,
            name,
            ext.as_str(),
            |_| {},
            |_, _, _| {},
            |_| {},
        )
        .await
    }

    /// Imports/re-imports a standalone wiki file from the specified path
    async fn import_file(
        &self,
        path: String,
    ) -> async_graphql::Result<ParsedFile> {
        trace!("import_file(path: {:?})", path);
        ParsedFile::load(path).await
    }

    /// Creates a new vimwiki file at the specified path using the given text
    /// as the contents of the file. The contents will be parsed and loaded
    /// into the server. By default, if the file already exists, it will not
    /// be overwritten and instead will return an error.
    async fn create_file(
        &self,
        path: String,
        contents: String,
        #[graphql(default)] overwrite: bool,
    ) -> async_graphql::Result<ParsedFile> {
        trace!(
            "create_file(path: {:?}, contents: {:?}, overwrite: {})",
            path,
            contents,
            overwrite
        );
        ParsedFile::create(path, contents, overwrite).await
    }
}
