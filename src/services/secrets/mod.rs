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
use crate::models::secrets::{EncryptionKey, KEY_LENGTH, MasterKey, MasterKeyShare, MasterKeySharePayload};
use crate::services::secrets::files::SecretFilesService;
use crate::services::secrets::shares::MasterKeySharesService;

const MINIMUM_SHARES_THRESHOLD: u8 = 8;
const NONCE_SIZE: usize = 24;

pub mod shares;

mod files;
mod generate;
mod encryption;

pub struct SecretsService {

    encryption_key: AtomicPtr<XChaCha20Poly1305>,
    files_service: SecretFilesService,
    is_sealed: AtomicBool,
    shares_service: Arc<MasterKeySharesService>,
    sys_random: Arc<SystemRandom>,

}

impl SecretsService {

    pub fn new(shares_service: Arc<MasterKeySharesService>,
               sys_random: Arc<SystemRandom>) -> SecretsService {
        SecretsService::from(SecretFilesService::new(), shares_service, sys_random)
    }

    pub fn from(files_service: SecretFilesService,
                shares_service: Arc<MasterKeySharesService>,
                sys_random: Arc<SystemRandom>) -> SecretsService {
        SecretsService {
            encryption_key: AtomicPtr::new(
                &mut XChaCha20Poly1305::new(&GenericArray::default())),
            files_service,
            is_sealed: AtomicBool::new(true),
            shares_service,
            sys_random
        }
    }

    pub fn encrypt(&self, target: &Vec<u8>) -> Result<Vec<u8>, SecretsError> {
        let nonce = self.generate_nonce()?;

        self.apply(
            |algo| algo.encrypt(
                &GenericArray::from_slice(nonce.get_value()),
                Payload {
                    msg: target,
                    aad: &Vec::new()
                }))
    }

    pub fn decrypt(&self, target: &[u8]) -> Result<Vec<u8>, SecretsError> {
        let nonce = self.generate_nonce()?;

        self.apply(
            |algo| algo.decrypt(
                &GenericArray::from_slice(nonce.get_value()),
                Payload {
                    msg: target,
                    aad: &Vec::new()
                }))
    }

    pub fn initialize(&self) -> Result<Vec<MasterKeySharePayload>, SecretsError> {
        if !self.is_sealed() || self.files_service.exists()? {
            return Err(SecretsError::AlreadyInitialized);
        }

        let enc_key = self.generate_encryption_key()?;

        let master_key = self.generate_master_key()?;

        let encrypted_enc_key =
            XChaCha20Poly1305::new(
                GenericArray::from_slice(master_key.get_value()))
                .encrypt(&GenericArray::from_slice(self.generate_nonce()?.get_value()),
                         Payload {
                             msg: enc_key.get_value(),
                             aad: &Vec::new()
                         })?;

        self.files_service.store(encrypted_enc_key)?;

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
                    .map_err(|err| SecretsError::MasterKeyShareError(err.to_string()))?);
        }

        let master_key = MasterKey::new(
            sharks.recover(&shares)
                .map_err(|err| SecretsError::MasterKeyShareError(err.to_string()))?);

        let mut enc_key = self.files_service.read()?;

        XChaCha20Poly1305::new(GenericArray::from_slice(master_key.get_value()))
            .decrypt_in_place(
                GenericArray::from_slice(self.generate_nonce()?.get_value()),
                &Vec::new(),
                &mut enc_key)?;

        self.encryption_key.store(
            &mut XChaCha20Poly1305::new(
                GenericArray::from_slice(
                    EncryptionKey::new(enc_key).get_value())),
            Ordering::Relaxed);

        self.is_sealed.store(false, Ordering::Relaxed);

        Ok(())
    }

    pub fn is_sealed(&self) -> bool {
        self.is_sealed.load(Ordering::Relaxed)
    }

    fn apply<T>(&self, operation: T) -> Result<Vec<u8>, SecretsError>
        where T: FnOnce(&XChaCha20Poly1305) -> Result<Vec<u8>, Error> {
        if self.is_sealed() {
            return Err(SecretsError::Sealed)
        }

        unsafe {
            match self.encryption_key.load(Ordering::Relaxed).as_ref() {
                Some(algo) =>
                    Ok( operation(algo)?),
                None => Err(SecretsError::Unspecified)
            }
        }
    }

    fn generate_nonce(&self) -> Result<ZeroizeWrapper, SecretsError> {
        Ok(ZeroizeWrapper::new(self.generate_random(NONCE_SIZE)?))
    }

    fn generate_encryption_key(&self) -> Result<EncryptionKey, SecretsError> {
        Ok(EncryptionKey::new(self.generate_key()?))
    }

    fn generate_master_key(&self) -> Result<MasterKey, SecretsError> {
        Ok(MasterKey::new(self.generate_key()?))
    }

    fn generate_key(&self) -> Result<Vec<u8>, SecretsError> {
        self.generate_random(KEY_LENGTH)
    }

    fn generate_random(&self,
                       size: usize) -> Result<Vec<u8>, SecretsError> {
        let mut value = vec![0; size];

        self.sys_random.fill(&mut value)?;

        Ok(value)
    }

}
