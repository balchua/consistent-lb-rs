use std::sync::Arc;

use hash_ring::NodeInfo;
use hyper::Server;
use service::consistent::Consistent;

use crate::handlers::proxy::MakeSvc;

mod handlers;
mod service;

#[tokio::main]
async fn main() {
    let addr = ([127, 0, 0, 1], 3000).into();
    let nodes = setup_nodes();
    let c = Consistent::new(10, nodes.clone());
    let server = Server::bind(&addr)
        .http2_only(true)
        .serve(MakeSvc { c: Arc::new(c) });

    println!("Listening on http://{}", addr);
    // And now add a graceful shutdown signal...
    let graceful = server.with_graceful_shutdown(shutdown_signal());
    // Run this server for... forever!
    if let Err(e) = graceful.await {
        eprintln!("server error: {}", e);
    }
}

fn setup_nodes() -> Vec<NodeInfo> {
    let mut nodes: Vec<NodeInfo> = Vec::new();
    nodes.push(NodeInfo {
        host: "localhost",
        port: 9000,
    });
    nodes.push(NodeInfo {
        host: "localhost",
        port: 8000,
    });
    nodes
}

async fn shutdown_signal() {
    // Wait for the CTRL+C signal
    tokio::signal::ctrl_c()
        .await
        .expect("failed to install CTRL+C signal handler");
}
