use std::fs;
use std::fs::Metadata;
use std::path::Path;
use std::sync::{LockResult, RwLock};

use crate::models::errors::SecretsError;
use crate::models::secrets::{EncryptedPayload, EncryptionKey};

const ENCRYPTION_KEY_PATH: &str = "./stored/pk";
const AAD_PATH: &str = "./stored/pa";

pub struct SecretFilesService {

    lock: RwLock<()>

}

impl SecretFilesService {

    pub fn new() -> SecretFilesService {
        SecretFilesService {
            lock: RwLock::new(())
        }
    }

    pub fn read_key(&self) -> Result<EncryptedPayload, SecretsError> {
        Ok(bincode::deserialize(&self.read(ENCRYPTION_KEY_PATH)?)?)
    }

    pub fn store_key(&self,
                     payload: EncryptedPayload) -> Result<(), SecretsError> {
        self.store(bincode::serialize(&payload)?, ENCRYPTION_KEY_PATH)
    }

    pub fn exists_key(&self) -> Result<bool, SecretsError> {
        self.exists(ENCRYPTION_KEY_PATH)
    }

    pub fn read_aad(&self) -> Result<Vec<u8>, SecretsError> {
        self.read(AAD_PATH)
    }

    pub fn store_aad(&self,
                     aad: Vec<u8>) -> Result<(), SecretsError> {
        self.store(aad, AAD_PATH)
    }

    pub fn exists_aad(&self) -> Result<bool, SecretsError> {
        self.exists(AAD_PATH)
    }

    fn read(&self,
            path: &str) -> Result<Vec<u8>, SecretsError> {
        match self.lock.read() {
            Ok(_) => fs::read(path)
                .map_err(SecretFilesService::file_read_err),
            Err(_) => Err(SecretsError::LockError)
        }
    }

    fn store(&self,
             payload: Vec<u8>,
             path: &str) -> Result<(), SecretsError> {
        match self.lock.write() {
            Ok(_) => fs::write(path, payload)
                .map_err(SecretFilesService::file_read_err),
            Err(_) => Err(SecretsError::LockError)
        }
    }

    fn exists(&self,
              path: &str) -> Result<bool, SecretsError> {
        match self.lock.read() {
            Ok(_) => Ok(Path::new(path).exists()),
            Err(_) => Err(SecretsError::LockError)
        }
    }

    fn file_read_err<T>(_: T) -> SecretsError {
        SecretsError::FileReadError
    }

}