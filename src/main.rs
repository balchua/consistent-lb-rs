use actix_web::{web, App, HttpServer};

use chrono::Local;
use env_logger::Builder;
use log::LevelFilter;
use routes::consistent_handler::pick;
use service::consistent::Consistent;
use std::io::Write;

mod routes;
mod service;

struct ConsistentHash {
    consistent: Consistent, // <- Mutex is necessary to mutate safely across threads
}
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    Builder::new()
        .format(|buf, record| {
            writeln!(
                buf,
                "{} [{}] - {}",
                Local::now().format("%Y-%m-%dT%H:%M:%S"),
                record.level(),
                record.args()
            )
        })
        .filter(None, LevelFilter::Info)
        .init();
    let consistent = Consistent::new(10);
    let c = web::Data::new(ConsistentHash {
        consistent: consistent,
    });
    HttpServer::new(move || {
        App::new()
            .app_data(c.clone()) // <- register the created data
            .service(pick)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
