use std::fs;
use std::fs::File;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicPtr, Ordering};

use chacha20poly1305::aead::{Aead, AeadInPlace, Error, NewAead, Payload};
use chacha20poly1305::aead::generic_array::GenericArray;
use chacha20poly1305::aead::generic_array::typenum::Or;
use chacha20poly1305::XChaCha20Poly1305;
use ring::rand::{SecureRandom, SystemRandom};
use sharks::{Share, Sharks};

use crate::models::connections::ZeroizeWrapper;
use crate::models::errors::SecretsError;
use crate::models::secrets::{EncryptedPayload, EncryptionKey, KEY_LENGTH, MasterKey, MasterKeyShare, MasterKeySharePayload};
use crate::services::secrets::encryption::EncryptionService;
use crate::services::secrets::files::SecretFilesService;
use crate::services::secrets::generate::VecGenerator;
use crate::services::secrets::shares::MasterKeySharesService;

const MINIMUM_SHARES_THRESHOLD: u8 = 8;
const NONCE_SIZE: usize = 24;

pub mod generate;
pub mod shares;

mod files;
mod encryption;

pub struct SecretsService {

    encryption_service: EncryptionService,
    files_service: SecretFilesService,
    is_sealed: AtomicBool,
    shares_service: Arc<MasterKeySharesService>,
    vec_generator: Arc<VecGenerator>,

}

impl SecretsService {

    pub fn new(shares_service: Arc<MasterKeySharesService>,
               vec_generator: Arc<VecGenerator>) -> SecretsService {
        SecretsService::from(SecretFilesService::new(), shares_service, vec_generator)
    }

    pub fn from(files_service: SecretFilesService,
                shares_service: Arc<MasterKeySharesService>,
                vec_generator: Arc<VecGenerator>) -> SecretsService {
        SecretsService {
            encryption_service: EncryptionService::new(vec_generator.clone()),
            files_service,
            is_sealed: AtomicBool::new(true),
            shares_service,
            vec_generator
        }
    }

    pub fn encrypt(&self, target: &Vec<u8>) -> Result<EncryptedPayload, SecretsError> {
        if self.is_sealed() {
            return Err(SecretsError::Sealed);
        }

        self.encryption_service.encrypt(target)
    }

    pub fn decrypt(&self, target: &EncryptedPayload) -> Result<ZeroizeWrapper, SecretsError> {
        if self.is_sealed() {
            return Err(SecretsError::Sealed);
        }

        self.encryption_service.decrypt(target)
    }

    pub fn initialize(&self) -> Result<Vec<MasterKeySharePayload>, SecretsError> {
        if !self.is_sealed()
            || self.files_service.exists_key()?
            || self.files_service.exists_aad()? {
            return Err(SecretsError::AlreadyInitialized);
        }

        let enc_key = EncryptionKey::new(
            self.vec_generator.generate_random(KEY_LENGTH)?
        );

        let master_key = MasterKey::new(
            self.vec_generator.generate_random(KEY_LENGTH)?
        );

        let aad = self.vec_generator.generate_random(KEY_LENGTH)?;

        let nonce = self.vec_generator.generate_random(NONCE_SIZE)?;

        let encrypted_enc_key =
            XChaCha20Poly1305::new(
                GenericArray::from_slice(master_key.get_value()))
                .encrypt(&GenericArray::from_slice(&nonce),
                         Payload {
                             msg: enc_key.get_value(),
                             aad: &aad
                         })?;

        self.files_service.store_key(
            EncryptedPayload::new(nonce, encrypted_enc_key))?;
        self.files_service.store_aad(aad)?;

        let sharks = Sharks(MINIMUM_SHARES_THRESHOLD);

        Ok(
            sharks
                .dealer(master_key.get_value())
                .take(MINIMUM_SHARES_THRESHOLD as usize)
                .map(|share|
                    MasterKeyShare::new((&share).into()).into())
                .collect()
        )
    }

    pub fn unseal(&self) -> Result<(), SecretsError> {
        if !self.is_sealed() {
            return Err(SecretsError::AlreadyUnsealed);
        }

        let master_key_shares = self.shares_service.get();

        let sharks = Sharks(MINIMUM_SHARES_THRESHOLD);

        let mut shares = vec![];

        for master_key_share in master_key_shares.iter() {
            shares.push(
                Share::try_from(master_key_share.get_value())
                    .map_err(|err| SecretsError::MasterKeyShareError(err.to_string()))?
            );
        }

        let master_key = MasterKey::new(
            sharks
                .recover(&shares)
                .map_err(|err| SecretsError::MasterKeyShareError(err.to_string()))?
        );

        let mut encrypted_enc_key = self.files_service.read_key()?;
        let aad = self.files_service.read_aad()?;

        let enc_key =
            XChaCha20Poly1305::new(GenericArray::from_slice(master_key.get_value()))
            .decrypt(&GenericArray::from_slice(encrypted_enc_key.get_nonce()),
                     Payload {
                         msg: encrypted_enc_key.get_payload(),
                         aad: &aad
                     })?;

        self.encryption_service.set_key(
            ZeroizeWrapper::new(aad), EncryptionKey::new(enc_key))?;

        self.is_sealed.store(false, Ordering::Relaxed);

        Ok(())
    }

    pub fn is_sealed(&self) -> bool {
        self.is_sealed.load(Ordering::Relaxed)
    }

}
