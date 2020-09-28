use async_graphql::{EmptyMutation, EmptySubscription, Schema};
use std::convert::Infallible;
use warp::{reply::Reply, Filter};

mod graphql;

macro_rules! graphql_endpoint {
    () => {{
        let schema = Schema::new(graphql::Query, EmptyMutation, EmptySubscription);
        async_graphql_warp::graphql(schema).and_then(
            |(schema, request): (graphql::Schema, async_graphql::Request)| async move {
                let resp = schema.execute(request).await;
                Ok::<_, Infallible>(warp::reply::json(&resp).into_response())
            },
        )
    }};
}

macro_rules! graphiql_endpoint {
    ($path:expr, $graphql_endpoint:expr) => {{
        warp::path($path).map(move || {
            warp::reply::html(async_graphql::http::graphiql_source(
                $graphql_endpoint,
                None,
            ))
        })
    }};
}

macro_rules! graphql_playground_endpoint {
    ($path:expr, $graphql_endpoint:expr) => {{
        warp::path($path).map(move || {
            warp::reply::html(async_graphql::http::playground_source(
                async_graphql::http::GraphQLPlaygroundConfig::new(
                    $graphql_endpoint,
                ),
            ))
        })
    }};
}

pub async fn run_server(graphql_endpoint: &'static str) {
    let graphql_filter = graphql_endpoint!();
    let graphiql_filter = graphiql_endpoint!("graphiql", graphql_endpoint);
    let graphql_playground_filter =
        graphql_playground_endpoint!("graphql_playground", graphql_endpoint);

    let routes = warp::any().and(
        graphql_filter
            .or(graphiql_filter)
            .or(graphql_playground_filter),
    );
    warp::serve(routes).run(([0, 0, 0, 0], 8000)).await;
}
