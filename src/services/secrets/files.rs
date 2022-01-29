use std::fs;
use std::fs::Metadata;
use std::path::Path;
use std::sync::{LockResult, RwLock};

use crate::models::errors::SecretsError;
use crate::models::secrets::EncryptionKey;

const ENCRYPTION_KEY_PATH: &str = "./stored/pooly_key";

pub struct SecretFilesService {

    lock: RwLock<()>

}

impl SecretFilesService {

    pub fn new() -> SecretFilesService {
        SecretFilesService {
            lock: RwLock::new(())
        }
    }

    pub fn read(&self) -> Result<Vec<u8>, SecretsError> {
        match self.lock.read() {
            Ok(_) => fs::read(ENCRYPTION_KEY_PATH)
                .map_err(SecretFilesService::file_read_err),
            Err(_) => Err(SecretsError::LockError)
        }
    }

    pub fn store(&self,
                 encrypted_enc_key: Vec<u8>) -> Result<(), SecretsError> {
        match self.lock.write() {
            Ok(_) => fs::write(ENCRYPTION_KEY_PATH, encrypted_enc_key)
                .map_err(SecretFilesService::file_read_err),
            Err(_) => Err(SecretsError::LockError)
        }
    }

    pub fn exists(&self) -> Result<bool, SecretsError> {
        match self.lock.read() {
            Ok(_) => Ok(Path::new(ENCRYPTION_KEY_PATH).exists()),
            Err(_) => Err(SecretsError::LockError)
        }
    }

    fn file_read_err<T>(_: T) -> SecretsError {
        SecretsError::FileReadError
    }

}