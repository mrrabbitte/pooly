use std::fs;
use std::path::Path;
use std::sync::RwLock;

#[cfg(test)]
use mockall::automock;

use crate::data::db::BASE_STORAGE_PATH;
use crate::models::errors::SecretsError;
use crate::models::sec::secrets::EncryptedPayload;

pub struct SimpleFilesService {

    enc_key_path: String,
    aad_path: String,
    lock: RwLock<()>,

}

#[cfg_attr(test, automock)]
pub trait FilesService {
    fn read_key(&self) -> Result<EncryptedPayload, SecretsError>;

    fn store_key(&self, payload: EncryptedPayload) -> Result<(), SecretsError>;

    fn exists_key(&self) -> Result<bool, SecretsError>;

    fn read_aad(&self) -> Result<Vec<u8>, SecretsError>;

    fn store_aad(&self, aad: Vec<u8>) -> Result<(), SecretsError>;

    fn exists_aad(&self) -> Result<bool, SecretsError>;

    fn clear(&self) -> Result<(), SecretsError>;
}

impl SimpleFilesService {

    pub fn new() -> SimpleFilesService {
        SimpleFilesService {
            aad_path: BASE_STORAGE_PATH.to_owned() + "/pa",
            enc_key_path: BASE_STORAGE_PATH.to_owned() + "/pk",
            lock: RwLock::new(())
        }
    }

    fn read(&self,
            path: &str) -> Result<Vec<u8>, SecretsError> {
        match self.lock.read() {
            Ok(_) => fs::read(path)
                .map_err(SimpleFilesService::file_err),
            Err(_) => Err(SecretsError::LockError)
        }
    }

    fn store(&self,
             payload: Vec<u8>,
             path: &str) -> Result<(), SecretsError> {
        match self.lock.write() {
            Ok(_) => fs::write(path, payload)
                .map_err(SimpleFilesService::file_err),
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

    fn remove_if_exists(path: &str) -> Result<(), SecretsError> {
        if Path::new(path).exists() {
            return fs::remove_file(path).map_err(SimpleFilesService::file_err);
        }

        Ok(())
    }

    fn file_err<T>(_: T) -> SecretsError {
        SecretsError::FileReadError
    }

}

impl FilesService for SimpleFilesService {
    fn read_key(&self) -> Result<EncryptedPayload, SecretsError> {
        Ok(bincode::deserialize(&self.read(&self.enc_key_path)?)?)
    }

    fn store_key(&self,
                 payload: EncryptedPayload) -> Result<(), SecretsError> {
        self.store(bincode::serialize(&payload)?, &self.enc_key_path)
    }

    fn exists_key(&self) -> Result<bool, SecretsError> {
        self.exists(&self.enc_key_path)
    }

    fn read_aad(&self) -> Result<Vec<u8>, SecretsError> {
        self.read(&self.aad_path)
    }

    fn store_aad(&self,
                 aad: Vec<u8>) -> Result<(), SecretsError> {
        self.store(aad, &self.aad_path)
    }

    fn exists_aad(&self) -> Result<bool, SecretsError> {
        self.exists(&self.aad_path)
    }

    fn clear(&self) -> Result<(), SecretsError> {
        match self.lock.write() {
            Ok(_) => {
                SimpleFilesService::remove_if_exists(&self.enc_key_path)
                    .and(
                        SimpleFilesService::remove_if_exists(&self.aad_path))
            },
            Err(_) => Err(SecretsError::LockError)
        }
    }
}