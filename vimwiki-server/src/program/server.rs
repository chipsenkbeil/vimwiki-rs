use super::{graphql, Config, Program};
use log::info;
use std::convert::Infallible;
use warp::{reply::Reply, Filter};

macro_rules! graphql_endpoint {
    ($path:expr, $program:expr) => {{
        let schema = graphql::build_schema_with_program($program);
        warp::path($path).and(
            async_graphql_warp::graphql(schema).and_then(
                |(schema, request): (
                    graphql::Schema,
                    async_graphql::Request,
                )| async move {
                    let resp = schema.execute(request).await;
                    Ok::<_, Infallible>(
                        warp::reply::json(&resp).into_response(),
                    )
                },
            ),
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

pub async fn run(program: Program, config: Config) {
    let endpoint = format!("http://{}:{}/graphql", config.host, config.port);
    let endpoint_2 = endpoint.clone();

    let graphql_filter = graphql_endpoint!("graphql", program);
    let graphiql_filter = graphiql_endpoint!("graphiql", &endpoint);
    let graphql_playground_filter =
        graphql_playground_endpoint!("graphql_playground", &endpoint_2);

    let routes = warp::any().and(
        graphiql_filter
            .or(graphql_playground_filter)
            .or(graphql_filter),
    );

    info!("Listening on 0.0.0.0:{}", config.port);
    warp::serve(routes).run(([0, 0, 0, 0], config.port)).await;
}
