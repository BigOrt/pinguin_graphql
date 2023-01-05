use std::{convert::Infallible, sync::Arc, net::SocketAddr};

use hyper::{
    service::{make_service_fn, service_fn},
    Method, Response, StatusCode, Body,
    server::Server,
};
use juniper::{
    // tests::fixtures::starwars::schema::{Database, Query},
    EmptyMutation, EmptySubscription, RootNode,
};
use juniper_hyper::{ graphiql, graphql };
mod dbhelp;


// Try Impl DB ! 



#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    let addr = SocketAddr::from(([127,0,0,1], 9000));
    let db = Arc::new(dbhelp::Database::new());

    let root_node = Arc::new(RootNode::new(
        dbhelp::Query,
        EmptyMutation::<dbhelp::Database>::new(),
        EmptySubscription::<dbhelp::Database>::new(),
    ));

    let new_service = make_service_fn(move |_| {
        let root_node = root_node.clone();
        let ctx = db.clone();

        async {
            Ok::<_, hyper::Error>(service_fn(move |req| {
                let root_node = root_node.clone();
                let ctx = ctx.clone();
                async {
                    Ok::<_, Infallible>(match (req.method(), req.uri().path()) {
                        (&Method::GET, "/") => graphiql("/graphql", None).await,
                        (&Method::GET, "/graphql") | (&Method::POST, "/graphql") => {
                            graphql(root_node, ctx, req).await
                        }
                        _ => {
                            let mut response = Response::new(Body::empty());
                            *response.status_mut() = StatusCode::NOT_FOUND;
                            response
                        }
                    })
                }
            }))
        }
    });

    let server1 = Server::bind(&addr).serve(new_service);
    println!("Listening on http://{addr}");

    if let Err(e) = server1.await {
        eprintln!("server error: {e}");
    }
}
