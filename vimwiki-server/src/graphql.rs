use crate::{data::*, database::gql_db};
use entity::*;
use log::trace;
use paste::paste;

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

#[inline]
fn default_exts() -> Vec<String> {
    vec!["wiki".to_string()]
}

#[async_graphql::Object]
impl Mutation {
    /// Imports/re-imports a wiki from the specified path
    async fn import_wiki(
        &self,
        path: String,
        index: u32,
        name: Option<String>,
        #[graphql(default_with = "default_exts()")] exts: Vec<String>,
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
            &exts,
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

/// Represents the overall schema for the vimwiki GraphQL server
pub type Schema =
    async_graphql::Schema<Query, Mutation, async_graphql::EmptySubscription>;

pub fn new_schema() -> Schema {
    Schema::build(Query, Mutation, async_graphql::EmptySubscription).finish()
}
