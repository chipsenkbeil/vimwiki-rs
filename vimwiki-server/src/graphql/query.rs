use crate::{data::*, database::gql_db};
use entity::{TypedPredicate as P, *};

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

    /// Queries for instances of Blockquote that match the filter, or return all
    /// instances if no filter provided
    async fn blockquotes(
        &self,
        filter: Option<GqlBlockquoteFilter>,
    ) -> async_graphql::Result<Vec<Blockquote>> {
        let query: entity::Query = match filter {
            Some(x) => x.into(),
            None => Blockquote::query().into(),
        };

        gql_db()?
            .find_all_typed::<Blockquote>(query)
            .map_err(|x| async_graphql::Error::new(x.to_string()))
    }

    /// Queries for a single instance of Blockquote by its id
    async fn blockquote(
        &self,
        id: Id,
    ) -> async_graphql::Result<Option<Blockquote>> {
        gql_db()?
            .get_typed::<Blockquote>(id)
            .map_err(|x| async_graphql::Error::new(x.to_string()))
    }

    /// Queries for instances of DefinitionList that match the filter, or return all
    /// instances if no filter provided
    async fn definition_lists(
        &self,
        filter: Option<GqlDefinitionListFilter>,
    ) -> async_graphql::Result<Vec<DefinitionList>> {
        let query: entity::Query = match filter {
            Some(x) => x.into(),
            None => DefinitionList::query().into(),
        };

        gql_db()?
            .find_all_typed::<DefinitionList>(query)
            .map_err(|x| async_graphql::Error::new(x.to_string()))
    }

    /// Queries for a single instance of DefinitionList by its id
    async fn definition_list(
        &self,
        id: Id,
    ) -> async_graphql::Result<Option<DefinitionList>> {
        gql_db()?
            .get_typed::<DefinitionList>(id)
            .map_err(|x| async_graphql::Error::new(x.to_string()))
    }

    /// Queries for instances of Term that match the filter, or return all
    /// instances if no filter provided
    async fn terms(
        &self,
        filter: Option<GqlTermFilter>,
    ) -> async_graphql::Result<Vec<Term>> {
        let query: entity::Query = match filter {
            Some(x) => x.into(),
            None => Term::query().into(),
        };

        gql_db()?
            .find_all_typed::<Term>(query)
            .map_err(|x| async_graphql::Error::new(x.to_string()))
    }

    /// Queries for a single instance of Term by its id
    async fn term(&self, id: Id) -> async_graphql::Result<Option<Term>> {
        gql_db()?
            .get_typed::<Term>(id)
            .map_err(|x| async_graphql::Error::new(x.to_string()))
    }

    /// Queries for instances of Definition that match the filter, or return all
    /// instances if no filter provided
    async fn definitions(
        &self,
        filter: Option<GqlDefinitionFilter>,
    ) -> async_graphql::Result<Vec<Definition>> {
        let query: entity::Query = match filter {
            Some(x) => x.into(),
            None => Definition::query().into(),
        };

        gql_db()?
            .find_all_typed::<Definition>(query)
            .map_err(|x| async_graphql::Error::new(x.to_string()))
    }

    /// Queries for a single instance of Definition by its id
    async fn definition(
        &self,
        id: Id,
    ) -> async_graphql::Result<Option<Definition>> {
        gql_db()?
            .get_typed::<Definition>(id)
            .map_err(|x| async_graphql::Error::new(x.to_string()))
    }

    /// Queries for instances of Divider that match the filter, or return all
    /// instances if no filter provided
    async fn dividers(
        &self,
        filter: Option<GqlDividerFilter>,
    ) -> async_graphql::Result<Vec<Divider>> {
        let query: entity::Query = match filter {
            Some(x) => x.into(),
            None => Divider::query().into(),
        };

        gql_db()?
            .find_all_typed::<Divider>(query)
            .map_err(|x| async_graphql::Error::new(x.to_string()))
    }

    /// Queries for a single instance of Divider by its id
    async fn divider(&self, id: Id) -> async_graphql::Result<Option<Divider>> {
        gql_db()?
            .get_typed::<Divider>(id)
            .map_err(|x| async_graphql::Error::new(x.to_string()))
    }

    /// Queries for instances of Header that match the filter, or return all
    /// instances if no filter provided
    async fn headers(
        &self,
        filter: Option<GqlHeaderFilter>,
    ) -> async_graphql::Result<Vec<Header>> {
        let query: entity::Query = match filter {
            Some(x) => x.into(),
            None => Header::query().into(),
        };

        gql_db()?
            .find_all_typed::<Header>(query)
            .map_err(|x| async_graphql::Error::new(x.to_string()))
    }

    /// Queries for a single instance of Header by its id
    async fn header(&self, id: Id) -> async_graphql::Result<Option<Header>> {
        gql_db()?
            .get_typed::<Header>(id)
            .map_err(|x| async_graphql::Error::new(x.to_string()))
    }

    /// Queries for instances of List that match the filter, or return all
    /// instances if no filter provided
    async fn lists(
        &self,
        filter: Option<GqlListFilter>,
    ) -> async_graphql::Result<Vec<List>> {
        let query: entity::Query = match filter {
            Some(x) => x.into(),
            None => List::query().into(),
        };

        gql_db()?
            .find_all_typed::<List>(query)
            .map_err(|x| async_graphql::Error::new(x.to_string()))
    }

    /// Queries for a single instance of List by its id
    async fn list(&self, id: Id) -> async_graphql::Result<Option<List>> {
        gql_db()?
            .get_typed::<List>(id)
            .map_err(|x| async_graphql::Error::new(x.to_string()))
    }

    /// Queries for instances of ListItem that match the filter, or return all
    /// instances if no filter provided
    async fn list_items(
        &self,
        filter: Option<GqlListItemFilter>,
    ) -> async_graphql::Result<Vec<ListItem>> {
        let query: entity::Query = match filter {
            Some(x) => x.into(),
            None => ListItem::query().into(),
        };

        gql_db()?
            .find_all_typed::<ListItem>(query)
            .map_err(|x| async_graphql::Error::new(x.to_string()))
    }

    /// Queries for a single instance of ListItem by its id
    async fn list_item(
        &self,
        id: Id,
    ) -> async_graphql::Result<Option<ListItem>> {
        gql_db()?
            .get_typed::<ListItem>(id)
            .map_err(|x| async_graphql::Error::new(x.to_string()))
    }

    /// Queries for instances of ListItemAttributes that match the filter, or return all
    /// instances if no filter provided
    async fn list_items_attributes(
        &self,
        filter: Option<GqlListItemAttributesFilter>,
    ) -> async_graphql::Result<Vec<ListItemAttributes>> {
        let query: entity::Query = match filter {
            Some(x) => x.into(),
            None => ListItemAttributes::query().into(),
        };

        gql_db()?
            .find_all_typed::<ListItemAttributes>(query)
            .map_err(|x| async_graphql::Error::new(x.to_string()))
    }

    /// Queries for a single instance of ListItemAttributes by its id
    async fn list_item_attributes(
        &self,
        id: Id,
    ) -> async_graphql::Result<Option<ListItemAttributes>> {
        gql_db()?
            .get_typed::<ListItemAttributes>(id)
            .map_err(|x| async_graphql::Error::new(x.to_string()))
    }

    /// Queries for instances of ListItemContent that matches the given filter, or
    /// return all instances if no filter provided
    async fn list_item_contents(
        &self,
        filter: Option<entity::ext::async_graphql::GqlEntFilter>,
    ) -> async_graphql::Result<Vec<ListItemContent>> {
        let query: entity::Query = match filter {
            Some(x) => x.into(),
            None => entity::Query::default().where_created(P::greater_than(0)),
        };

        ListItemContentQuery::from(query)
            .execute(gql_db_typed_ref!()?)
            .map_err(|x| async_graphql::Error::new(x.to_string()))
    }

    /// Queries for a single instance of ListItemContent by its id
    async fn list_item_content(
        &self,
        id: Id,
    ) -> async_graphql::Result<Option<ListItemContent>> {
        ListItemContent::query()
            .where_id(P::equals(id))
            .execute(gql_db_typed_ref!()?)
            .map(|x| x.into_iter().next())
            .map_err(|x| async_graphql::Error::new(x.to_string()))
    }

    /// Queries for instances of MathBlock that match the filter, or return all
    /// instances if no filter provided
    async fn math_blocks(
        &self,
        filter: Option<GqlMathBlockFilter>,
    ) -> async_graphql::Result<Vec<MathBlock>> {
        let query: entity::Query = match filter {
            Some(x) => x.into(),
            None => MathBlock::query().into(),
        };

        gql_db()?
            .find_all_typed::<MathBlock>(query)
            .map_err(|x| async_graphql::Error::new(x.to_string()))
    }

    /// Queries for a single instance of MathBlock by its id
    async fn math_block(
        &self,
        id: Id,
    ) -> async_graphql::Result<Option<MathBlock>> {
        gql_db()?
            .get_typed::<MathBlock>(id)
            .map_err(|x| async_graphql::Error::new(x.to_string()))
    }

    /// Queries for instances of Paragraph that match the filter, or return all
    /// instances if no filter provided
    async fn paragraphs(
        &self,
        filter: Option<GqlParagraphFilter>,
    ) -> async_graphql::Result<Vec<Paragraph>> {
        let query: entity::Query = match filter {
            Some(x) => x.into(),
            None => Paragraph::query().into(),
        };

        gql_db()?
            .find_all_typed::<Paragraph>(query)
            .map_err(|x| async_graphql::Error::new(x.to_string()))
    }

    /// Queries for a single instance of Paragraph by its id
    async fn paragraph(
        &self,
        id: Id,
    ) -> async_graphql::Result<Option<Paragraph>> {
        gql_db()?
            .get_typed::<Paragraph>(id)
            .map_err(|x| async_graphql::Error::new(x.to_string()))
    }

    /// Queries for instances of Placeholder that matches the given filter, or
    /// return all instances if no filter provided
    async fn placeholders(
        &self,
        filter: Option<entity::ext::async_graphql::GqlEntFilter>,
    ) -> async_graphql::Result<Vec<Placeholder>> {
        let query: entity::Query = match filter {
            Some(x) => x.into(),
            None => entity::Query::default().where_created(P::greater_than(0)),
        };

        PlaceholderQuery::from(query)
            .execute(gql_db_typed_ref!()?)
            .map_err(|x| async_graphql::Error::new(x.to_string()))
    }

    /// Queries for a single instance of Placeholder by its id
    async fn placeholder(
        &self,
        id: Id,
    ) -> async_graphql::Result<Option<Placeholder>> {
        Placeholder::query()
            .where_id(P::equals(id))
            .execute(gql_db_typed_ref!()?)
            .map(|x| x.into_iter().next())
            .map_err(|x| async_graphql::Error::new(x.to_string()))
    }

    /// Queries for instances of PlaceholderTitle that match the filter, or return all
    /// instances if no filter provided
    async fn placeholder_titles(
        &self,
        filter: Option<GqlPlaceholderTitleFilter>,
    ) -> async_graphql::Result<Vec<PlaceholderTitle>> {
        let query: entity::Query = match filter {
            Some(x) => x.into(),
            None => PlaceholderTitle::query().into(),
        };

        gql_db()?
            .find_all_typed::<PlaceholderTitle>(query)
            .map_err(|x| async_graphql::Error::new(x.to_string()))
    }

    /// Queries for a single instance of PlaceholderTitle by its id
    async fn placeholder_title(
        &self,
        id: Id,
    ) -> async_graphql::Result<Option<PlaceholderTitle>> {
        gql_db()?
            .get_typed::<PlaceholderTitle>(id)
            .map_err(|x| async_graphql::Error::new(x.to_string()))
    }

    /// Queries for instances of PlaceholderNoHtml that match the filter, or return all
    /// instances if no filter provided
    async fn placeholder_no_htmls(
        &self,
        filter: Option<GqlPlaceholderNoHtmlFilter>,
    ) -> async_graphql::Result<Vec<PlaceholderNoHtml>> {
        let query: entity::Query = match filter {
            Some(x) => x.into(),
            None => PlaceholderNoHtml::query().into(),
        };

        gql_db()?
            .find_all_typed::<PlaceholderNoHtml>(query)
            .map_err(|x| async_graphql::Error::new(x.to_string()))
    }

    /// Queries for a single instance of PlaceholderNoHtml by its id
    async fn placeholder_no_html(
        &self,
        id: Id,
    ) -> async_graphql::Result<Option<PlaceholderNoHtml>> {
        gql_db()?
            .get_typed::<PlaceholderNoHtml>(id)
            .map_err(|x| async_graphql::Error::new(x.to_string()))
    }

    /// Queries for instances of PlaceholderTemplate that match the filter, or return all
    /// instances if no filter provided
    async fn placeholder_templates(
        &self,
        filter: Option<GqlPlaceholderTemplateFilter>,
    ) -> async_graphql::Result<Vec<PlaceholderTemplate>> {
        let query: entity::Query = match filter {
            Some(x) => x.into(),
            None => PlaceholderTemplate::query().into(),
        };

        gql_db()?
            .find_all_typed::<PlaceholderTemplate>(query)
            .map_err(|x| async_graphql::Error::new(x.to_string()))
    }

    /// Queries for a single instance of PlaceholderTemplate by its id
    async fn placeholder_template(
        &self,
        id: Id,
    ) -> async_graphql::Result<Option<PlaceholderTemplate>> {
        gql_db()?
            .get_typed::<PlaceholderTemplate>(id)
            .map_err(|x| async_graphql::Error::new(x.to_string()))
    }

    /// Queries for instances of PlaceholderDate that match the filter, or return all
    /// instances if no filter provided
    async fn placeholder_dates(
        &self,
        filter: Option<GqlPlaceholderDateFilter>,
    ) -> async_graphql::Result<Vec<PlaceholderDate>> {
        let query: entity::Query = match filter {
            Some(x) => x.into(),
            None => PlaceholderDate::query().into(),
        };

        gql_db()?
            .find_all_typed::<PlaceholderDate>(query)
            .map_err(|x| async_graphql::Error::new(x.to_string()))
    }

    /// Queries for a single instance of PlaceholderDate by its id
    async fn placeholder_date(
        &self,
        id: Id,
    ) -> async_graphql::Result<Option<PlaceholderDate>> {
        gql_db()?
            .get_typed::<PlaceholderDate>(id)
            .map_err(|x| async_graphql::Error::new(x.to_string()))
    }

    /// Queries for instances of PlaceholderOther that match the filter, or return all
    /// instances if no filter provided
    async fn placeholder_others(
        &self,
        filter: Option<GqlPlaceholderOtherFilter>,
    ) -> async_graphql::Result<Vec<PlaceholderOther>> {
        let query: entity::Query = match filter {
            Some(x) => x.into(),
            None => PlaceholderOther::query().into(),
        };

        gql_db()?
            .find_all_typed::<PlaceholderOther>(query)
            .map_err(|x| async_graphql::Error::new(x.to_string()))
    }

    /// Queries for a single instance of PlaceholderOther by its id
    async fn placeholder_other(
        &self,
        id: Id,
    ) -> async_graphql::Result<Option<PlaceholderOther>> {
        gql_db()?
            .get_typed::<PlaceholderOther>(id)
            .map_err(|x| async_graphql::Error::new(x.to_string()))
    }

    /// Queries for instances of PreformattedText that match the filter, or return all
    /// instances if no filter provided
    async fn preformatted_texts(
        &self,
        filter: Option<GqlPreformattedTextFilter>,
    ) -> async_graphql::Result<Vec<PreformattedText>> {
        let query: entity::Query = match filter {
            Some(x) => x.into(),
            None => PreformattedText::query().into(),
        };

        gql_db()?
            .find_all_typed::<PreformattedText>(query)
            .map_err(|x| async_graphql::Error::new(x.to_string()))
    }

    /// Queries for a single instance of PreformattedText by its id
    async fn preformatted_text(
        &self,
        id: Id,
    ) -> async_graphql::Result<Option<PreformattedText>> {
        gql_db()?
            .get_typed::<PreformattedText>(id)
            .map_err(|x| async_graphql::Error::new(x.to_string()))
    }

    /// Queries for instances of Table that match the filter, or return all
    /// instances if no filter provided
    async fn tables(
        &self,
        filter: Option<GqlTableFilter>,
    ) -> async_graphql::Result<Vec<Table>> {
        let query: entity::Query = match filter {
            Some(x) => x.into(),
            None => Table::query().into(),
        };

        gql_db()?
            .find_all_typed::<Table>(query)
            .map_err(|x| async_graphql::Error::new(x.to_string()))
    }

    /// Queries for a single instance of Table by its id
    async fn table(&self, id: Id) -> async_graphql::Result<Option<Table>> {
        gql_db()?
            .get_typed::<Table>(id)
            .map_err(|x| async_graphql::Error::new(x.to_string()))
    }

    /// Queries for instances of Row that matches the given filter, or
    /// return all instances if no filter provided
    async fn rows(
        &self,
        filter: Option<entity::ext::async_graphql::GqlEntFilter>,
    ) -> async_graphql::Result<Vec<Row>> {
        let query: entity::Query = match filter {
            Some(x) => x.into(),
            None => entity::Query::default().where_created(P::greater_than(0)),
        };

        RowQuery::from(query)
            .execute(gql_db_typed_ref!()?)
            .map_err(|x| async_graphql::Error::new(x.to_string()))
    }

    /// Queries for a single instance of Row by its id
    async fn row(&self, id: Id) -> async_graphql::Result<Option<Row>> {
        Row::query()
            .where_id(P::equals(id))
            .execute(gql_db_typed_ref!()?)
            .map(|x| x.into_iter().next())
            .map_err(|x| async_graphql::Error::new(x.to_string()))
    }

    /// Queries for instances of ContentRow that match the filter, or return all
    /// instances if no filter provided
    async fn content_rows(
        &self,
        filter: Option<GqlContentRowFilter>,
    ) -> async_graphql::Result<Vec<ContentRow>> {
        let query: entity::Query = match filter {
            Some(x) => x.into(),
            None => ContentRow::query().into(),
        };

        gql_db()?
            .find_all_typed::<ContentRow>(query)
            .map_err(|x| async_graphql::Error::new(x.to_string()))
    }

    /// Queries for a single instance of ContentRow by its id
    async fn content_row(
        &self,
        id: Id,
    ) -> async_graphql::Result<Option<ContentRow>> {
        gql_db()?
            .get_typed::<ContentRow>(id)
            .map_err(|x| async_graphql::Error::new(x.to_string()))
    }

    /// Queries for instances of DividerRow that match the filter, or return all
    /// instances if no filter provided
    async fn divider_rows(
        &self,
        filter: Option<GqlDividerRowFilter>,
    ) -> async_graphql::Result<Vec<DividerRow>> {
        let query: entity::Query = match filter {
            Some(x) => x.into(),
            None => DividerRow::query().into(),
        };

        gql_db()?
            .find_all_typed::<DividerRow>(query)
            .map_err(|x| async_graphql::Error::new(x.to_string()))
    }

    /// Queries for a single instance of DividerRow by its id
    async fn divider_row(
        &self,
        id: Id,
    ) -> async_graphql::Result<Option<DividerRow>> {
        gql_db()?
            .get_typed::<DividerRow>(id)
            .map_err(|x| async_graphql::Error::new(x.to_string()))
    }

    /// Queries for instances of Cell that matches the given filter, or
    /// return all instances if no filter provided
    async fn cells(
        &self,
        filter: Option<entity::ext::async_graphql::GqlEntFilter>,
    ) -> async_graphql::Result<Vec<Cell>> {
        let query: entity::Query = match filter {
            Some(x) => x.into(),
            None => entity::Query::default().where_created(P::greater_than(0)),
        };

        CellQuery::from(query)
            .execute(gql_db_typed_ref!()?)
            .map_err(|x| async_graphql::Error::new(x.to_string()))
    }

    /// Queries for a single instance of Cell by its id
    async fn cell(&self, id: Id) -> async_graphql::Result<Option<Cell>> {
        Cell::query()
            .where_id(P::equals(id))
            .execute(gql_db_typed_ref!()?)
            .map(|x| x.into_iter().next())
            .map_err(|x| async_graphql::Error::new(x.to_string()))
    }

    /// Queries for instances of ContentCell that match the filter, or return all
    /// instances if no filter provided
    async fn content_cells(
        &self,
        filter: Option<GqlContentCellFilter>,
    ) -> async_graphql::Result<Vec<ContentCell>> {
        let query: entity::Query = match filter {
            Some(x) => x.into(),
            None => ContentCell::query().into(),
        };

        gql_db()?
            .find_all_typed::<ContentCell>(query)
            .map_err(|x| async_graphql::Error::new(x.to_string()))
    }

    /// Queries for a single instance of ContentCell by its id
    async fn content_cell(
        &self,
        id: Id,
    ) -> async_graphql::Result<Option<ContentCell>> {
        gql_db()?
            .get_typed::<ContentCell>(id)
            .map_err(|x| async_graphql::Error::new(x.to_string()))
    }

    /// Queries for instances of SpanLeftCell that match the filter, or return all
    /// instances if no filter provided
    async fn span_left_cells(
        &self,
        filter: Option<GqlSpanLeftCellFilter>,
    ) -> async_graphql::Result<Vec<SpanLeftCell>> {
        let query: entity::Query = match filter {
            Some(x) => x.into(),
            None => SpanLeftCell::query().into(),
        };

        gql_db()?
            .find_all_typed::<SpanLeftCell>(query)
            .map_err(|x| async_graphql::Error::new(x.to_string()))
    }

    /// Queries for a single instance of SpanLeftCell by its id
    async fn span_left_cell(
        &self,
        id: Id,
    ) -> async_graphql::Result<Option<SpanLeftCell>> {
        gql_db()?
            .get_typed::<SpanLeftCell>(id)
            .map_err(|x| async_graphql::Error::new(x.to_string()))
    }

    /// Queries for instances of SpanAboveCell that match the filter, or return all
    /// instances if no filter provided
    async fn span_above_cells(
        &self,
        filter: Option<GqlSpanAboveCellFilter>,
    ) -> async_graphql::Result<Vec<SpanAboveCell>> {
        let query: entity::Query = match filter {
            Some(x) => x.into(),
            None => SpanAboveCell::query().into(),
        };

        gql_db()?
            .find_all_typed::<SpanAboveCell>(query)
            .map_err(|x| async_graphql::Error::new(x.to_string()))
    }

    /// Queries for a single instance of SpanAboveCell by its id
    async fn span_above_cell(
        &self,
        id: Id,
    ) -> async_graphql::Result<Option<SpanAboveCell>> {
        gql_db()?
            .get_typed::<SpanAboveCell>(id)
            .map_err(|x| async_graphql::Error::new(x.to_string()))
    }

    /// Queries for instances of Text that match the filter, or return all
    /// instances if no filter provided
    async fn texts(
        &self,
        filter: Option<GqlTextFilter>,
    ) -> async_graphql::Result<Vec<Text>> {
        let query: entity::Query = match filter {
            Some(x) => x.into(),
            None => Text::query().into(),
        };

        gql_db()?
            .find_all_typed::<Text>(query)
            .map_err(|x| async_graphql::Error::new(x.to_string()))
    }

    /// Queries for a single instance of Text by its id
    async fn text(&self, id: Id) -> async_graphql::Result<Option<Text>> {
        gql_db()?
            .get_typed::<Text>(id)
            .map_err(|x| async_graphql::Error::new(x.to_string()))
    }

    /// Queries for instances of DecoratedText that match the filter, or return all
    /// instances if no filter provided
    async fn decorated_texts(
        &self,
        filter: Option<GqlDecoratedTextFilter>,
    ) -> async_graphql::Result<Vec<DecoratedText>> {
        let query: entity::Query = match filter {
            Some(x) => x.into(),
            None => DecoratedText::query().into(),
        };

        gql_db()?
            .find_all_typed::<DecoratedText>(query)
            .map_err(|x| async_graphql::Error::new(x.to_string()))
    }

    /// Queries for a single instance of DecoratedText by its id
    async fn decorated_text(
        &self,
        id: Id,
    ) -> async_graphql::Result<Option<DecoratedText>> {
        gql_db()?
            .get_typed::<DecoratedText>(id)
            .map_err(|x| async_graphql::Error::new(x.to_string()))
    }

    /// Queries for instances of DecoratedTextContent that matches the given filter, or
    /// return all instances if no filter provided
    async fn decorated_text_contents(
        &self,
        filter: Option<entity::ext::async_graphql::GqlEntFilter>,
    ) -> async_graphql::Result<Vec<DecoratedTextContent>> {
        let query: entity::Query = match filter {
            Some(x) => x.into(),
            None => entity::Query::default().where_created(P::greater_than(0)),
        };

        DecoratedTextContentQuery::from(query)
            .execute(gql_db_typed_ref!()?)
            .map_err(|x| async_graphql::Error::new(x.to_string()))
    }

    /// Queries for a single instance of DecoratedTextContent by its id
    async fn decorated_text_content(
        &self,
        id: Id,
    ) -> async_graphql::Result<Option<DecoratedTextContent>> {
        DecoratedTextContent::query()
            .where_id(P::equals(id))
            .execute(gql_db_typed_ref!()?)
            .map(|x| x.into_iter().next())
            .map_err(|x| async_graphql::Error::new(x.to_string()))
    }

    /// Queries for instances of Keyword that match the filter, or return all
    /// instances if no filter provided
    async fn keywords(
        &self,
        filter: Option<GqlKeywordFilter>,
    ) -> async_graphql::Result<Vec<Keyword>> {
        let query: entity::Query = match filter {
            Some(x) => x.into(),
            None => Keyword::query().into(),
        };

        gql_db()?
            .find_all_typed::<Keyword>(query)
            .map_err(|x| async_graphql::Error::new(x.to_string()))
    }

    /// Queries for a single instance of Keyword by its id
    async fn keyword(&self, id: Id) -> async_graphql::Result<Option<Keyword>> {
        gql_db()?
            .get_typed::<Keyword>(id)
            .map_err(|x| async_graphql::Error::new(x.to_string()))
    }

    /// Queries for instances of Link that matches the given filter, or
    /// return all instances if no filter provided
    async fn links(
        &self,
        filter: Option<entity::ext::async_graphql::GqlEntFilter>,
    ) -> async_graphql::Result<Vec<Link>> {
        let query: entity::Query = match filter {
            Some(x) => x.into(),
            None => entity::Query::default().where_created(P::greater_than(0)),
        };

        LinkQuery::from(query)
            .execute(gql_db_typed_ref!()?)
            .map_err(|x| async_graphql::Error::new(x.to_string()))
    }

    /// Queries for a single instance of Link by its id
    async fn link(&self, id: Id) -> async_graphql::Result<Option<Link>> {
        Link::query()
            .where_id(P::equals(id))
            .execute(gql_db_typed_ref!()?)
            .map(|x| x.into_iter().next())
            .map_err(|x| async_graphql::Error::new(x.to_string()))
    }

    /// Queries for instances of WikiLink that match the filter, or return all
    /// instances if no filter provided
    async fn wiki_links(
        &self,
        filter: Option<GqlWikiLinkFilter>,
    ) -> async_graphql::Result<Vec<WikiLink>> {
        let query: entity::Query = match filter {
            Some(x) => x.into(),
            None => WikiLink::query().into(),
        };

        gql_db()?
            .find_all_typed::<WikiLink>(query)
            .map_err(|x| async_graphql::Error::new(x.to_string()))
    }

    /// Queries for a single instance of WikiLink by its id
    async fn wiki_link(
        &self,
        id: Id,
    ) -> async_graphql::Result<Option<WikiLink>> {
        gql_db()?
            .get_typed::<WikiLink>(id)
            .map_err(|x| async_graphql::Error::new(x.to_string()))
    }

    /// Queries for instances of IndexedInterWikiLink that match the filter, or return all
    /// instances if no filter provided
    async fn indexed_inter_wiki_links(
        &self,
        filter: Option<GqlIndexedInterWikiLinkFilter>,
    ) -> async_graphql::Result<Vec<IndexedInterWikiLink>> {
        let query: entity::Query = match filter {
            Some(x) => x.into(),
            None => IndexedInterWikiLink::query().into(),
        };

        gql_db()?
            .find_all_typed::<IndexedInterWikiLink>(query)
            .map_err(|x| async_graphql::Error::new(x.to_string()))
    }

    /// Queries for a single instance of IndexedInterWikiLink by its id
    async fn indexed_inter_wiki_link(
        &self,
        id: Id,
    ) -> async_graphql::Result<Option<IndexedInterWikiLink>> {
        gql_db()?
            .get_typed::<IndexedInterWikiLink>(id)
            .map_err(|x| async_graphql::Error::new(x.to_string()))
    }

    /// Queries for instances of NamedInterWikiLink that match the filter, or return all
    /// instances if no filter provided
    async fn named_inter_wiki_links(
        &self,
        filter: Option<GqlNamedInterWikiLinkFilter>,
    ) -> async_graphql::Result<Vec<NamedInterWikiLink>> {
        let query: entity::Query = match filter {
            Some(x) => x.into(),
            None => NamedInterWikiLink::query().into(),
        };

        gql_db()?
            .find_all_typed::<NamedInterWikiLink>(query)
            .map_err(|x| async_graphql::Error::new(x.to_string()))
    }

    /// Queries for a single instance of NamedInterWikiLink by its id
    async fn named_inter_wiki_link(
        &self,
        id: Id,
    ) -> async_graphql::Result<Option<NamedInterWikiLink>> {
        gql_db()?
            .get_typed::<NamedInterWikiLink>(id)
            .map_err(|x| async_graphql::Error::new(x.to_string()))
    }

    /// Queries for instances of DiaryLink that match the filter, or return all
    /// instances if no filter provided
    async fn diary_links(
        &self,
        filter: Option<GqlDiaryLinkFilter>,
    ) -> async_graphql::Result<Vec<DiaryLink>> {
        let query: entity::Query = match filter {
            Some(x) => x.into(),
            None => DiaryLink::query().into(),
        };

        gql_db()?
            .find_all_typed::<DiaryLink>(query)
            .map_err(|x| async_graphql::Error::new(x.to_string()))
    }

    /// Queries for a single instance of DiaryLink by its id
    async fn diary_link(
        &self,
        id: Id,
    ) -> async_graphql::Result<Option<DiaryLink>> {
        gql_db()?
            .get_typed::<DiaryLink>(id)
            .map_err(|x| async_graphql::Error::new(x.to_string()))
    }

    /// Queries for instances of RawLink that match the filter, or return all
    /// instances if no filter provided
    async fn raw_links(
        &self,
        filter: Option<GqlRawLinkFilter>,
    ) -> async_graphql::Result<Vec<RawLink>> {
        let query: entity::Query = match filter {
            Some(x) => x.into(),
            None => RawLink::query().into(),
        };

        gql_db()?
            .find_all_typed::<RawLink>(query)
            .map_err(|x| async_graphql::Error::new(x.to_string()))
    }

    /// Queries for a single instance of RawLink by its id
    async fn raw_link(&self, id: Id) -> async_graphql::Result<Option<RawLink>> {
        gql_db()?
            .get_typed::<RawLink>(id)
            .map_err(|x| async_graphql::Error::new(x.to_string()))
    }

    /// Queries for instances of ExternalFileLink that match the filter, or return all
    /// instances if no filter provided
    async fn external_file_links(
        &self,
        filter: Option<GqlExternalFileLinkFilter>,
    ) -> async_graphql::Result<Vec<ExternalFileLink>> {
        let query: entity::Query = match filter {
            Some(x) => x.into(),
            None => ExternalFileLink::query().into(),
        };

        gql_db()?
            .find_all_typed::<ExternalFileLink>(query)
            .map_err(|x| async_graphql::Error::new(x.to_string()))
    }

    /// Queries for a single instance of ExternalFileLink by its id
    async fn external_file_link(
        &self,
        id: Id,
    ) -> async_graphql::Result<Option<ExternalFileLink>> {
        gql_db()?
            .get_typed::<ExternalFileLink>(id)
            .map_err(|x| async_graphql::Error::new(x.to_string()))
    }

    /// Queries for instances of TransclusionLink that match the filter, or return all
    /// instances if no filter provided
    async fn transclusion_links(
        &self,
        filter: Option<GqlTransclusionLinkFilter>,
    ) -> async_graphql::Result<Vec<TransclusionLink>> {
        let query: entity::Query = match filter {
            Some(x) => x.into(),
            None => TransclusionLink::query().into(),
        };

        gql_db()?
            .find_all_typed::<TransclusionLink>(query)
            .map_err(|x| async_graphql::Error::new(x.to_string()))
    }

    /// Queries for a single instance of TransclusionLink by its id
    async fn transclusion_link(
        &self,
        id: Id,
    ) -> async_graphql::Result<Option<TransclusionLink>> {
        gql_db()?
            .get_typed::<TransclusionLink>(id)
            .map_err(|x| async_graphql::Error::new(x.to_string()))
    }

    /// Queries for instances of Tags that match the filter, or return all
    /// instances if no filter provided
    async fn tags(
        &self,
        filter: Option<GqlTagsFilter>,
    ) -> async_graphql::Result<Vec<Tags>> {
        let query: entity::Query = match filter {
            Some(x) => x.into(),
            None => Tags::query().into(),
        };

        gql_db()?
            .find_all_typed::<Tags>(query)
            .map_err(|x| async_graphql::Error::new(x.to_string()))
    }

    /// Queries for a single instance of Tags by its id
    async fn tag(&self, id: Id) -> async_graphql::Result<Option<Tags>> {
        gql_db()?
            .get_typed::<Tags>(id)
            .map_err(|x| async_graphql::Error::new(x.to_string()))
    }

    /// Queries for instances of CodeInline that match the filter, or return all
    /// instances if no filter provided
    async fn code_inlines(
        &self,
        filter: Option<GqlCodeInlineFilter>,
    ) -> async_graphql::Result<Vec<CodeInline>> {
        let query: entity::Query = match filter {
            Some(x) => x.into(),
            None => CodeInline::query().into(),
        };

        gql_db()?
            .find_all_typed::<CodeInline>(query)
            .map_err(|x| async_graphql::Error::new(x.to_string()))
    }

    /// Queries for a single instance of CodeInline by its id
    async fn code_inline(
        &self,
        id: Id,
    ) -> async_graphql::Result<Option<CodeInline>> {
        gql_db()?
            .get_typed::<CodeInline>(id)
            .map_err(|x| async_graphql::Error::new(x.to_string()))
    }

    /// Queries for instances of MathInline that match the filter, or return all
    /// instances if no filter provided
    async fn math_inlines(
        &self,
        filter: Option<GqlMathInlineFilter>,
    ) -> async_graphql::Result<Vec<MathInline>> {
        let query: entity::Query = match filter {
            Some(x) => x.into(),
            None => MathInline::query().into(),
        };

        gql_db()?
            .find_all_typed::<MathInline>(query)
            .map_err(|x| async_graphql::Error::new(x.to_string()))
    }

    /// Queries for a single instance of MathInline by its id
    async fn math_inline(
        &self,
        id: Id,
    ) -> async_graphql::Result<Option<MathInline>> {
        gql_db()?
            .get_typed::<MathInline>(id)
            .map_err(|x| async_graphql::Error::new(x.to_string()))
    }

    /// Queries for instances of Comment that matches the given filter, or
    /// return all instances if no filter provided
    async fn comments(
        &self,
        filter: Option<entity::ext::async_graphql::GqlEntFilter>,
    ) -> async_graphql::Result<Vec<Comment>> {
        let query: entity::Query = match filter {
            Some(x) => x.into(),
            None => entity::Query::default().where_created(P::greater_than(0)),
        };

        CommentQuery::from(query)
            .execute(gql_db_typed_ref!()?)
            .map_err(|x| async_graphql::Error::new(x.to_string()))
    }

    /// Queries for a single instance of Comment by its id
    async fn comment(&self, id: Id) -> async_graphql::Result<Option<Comment>> {
        Comment::query()
            .where_id(P::equals(id))
            .execute(gql_db_typed_ref!()?)
            .map(|x| x.into_iter().next())
            .map_err(|x| async_graphql::Error::new(x.to_string()))
    }

    /// Queries for instances of LineComment that match the filter, or return all
    /// instances if no filter provided
    async fn line_comments(
        &self,
        filter: Option<GqlLineCommentFilter>,
    ) -> async_graphql::Result<Vec<LineComment>> {
        let query: entity::Query = match filter {
            Some(x) => x.into(),
            None => LineComment::query().into(),
        };

        gql_db()?
            .find_all_typed::<LineComment>(query)
            .map_err(|x| async_graphql::Error::new(x.to_string()))
    }

    /// Queries for a single instance of LineComment by its id
    async fn line_comment(
        &self,
        id: Id,
    ) -> async_graphql::Result<Option<LineComment>> {
        gql_db()?
            .get_typed::<LineComment>(id)
            .map_err(|x| async_graphql::Error::new(x.to_string()))
    }

    /// Queries for instances of MultiLineComment that match the filter, or return all
    /// instances if no filter provided
    async fn multi_line_comments(
        &self,
        filter: Option<GqlMultiLineCommentFilter>,
    ) -> async_graphql::Result<Vec<MultiLineComment>> {
        let query: entity::Query = match filter {
            Some(x) => x.into(),
            None => MultiLineComment::query().into(),
        };

        gql_db()?
            .find_all_typed::<MultiLineComment>(query)
            .map_err(|x| async_graphql::Error::new(x.to_string()))
    }

    /// Queries for a single instance of MultiLineComment by its id
    async fn multi_line_comment(
        &self,
        id: Id,
    ) -> async_graphql::Result<Option<MultiLineComment>> {
        gql_db()?
            .get_typed::<MultiLineComment>(id)
            .map_err(|x| async_graphql::Error::new(x.to_string()))
    }
}
