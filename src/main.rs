extern crate actix_web;
extern crate deadpool_postgres;
extern crate rustls;
extern crate tokio_postgres;
extern crate tokio_postgres_rustls;

use std::collections::HashMap;
use std::io::ErrorKind;
use std::sync::Arc;

use actix_web::{App, HttpServer, middleware, web};
use actix_web::web::Data;

use crate::connections::ConnectionService;
use crate::queries::QueryService;
use crate::models::payloads::QueryRequest;

pub mod connections;
pub mod queries;
pub mod resources;
pub mod models;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let query_service =
        Arc::new(QueryService::new(ConnectionService::new()));

    let server = HttpServer::new(
        move || {
            App::new()
                .wrap(middleware::Logger::default())
                .app_data(Data::new(query_service.clone()))
                .service(resources::query)
        })
        .bind("127.0.0.1:59090")?
        .run();

    println!("Server running at http://{}/", "127.0.0.1:59090");

    server.await
}
