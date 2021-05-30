use crate::{graphql, Opt};
use log::info;
use std::convert::Infallible;
use warp::{reply::Reply, Filter};

macro_rules! graphql_endpoint {
    ($path:expr, $program:expr) => {{
        let schema = graphql::new_schema();
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

pub async fn run(opt: Opt) {
    let graphql_filter = graphql_endpoint!("graphql", program);
    let graphiql_filter = graphiql_endpoint!("graphiql", "/graphql");

    let routes = warp::any().and(graphiql_filter.or(graphql_filter));

    info!("Listening on 0.0.0.0:{}", opt.port);
    warp::serve(routes).run(([0, 0, 0, 0], opt.port)).await;
}
