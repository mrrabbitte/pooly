use std::sync::Arc;

use ring::rand::SystemRandom;

use crate::services::connections::config::ConnectionConfigService;
use crate::services::connections::ConnectionService;
use crate::services::queries::QueryService;
use crate::services::secrets::{LocalSecretsService, SecretServiceFactory};
use crate::services::secrets::generate::VecGenerator;
use crate::services::secrets::shares::MasterKeySharesService;

pub mod resources;
pub mod models;
pub mod services;

pub struct AppContext {

    pub connection_config_service: Arc<ConnectionConfigService>,
    pub secrets_service: Arc<LocalSecretsService>,
    pub shares_service: Arc<MasterKeySharesService>,
    pub query_service: Arc<QueryService>,
    pub vec_generator: Arc<VecGenerator>

}

impl AppContext {

    pub fn new() -> AppContext {
        let vec_generator =
            Arc::new(
                VecGenerator::new(
                    Arc::new(SystemRandom::new())));

        let shares_service = Arc::new(MasterKeySharesService::new());

        let secrets_service: Arc<LocalSecretsService> =
            Arc::new(
                SecretServiceFactory::create(
                    shares_service.clone(), vec_generator.clone()));

        let connection_config_service =
            Arc::new(ConnectionConfigService::new(secrets_service.clone()));

        let query_service =
            Arc::new(QueryService::new(
                ConnectionService::new(
                    connection_config_service.clone())));

        AppContext {
            connection_config_service,
            secrets_service,
            shares_service,
            query_service,
            vec_generator
        }
    }

}