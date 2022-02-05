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
use ring::rand::SystemRandom;

use crate::models::payloads::QueryRequest;
use crate::services::connections::config::ConnectionConfigService;
use crate::services::connections::ConnectionService;
use crate::services::queries::QueryService;
use crate::services::secrets::generate::VecGenerator;
use crate::services::secrets::SecretsService;
use crate::services::secrets::shares::MasterKeySharesService;

pub mod resources;
pub mod models;
pub mod services;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let vec_generator =
        Arc::new(
            VecGenerator::new(
                Arc::new(SystemRandom::new())));

    let shares_service = Arc::new(MasterKeySharesService::new());

    let secrets_service =
        Arc::new(
            SecretsService::new(shares_service.clone(), vec_generator));

    let connection_config_service =
        Arc::new(ConnectionConfigService::new(secrets_service.clone()));

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
                .app_data(Data::new(secrets_service.clone()))
                .app_data(Data::new(shares_service.clone()))
                .service(resources::query::bulk)
                .service(resources::query::query)
                .service(resources::configs::create)
                .service(resources::secrets::initialize)
                .service(resources::secrets::unseal)
                .service(resources::shares::add_share)
                .service(resources::shares::clear_shares)
        })
        .bind("127.0.0.1:8868")?
        .run();

    println!("Server running at http://{}/", "127.0.0.1:8868");

    server.await
}
