use crate::{data::*, database::gql_db};
use entity::{TypedPredicate as P, *};
use log::trace;
use paste::paste;

/// Provides a reference to a typed version of the GraphQL database if available
macro_rules! gql_db_typed_ref {
    () => {
        gql_db()?
            .as_ref()
            .as_database::<InmemoryDatabase>()
            .ok_or_else(|| {
                async_graphql::Error::new("Invalid database type found")
            })
    };
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

    /// Queries for instances of any ent that matches the given filter, or
    /// return all instances if no filter provided
    async fn ents(
        &self,
        filter: Option<entity::ext::async_graphql::GqlEntFilter>,
    ) -> async_graphql::Result<Vec<Box<dyn Ent>>> {
        let query: entity::Query = match filter {
            Some(x) => x.into(),
            None => entity::Query::default().where_created(P::greater_than(0)),
        };

        gql_db()?
            .find_all(query)
            .map_err(|x| async_graphql::Error::new(x.to_string()))
    }

    /// Queries for instances of Wiki that match the filter, or return all
    /// instances if no filter provided
    async fn wikis(
        &self,
        filter: Option<GqlWikiFilter>,
    ) -> async_graphql::Result<Vec<Wiki>> {
        let query: entity::Query = match filter {
            Some(x) => x.into(),
            None => Wiki::query().into(),
        };

        gql_db()?
            .find_all_typed::<Wiki>(query)
            .map_err(|x| async_graphql::Error::new(x.to_string()))
    }

    /// Queries for a single instance of Wiki by its id
    async fn wiki(&self, id: Id) -> async_graphql::Result<Option<Wiki>> {
        gql_db()?
            .get_typed::<Wiki>(id)
            .map_err(|x| async_graphql::Error::new(x.to_string()))
    }

    /// Queries for instances of ParsedFile that match the filter, or return all
    /// instances if no filter provided
    async fn parsed_files(
        &self,
        filter: Option<GqlParsedFileFilter>,
    ) -> async_graphql::Result<Vec<ParsedFile>> {
        let query: entity::Query = match filter {
            Some(x) => x.into(),
            None => ParsedFile::query().into(),
        };

        gql_db()?
            .find_all_typed::<ParsedFile>(query)
            .map_err(|x| async_graphql::Error::new(x.to_string()))
    }

    /// Queries for a single instance of ParsedFile by its id
    async fn parsed_file(
        &self,
        id: Id,
    ) -> async_graphql::Result<Option<ParsedFile>> {
        gql_db()?
            .get_typed::<ParsedFile>(id)
            .map_err(|x| async_graphql::Error::new(x.to_string()))
    }

    /// Queries for instances of Page that match the filter, or return all
    /// instances if no filter provided
    async fn pages(
        &self,
        filter: Option<GqlPageFilter>,
    ) -> async_graphql::Result<Vec<Page>> {
        let query: entity::Query = match filter {
            Some(x) => x.into(),
            None => Page::query().into(),
        };

        gql_db()?
            .find_all_typed::<Page>(query)
            .map_err(|x| async_graphql::Error::new(x.to_string()))
    }

    /// Queries for a single instance of Page by its id
    async fn page(&self, id: Id) -> async_graphql::Result<Option<Page>> {
        gql_db()?
            .get_typed::<Page>(id)
            .map_err(|x| async_graphql::Error::new(x.to_string()))
    }

    /// Queries for instances of Element that matches the given filter, or
    /// return all instances if no filter provided
    async fn elements(
        &self,
        filter: Option<entity::ext::async_graphql::GqlEntFilter>,
    ) -> async_graphql::Result<Vec<Element>> {
        let query: entity::Query = match filter {
            Some(x) => x.into(),
            None => entity::Query::default().where_created(P::greater_than(0)),
        };

        ElementQuery::from(query)
            .execute(gql_db_typed_ref!()?)
            .map_err(|x| async_graphql::Error::new(x.to_string()))
    }

    /// Queries for a single instance of Element by its id
    async fn element(&self, id: Id) -> async_graphql::Result<Option<Element>> {
        Element::query()
            .where_id(P::equals(id))
            .execute(gql_db_typed_ref!()?)
            .map(|x| x.into_iter().next())
            .map_err(|x| async_graphql::Error::new(x.to_string()))
    }

    /// Queries for instances of BlockElement that matches the given filter, or
    /// return all instances if no filter provided
    async fn block_elements(
        &self,
        filter: Option<entity::ext::async_graphql::GqlEntFilter>,
    ) -> async_graphql::Result<Vec<BlockElement>> {
        let query: entity::Query = match filter {
            Some(x) => x.into(),
            None => entity::Query::default().where_created(P::greater_than(0)),
        };

        BlockElementQuery::from(query)
            .execute(gql_db_typed_ref!()?)
            .map_err(|x| async_graphql::Error::new(x.to_string()))
    }

    /// Queries for a single instance of BlockElement by its id
    async fn block_element(
        &self,
        id: Id,
    ) -> async_graphql::Result<Option<BlockElement>> {
        BlockElement::query()
            .where_id(P::equals(id))
            .execute(gql_db_typed_ref!()?)
            .map(|x| x.into_iter().next())
            .map_err(|x| async_graphql::Error::new(x.to_string()))
    }

    /// Queries for instances of InlineBlockElement that matches the given filter, or
    /// return all instances if no filter provided
    async fn inline_block_elements(
        &self,
        filter: Option<entity::ext::async_graphql::GqlEntFilter>,
    ) -> async_graphql::Result<Vec<InlineBlockElement>> {
        let query: entity::Query = match filter {
            Some(x) => x.into(),
            None => entity::Query::default().where_created(P::greater_than(0)),
        };

        InlineBlockElementQuery::from(query)
            .execute(gql_db_typed_ref!()?)
            .map_err(|x| async_graphql::Error::new(x.to_string()))
    }

    /// Queries for a single instance of InlineBlockElement by its id
    async fn inline_block_element(
        &self,
        id: Id,
    ) -> async_graphql::Result<Option<InlineBlockElement>> {
        InlineBlockElement::query()
            .where_id(P::equals(id))
            .execute(gql_db_typed_ref!()?)
            .map(|x| x.into_iter().next())
            .map_err(|x| async_graphql::Error::new(x.to_string()))
    }

    /// Queries for instances of InlineElement that matches the given filter, or
    /// return all instances if no filter provided
    async fn inline_elements(
        &self,
        filter: Option<entity::ext::async_graphql::GqlEntFilter>,
    ) -> async_graphql::Result<Vec<InlineElement>> {
        let query: entity::Query = match filter {
            Some(x) => x.into(),
            None => entity::Query::default().where_created(P::greater_than(0)),
        };

        InlineElementQuery::from(query)
            .execute(gql_db_typed_ref!()?)
            .map_err(|x| async_graphql::Error::new(x.to_string()))
    }

    /// Queries for a single instance of InlineElement by its id
    async fn inline_element(
        &self,
        id: Id,
    ) -> async_graphql::Result<Option<InlineElement>> {
        InlineElement::query()
            .where_id(P::equals(id))
            .execute(gql_db_typed_ref!()?)
            .map(|x| x.into_iter().next())
            .map_err(|x| async_graphql::Error::new(x.to_string()))
    }

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
