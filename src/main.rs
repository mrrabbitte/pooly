extern crate actix_web;
extern crate deadpool_postgres;
extern crate rustls;
extern crate tokio_postgres;
extern crate tokio_postgres_rustls;

use actix_web::{App, HttpServer, middleware, web};
use actix_web::web::Data;
use config::Config;

use pooly::{AppContext, resources, services};
use pooly::models::api_key::InitializeApiKey;
use pooly::models::config::AppConfig;
use pooly::services::auth::initialization::InitializationGuard;
use pooly::services::auth::middleware::AuthGuard;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let config = Config::builder()
        .add_source(config::File::with_name("config.toml"))
        .build()
        .unwrap();

    let config = config.try_deserialize::<AppConfig>().unwrap();

    let init_api_key: InitializeApiKey = (&config).into();

    let app_context = AppContext::new();

    let server = HttpServer::new(
        move || {
            App::new()
                .wrap(middleware::Logger::default())
                .app_data(Data::new(app_context.auth_service.clone()))
                .app_data(Data::new(app_context.initialization_service.clone()))
                .app_data(Data::new(app_context.query_service.clone()))
                .app_data(Data::new(app_context.connection_config_service.clone()))
                .app_data(Data::new(app_context.secrets_service.clone()))
                .app_data(Data::new(app_context.shares_service.clone()))
                .app_data(Data::new(app_context.literal_ids_service.clone()))
                .app_data(Data::new(app_context.pattern_ids_service.clone()))
                .service(
                    web::scope("/i")
                        .wrap(InitializationGuard::new(init_api_key.clone()))
                        .service(resources::secrets::actions::initialize)
                        .service(resources::secrets::shares::add_share)
                        .service(resources::secrets::shares::clear_shares)
                        .service(resources::secrets::actions::unseal)
                )
                .service(
                    web::scope("/c")
                        .wrap(AuthGuard::client())
                        .service(resources::query::bulk)
                        .service(resources::query::query)
                )
                .service(
                    web::scope("/a")
                        .wrap(AuthGuard::admin())

                        .service(resources::keys::create)
                        .service(resources::keys::get)
                        .service(resources::keys::update)
                        .service(resources::keys::delete)
                        .service(resources::connections::create)
                        .service(resources::connections::get)
                        .service(resources::connections::update)
                        .service(resources::connections::delete)
                        .service(resources::connections::access::literal_ids::create)
                        .service(resources::connections::access::literal_ids::get)
                        .service(resources::connections::access::literal_ids::update)
                        .service(resources::connections::access::literal_ids::delete)
                        .service(resources::connections::access::pattern_ids::create)
                        .service(resources::connections::access::pattern_ids::get)
                        .service(resources::connections::access::pattern_ids::update)
                        .service(resources::connections::access::pattern_ids::delete)
                )
        })
        .bind("127.0.0.1:8868")?
        .run();

    println!("Server running at http://{}/", "127.0.0.1:8868");

    server.await
}
