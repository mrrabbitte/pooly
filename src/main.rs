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

use crate::models::payloads::QueryRequest;
use crate::services::connections::config::ConnectionConfigService;
use crate::services::connections::ConnectionService;
use crate::services::queries::QueryService;

pub mod resources;
pub mod models;
pub mod services;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let connection_config_service =
        Arc::new(ConnectionConfigService::new());

    let query_service =
        Arc::new(QueryService::new(
            ConnectionService::new(
                connection_config_service.clone())));

    let server = HttpServer::new(
        move || {
            App::new()
                .wrap(middleware::Logger::default())
                .app_data(Data::new(query_service.clone()))
                .app_data(Data::new(connection_config_service.clone()))
                .service(resources::query)
                .service(resources::configs::create)
        })
        .bind("127.0.0.1:59090")?
        .run();

    println!("Server running at http://{}/", "127.0.0.1:59090");

    server.await
}
