use crate::data::*;
use entity::{TypedPredicate as P, *};
use log::trace;
use paste::paste;
use sha1::{Digest, Sha1};
use vimwiki::{elements as v, Language, ParseError};

#[inline]
fn gql_db() -> async_graphql::Result<DatabaseRc> {
    WeakDatabaseRc::upgrade(&entity::global::db())
        .ok_or(async_graphql::Error::new("Database unavailable"))
}

/// Represents the query-portion of the GraphQL schema
pub struct Query;

macro_rules! ent_query_fns {
    ($data:ident; untyped) => {
        paste! {
            #[doc = "Queries for instances of " $data " that match the filter"]
            async fn [<$data:snake s>](
                &self,
                filter: entity::ext::async_graphql::GqlEntFilter,
            ) -> async_graphql::Result<Vec<[<$data:camel>]>> {
                gql_db()?.find_all_typed::<[<$data:camel>]>(filter.into())
                    .map_err(|x| async_graphql::Error::new(x.to_string()))
            }
        }

        ent_query_fns!($data; id);
    };
    ($data:ident; typed) => {
        paste! {
            #[doc = "Queries for instances of " $data " that match the filter"]
            async fn [<$data:snake s>](
                &self,
                filter: [<Gql $data:camel Filter>],
            ) -> async_graphql::Result<Vec<[<$data:camel>]>> {
                gql_db()?.find_all_typed::<[<$data:camel>]>(filter.into())
                    .map_err(|x| async_graphql::Error::new(x.to_string()))
            }
        }

        ent_query_fns!($data; id);
    };
    ($data:ident; id) => {
        paste! {
            #[doc = "Query for single instance of " $data " by its id"]
            async fn [<$data:snake>](
                &self,
                id: entity::Id,
            ) -> async_graphql::Result<Option<[<$data:camel>]>> {
                gql_db()?.get_typed::<[<$data:camel>]>(id)
                    .map_err(|x| async_graphql::Error::new(x.to_string()))
            }
        }
    };
}

#[async_graphql::Object]
impl Query {
    /// Query for single instance of any ent by its id
    async fn ent(&self, id: Id) -> async_graphql::Result<Option<Box<dyn Ent>>> {
        gql_db()?
            .get(id)
            .map_err(|x| async_graphql::Error::new(x.to_string()))
    }

    /// Queries for instances of any ent that matches the given filter
    async fn ents(
        &self,
        filter: entity::ext::async_graphql::GqlEntFilter,
    ) -> async_graphql::Result<Vec<Box<dyn Ent>>> {
        gql_db()?
            .find_all(filter.into())
            .map_err(|x| async_graphql::Error::new(x.to_string()))
    }

    ent_query_fns!(Wiki; typed);
    ent_query_fns!(ParsedFile; typed);

    ent_query_fns!(Page; typed);
    ent_query_fns!(Element; untyped);
    ent_query_fns!(BlockElement; untyped);
    ent_query_fns!(InlineBlockElement; untyped);
    ent_query_fns!(InlineElement; untyped);

    ent_query_fns!(Blockquote; typed);
    ent_query_fns!(DefinitionList; typed);
    ent_query_fns!(Term; typed);
    ent_query_fns!(Definition; typed);
    ent_query_fns!(Divider; typed);
    ent_query_fns!(Header; typed);
    ent_query_fns!(List; typed);
    ent_query_fns!(ListItem; typed);
    ent_query_fns!(ListItemContent; untyped);
    ent_query_fns!(ListItemAttributes; typed);
    ent_query_fns!(MathBlock; typed);
    ent_query_fns!(Paragraph; typed);
    ent_query_fns!(Placeholder; untyped);
    ent_query_fns!(PlaceholderTitle; typed);
    ent_query_fns!(PlaceholderNoHtml; typed);
    ent_query_fns!(PlaceholderTemplate; typed);
    ent_query_fns!(PlaceholderDate; typed);
    ent_query_fns!(PlaceholderOther; typed);
    ent_query_fns!(PreformattedText; typed);
    ent_query_fns!(Table; typed);
    ent_query_fns!(Row; untyped);
    ent_query_fns!(ContentRow; typed);
    ent_query_fns!(DividerRow; typed);
    ent_query_fns!(Cell; untyped);
    ent_query_fns!(ContentCell; typed);
    ent_query_fns!(SpanLeftCell; typed);
    ent_query_fns!(SpanAboveCell; typed);

    ent_query_fns!(Text; typed);
    ent_query_fns!(DecoratedText; typed);
    ent_query_fns!(DecoratedTextContent; untyped);
    ent_query_fns!(Keyword; typed);
    ent_query_fns!(Link; untyped);
    ent_query_fns!(WikiLink; typed);
    ent_query_fns!(IndexedInterWikiLink; typed);
    ent_query_fns!(NamedInterWikiLink; typed);
    ent_query_fns!(DiaryLink; typed);
    ent_query_fns!(RawLink; typed);
    ent_query_fns!(ExternalFileLink; typed);
    ent_query_fns!(TransclusionLink; typed);
    ent_query_fns!(Tags; typed);
    ent_query_fns!(CodeInline; typed);
    ent_query_fns!(MathInline; typed);
    ent_query_fns!(Comment; untyped);
    ent_query_fns!(LineComment; typed);
    ent_query_fns!(MultiLineComment; typed);
}

pub struct Mutation;

#[async_graphql::Object]
impl Mutation {
    /// Imports/re-imports a wiki from the specified path
    async fn import_wiki(
        &self,
        path: String,
        index: u32,
        name: Option<String>,
    ) -> async_graphql::Result<Wiki> {
        Err(async_graphql::Error::new("TODO: Implement"))
    }

    /// Imports/re-imports a standalone wiki file from the specified path
    async fn import_file(
        &self,
        path: String,
    ) -> async_graphql::Result<ParsedFile> {
        trace!("import_file(path: {:?})", path);

        let c_path = tokio::fs::canonicalize(path)
            .await
            .map_err(|x| async_graphql::Error::new(x.to_string()))?;

        // First, search and remove any file with the given path as we will
        // be reloading it
        let results = gql_db()?
            .find_all_typed::<ParsedFile>(
                ParsedFile::query()
                    .where_path(P::equals(c_path.to_string_lossy().to_string()))
                    .into(),
            )
            .map_err(|x| async_graphql::Error::new(x.to_string()))?;
        for ent in results {
            ent.remove()?;
        }

        // Second, load the contents of the file into memory
        let text = tokio::fs::read_to_string(c_path.as_ref())
            .await
            .map_err(|x| async_graphql::Error::new(x.to_string()))?;
        let checksum = format!("{:x}", Sha1::digest(text.as_bytes()));

        // Third, convert file contents into a parsed file
        let page: v::Page = Language::from_vimwiki_str(&text).parse().map_err(
            |x: ParseError| async_graphql::Error::new(x.to_string()),
        )?;

        // Fourth, store parsed file into database
        // TODO: Convert logic above and here into a "load_from_path" function
        //       tied to the ParsedFile ent to make this reusable. It should
        //       search for an existing file at path, read the file's contents
        //       to get the checksum, compare current and new checksum, and if
        //       different parse the file and replace the current ent with the
        //       new one
        let page = Page::try_from(page)

        Ok(())
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
        Err(async_graphql::Error::new("TODO: Implement"))
    }
}

/// Represents the overall schema for the vimwiki GraphQL server
pub type Schema =
    async_graphql::Schema<Query, Mutation, async_graphql::EmptySubscription>;

pub fn new_schema() -> Schema {
    Schema::build(Query, Mutation, async_graphql::EmptySubscription).finish()
}
