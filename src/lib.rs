use std::sync::Arc;

use ring::rand::SystemRandom;

use crate::data::dao::{TypedDao, UpdatableDao};
use crate::data::db::DbBuilder;
use crate::services::auth::access::AccessControlService;
use crate::services::auth::connection_ids::{LiteralConnectionIdAccessEntryService, WildcardPatternConnectionIdAccessEntryService};
use crate::services::auth::identity::AuthService;
use crate::services::clock::Clock;
use crate::services::connections::config::ConnectionConfigService;
use crate::services::connections::ConnectionService;
use crate::services::initialize::InitializationService;
use crate::services::queries::QueryService;
use crate::services::secrets::{LocalSecretsService, SecretServiceFactory};
use crate::services::secrets::random::VecGenerator;
use crate::services::secrets::shares::MasterKeySharesService;
use crate::services::updatable::{CacheBackedService, UpdatableService};

pub mod data;
pub mod resources;
pub mod models;
pub mod services;

pub struct AppContext {

    pub access_control_service: Arc<AccessControlService>,
    pub auth_service: Arc<AuthService>,
    pub connection_config_service: Arc<ConnectionConfigService>,
    pub initialization_service: Arc<InitializationService>,
    pub literal_ids_service: Arc<LiteralConnectionIdAccessEntryService>,
    pub secrets_service: Arc<LocalSecretsService>,
    pub shares_service: Arc<MasterKeySharesService>,
    pub query_service: Arc<QueryService>,
    pub pattern_ids_service: Arc<WildcardPatternConnectionIdAccessEntryService>,
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

        let db = DbBuilder::new();

        let connection_config_service =
            Arc::new(
                ConnectionConfigService::new(
                    db.clone(), secrets_service.clone()).unwrap()
            );

        let literal_ids_service =
            Arc::new(
                LiteralConnectionIdAccessEntryService::new(
                    db.clone(), secrets_service.clone()).unwrap()
            );

        let pattern_ids_service =
            Arc::new(
                WildcardPatternConnectionIdAccessEntryService::new(
                    db.clone(), secrets_service.clone()).unwrap()
            );

        let access_control_service =
            Arc::new(
                AccessControlService::new(
                    literal_ids_service.clone(),
                    pattern_ids_service.clone())
            );

        let query_service =
            Arc::new(QueryService::new(
                access_control_service.clone(),
                ConnectionService::new(
                    connection_config_service.clone())));

        let auth_service = Arc::new(
            AuthService::new(
                Clock::new(), db.clone(), secrets_service.clone())
                .unwrap()
        );

        let initialization_service = Arc::new(
            InitializationService::new(
                auth_service.clone(),
                shares_service.clone(),
                secrets_service.clone())
        );

        AppContext {
            auth_service,
            access_control_service,
            connection_config_service,
            initialization_service,
            literal_ids_service,
            secrets_service,
            shares_service,
            query_service,
            pattern_ids_service,
            vec_generator
        }
    }

}