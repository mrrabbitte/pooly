use std::sync::Arc;

use crate::{JwtAuthService, LocalSecretsService, MasterKeySharesService, UpdatableService};
use crate::models::errors::InitializationError;
use crate::models::auth::jwt::JwtKey;
use crate::models::sec::secrets::MasterKeySharePayload;

pub struct InitializationService {

    auth_service: Arc<JwtAuthService>,
    shares_service: Arc<MasterKeySharesService>,
    secrets_service: Arc<LocalSecretsService>,

}

impl InitializationService {

    pub fn new(auth_service: Arc<JwtAuthService>,
               shares_service: Arc<MasterKeySharesService>,
               secrets_service: Arc<LocalSecretsService>) -> InitializationService {
        InitializationService {
            auth_service,
            shares_service,
            secrets_service
        }
    }

    pub fn initialize(&self,
                      jwt_key: JwtKey) -> Result<Vec<MasterKeySharePayload>, InitializationError> {
        let shares = self.secrets_service.initialize()?;

        self.shares_service.add_all(&shares).map_err(|_| InitializationError::TooManyShares)?;

        self.secrets_service.unseal()?;

        self.auth_service.create(jwt_key)?;

        Ok(shares.into_iter().map(|share| share.into()).collect())
    }

    pub fn clear(&self) -> Result<(), InitializationError> {
        self.secrets_service.clear()?;

        self.auth_service.clear().map_err(|_| InitializationError::AuthClearError)?;

        self.shares_service.clear();

        Ok(())
    }

}
