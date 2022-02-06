use std::fs;
use std::path::Path;
use std::sync::RwLock;

#[cfg(test)]
use mockall::automock;

use crate::models::errors::SecretsError;
use crate::models::secrets::EncryptedPayload;

const ENCRYPTION_KEY_PATH: &str = "./stored/pk";
const AAD_PATH: &str = "./stored/pa";

pub struct SimpleFilesService {

    lock: RwLock<()>

}

#[cfg_attr(test, automock)]
pub trait FilesService {
    fn read_key(&self) -> Result<EncryptedPayload, SecretsError>;

    fn store_key(&self, payload: EncryptedPayload) -> Result<(), SecretsError>;

    fn exists_key(&self) -> Result<bool, SecretsError>;

    fn read_aad(&self) -> Result<Vec<u8>, SecretsError>;

    fn store_aad(&self, aad: Vec<u8>) -> Result<(), SecretsError>;

    fn exists_aad(&self) -> Result<bool, SecretsError>;
}

impl SimpleFilesService {

    pub fn new() -> SimpleFilesService {
        SimpleFilesService {
            lock: RwLock::new(())
        }
    }

    fn read(&self,
            path: &str) -> Result<Vec<u8>, SecretsError> {
        match self.lock.read() {
            Ok(_) => fs::read(path)
                .map_err(SimpleFilesService::file_read_err),
            Err(_) => Err(SecretsError::LockError)
        }
    }

    fn store(&self,
             payload: Vec<u8>,
             path: &str) -> Result<(), SecretsError> {
        match self.lock.write() {
            Ok(_) => fs::write(path, payload)
                .map_err(SimpleFilesService::file_read_err),
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

impl FilesService for SimpleFilesService {
    fn read_key(&self) -> Result<EncryptedPayload, SecretsError> {
        Ok(bincode::deserialize(&self.read(ENCRYPTION_KEY_PATH)?)?)
    }

    fn store_key(&self,
                 payload: EncryptedPayload) -> Result<(), SecretsError> {
        self.store(bincode::serialize(&payload)?, ENCRYPTION_KEY_PATH)
    }

    fn exists_key(&self) -> Result<bool, SecretsError> {
        self.exists(ENCRYPTION_KEY_PATH)
    }

    fn read_aad(&self) -> Result<Vec<u8>, SecretsError> {
        self.read(AAD_PATH)
    }

    fn store_aad(&self,
                 aad: Vec<u8>) -> Result<(), SecretsError> {
        self.store(aad, AAD_PATH)
    }

    fn exists_aad(&self) -> Result<bool, SecretsError> {
        self.exists(AAD_PATH)
    }
}