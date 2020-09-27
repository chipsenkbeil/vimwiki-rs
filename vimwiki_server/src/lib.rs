use async_graphql::{EmptyMutation, EmptySubscription, Schema};
use std::convert::Infallible;
use vimwiki_macros::*;
use warp::{reply::Reply, Filter};

mod graphql;

struct Query;

#[async_graphql::Object]
impl Query {
    #[field(desc = "Returns a page")]
    async fn page(&self) -> graphql::data::Page {
        graphql::data::Page::from(vimwiki_page! {r#"
            = Some Header =
            =Another Header=
            =Third Header=
        "#})
    }
}

type MySchema = Schema<Query, EmptyMutation, EmptySubscription>;

macro_rules! graphql_endpoint {
    () => {{
        let schema = Schema::new(Query, EmptyMutation, EmptySubscription);
        async_graphql_warp::graphql(schema).and_then(
            |(schema, request): (MySchema, async_graphql::Request)| async move {
                let resp = schema.execute(request).await;
                Ok::<_, Infallible>(warp::reply::json(&resp).into_response())
            },
        )
    }};
}

macro_rules! graphiql_endpoint {
    () => {{
        warp::path("graphiql").map(|| {
            warp::reply::html(async_graphql::http::graphiql_source(
                "http://localhost:8000",
                None,
            ))
        })
    }};
}

macro_rules! graphql_playground_endpoint {
    () => {{
        warp::path("graphql_playground").map(|| {
            warp::reply::html(async_graphql::http::playground_source(
                async_graphql::http::GraphQLPlaygroundConfig::new(
                    "http://localhost:8000",
                ),
            ))
        })
    }};
}

pub async fn run_server() {
    let graphql_filter = graphql_endpoint!();
    let graphiql_filter = graphiql_endpoint!();
    let graphql_playground_filter = graphql_playground_endpoint!();

    let routes = warp::any().and(
        graphql_filter
            .or(graphiql_filter)
            .or(graphql_playground_filter),
    );
    warp::serve(routes).run(([0, 0, 0, 0], 8000)).await;
}
