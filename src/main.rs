extern crate actix_web;
extern crate deadpool_postgres;
extern crate rustls;
extern crate tokio_postgres;
extern crate tokio_postgres_rustls;

use actix_web::{App, HttpServer, middleware};
use actix_web::web::Data;

use pooly::{AppContext, resources};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let app_context = AppContext::new();

    let server = HttpServer::new(
        move || {
            App::new()
                .wrap(middleware::Logger::default())
                .app_data(Data::new(app_context.query_service.clone()))
                .app_data(Data::new(app_context.connection_config_service.clone()))
                .app_data(Data::new(app_context.secrets_service.clone()))
                .app_data(Data::new(app_context.shares_service.clone()))
                .service(resources::query::bulk)
                .service(resources::query::query)
                .service(resources::configs::create)
                .service(resources::configs::update)
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
