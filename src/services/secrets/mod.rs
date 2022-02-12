use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use chacha20poly1305::aead::{Aead, NewAead, Payload};
use chacha20poly1305::aead::generic_array::GenericArray;
use chacha20poly1305::XChaCha20Poly1305;
use sharks::{Share, Sharks};

use crate::models::connections::ZeroizeWrapper;
use crate::models::errors::SecretsError;
use crate::models::secrets::{EncryptedPayload, EncryptionKey, KEY_LENGTH, MasterKey, MasterKeyShare, MasterKeySharePayload};
use crate::services::secrets::encryption::EncryptionService;
use crate::data::files::{FilesService, SimpleFilesService};
use crate::services::secrets::generate::VecGenerator;
use crate::services::secrets::shares::MasterKeySharesService;

const MINIMUM_SHARES_THRESHOLD: u8 = 8;
const NONCE_SIZE: usize = 24;

pub mod generate;
pub mod shares;

mod encryption;

pub type LocalSecretsService = SecretsService<SimpleFilesService>;

pub struct SecretsService<T: FilesService> {

    encryption_service: EncryptionService,
    files_service: T,
    is_sealed: AtomicBool,
    shares_service: Arc<MasterKeySharesService>,
    vec_generator: Arc<VecGenerator>,

}

pub struct SecretServiceFactory;

impl SecretServiceFactory {

    pub fn create(shares_service: Arc<MasterKeySharesService>,
                  vec_generator: Arc<VecGenerator>) -> LocalSecretsService {
        SecretsService::new(SimpleFilesService::new(), shares_service, vec_generator)
    }

}

impl<T: FilesService> SecretsService<T> {

    fn new(files_service: T,
           shares_service: Arc<MasterKeySharesService>,
           vec_generator: Arc<VecGenerator>) -> SecretsService<T> {
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

        let encrypted_enc_key = self.files_service.read_key()?;
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

    pub fn clear(&self) -> Result<(), SecretsError> {
        self.files_service.clear()
    }

}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, RwLock};

    use ring::rand::SystemRandom;

    use crate::models::errors::SecretsError;
    use crate::models::secrets::EncryptedPayload;
    use crate::data::files::MockFilesService;
    use crate::services::secrets::generate::VecGenerator;
    use crate::services::secrets::SecretsService;
    use crate::services::secrets::shares::MasterKeySharesService;

    #[test]
    fn test_returns_error_on_encdec_when_sealed() {
        let service = build_default_service(MockFilesService::new());

        assert!(service.encrypt(&vec![1,2,3]).is_err());
        assert!(service.decrypt(
            &EncryptedPayload::new(vec![1,1], vec![3,4])).is_err());
    }

    #[test]
    fn test_initializes_and_useals_correctly() {
        let mut mock = MockFilesService::new();

        mock.expect_exists_key()
            .times(1)
            .returning(|| Ok(false));

        mock.expect_exists_aad()
            .times(1)
            .returning(|| Ok(false));

        let key_spy = Arc::new(
            FileSpy::new(EncryptedPayload::new(vec![], vec![])));
        let ks_ptr_one = key_spy.clone();
        let ks_ptr_two = key_spy.clone();

        mock.expect_store_key()
            .times(1)
            .returning(move |key| ks_ptr_one.consume(key));

        mock.expect_read_key()
            .times(1)
            .returning(move || Ok(ks_ptr_two.get()));

        let aad_spy: Arc<FileSpy<Vec<u8>>> = Arc::new(FileSpy::new(Vec::new()));
        let as_ptr_one = aad_spy.clone();
        let as_ptr_two = aad_spy.clone();

        mock.expect_store_aad()
            .times(1)
            .returning(move |aad| as_ptr_one.consume(aad));

        mock.expect_read_aad()
            .times(1)
            .returning(move || Ok(as_ptr_two.get()));

        let shares_service = Arc::new(MasterKeySharesService::new());

        let service =
            build_service(mock, shares_service.clone());

        let shares = service.initialize().unwrap();

        assert!(!shares.is_empty());

        let saved_key = key_spy.get();
        let saved_aad = aad_spy.get();

        assert!(!saved_key.get_payload().is_empty());
        assert!(!saved_key.get_nonce().is_empty());
        assert!(!saved_aad.is_empty());

        shares
            .into_iter()
            .for_each(|share|
                shares_service.add(share.try_into().unwrap()));

        assert!(service.unseal().is_ok());

        let payload = vec![1,2,3,4];

        assert_eq!(
            service.decrypt(&service.encrypt(&payload).unwrap()).unwrap().get_value(),
            &payload);

        assert!(service.unseal().is_err());
        assert!(service.initialize().is_err());

        assert_eq!(
            service.decrypt(&service.encrypt(&payload).unwrap()).unwrap().get_value(),
            &payload);
    }

    #[test]
    fn test_will_not_initialize_if_key_data_is_present() {
        let mut mock = MockFilesService::new();

        mock.expect_exists_key()
            .times(1)
            .returning(|| Ok(true));

        let service = build_default_service(mock);

        assert!(service.initialize().is_err());
    }

    fn build_service(mock: MockFilesService,
                     key_shares_service: Arc<MasterKeySharesService>)
        -> SecretsService<MockFilesService> {
        SecretsService::new(mock,
                            key_shares_service,
                            Arc::new(
                                VecGenerator::new(
                                    Arc::new(SystemRandom::new()))))
    }

    fn build_default_service(mock: MockFilesService) -> SecretsService<MockFilesService> {
        build_service(mock, Arc::new(MasterKeySharesService::new()))
    }

    struct FileSpy<T: Clone> {
        data: RwLock<T>
    }

    impl<T: Clone> FileSpy<T> {

        fn new(initial: T) -> FileSpy<T> {
            FileSpy {
                data: RwLock::new(initial)
            }
        }

        fn consume(&self,
                   data: T) -> Result<(), SecretsError> {
            *self.data.write().unwrap() = data;
            Ok(())
        }

        fn get(&self) -> T {
            self.data.read().unwrap().clone()
        }
    }

}
