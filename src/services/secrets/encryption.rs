use std::ops::Deref;
use std::sync::{Arc, LockResult, RwLock};
use std::sync::atomic::{AtomicBool, AtomicPtr, Ordering};

use chacha20poly1305::aead::{Aead, Error, NewAead, Payload};
use chacha20poly1305::aead::generic_array::GenericArray;
use chacha20poly1305::XChaCha20Poly1305;

use crate::models::connections::ZeroizeWrapper;
use crate::models::errors::SecretsError;
use crate::models::secrets::{EncryptedPayload, EncryptionKey};
use crate::services::secrets::generate::VecGenerator;

const NONCE_SIZE: usize = 24;

pub struct EncryptionService {

    aad: ZeroizeWrapper,
    encryption_key: RwLock<XChaCha20Poly1305>,
    is_sealed: AtomicBool,
    vec_generator: Arc<VecGenerator>

}

impl EncryptionService {

    pub fn encrypt(&self,
                   payload: &Vec<u8>) -> Result<EncryptedPayload, SecretsError> {
        let nonce = self.generate_nonce()?;

        let encrypted = self.encryption_key.read()?
            .encrypt(&GenericArray::from_slice(&nonce),
                     Payload {
                         msg: payload,
                         aad: &Vec::new()
                     })?;

        Ok(EncryptedPayload::new(nonce, encrypted))
    }

    pub fn decrypt(&self,
                   target: &EncryptedPayload) -> Result<ZeroizeWrapper, SecretsError> {
        let decrypted = self.encryption_key.read()?
            .decrypt(&GenericArray::from_slice(target.get_nonce()),
                     Payload {
                         msg: target.get_payload(),
                         aad: &Vec::new()
                     })?;

        Ok(ZeroizeWrapper::new(decrypted))
    }

    pub fn unseal(&self, enc_key: EncryptionKey) -> Result<(), SecretsError> {
        *self.encryption_key.write()? =
            XChaCha20Poly1305::new(
                GenericArray::from_slice(enc_key.get_value()));

        Ok(())
    }

    pub fn is_sealed(&self) -> bool {
        self.is_sealed.load(Ordering::Relaxed)
    }

    fn generate_nonce(&self) -> Result<Vec<u8>, SecretsError> {
        self.vec_generator.generate_random(NONCE_SIZE)
    }

}


#[cfg(test)]
mod tests {
    use std::sync::{Arc, RwLock};
    use std::sync::atomic::{AtomicBool, AtomicPtr};

    use chacha20poly1305::aead::{Aead, NewAead, Payload};
    use chacha20poly1305::aead::generic_array::GenericArray;
    use chacha20poly1305::XChaCha20Poly1305;
    use ring::rand::SystemRandom;

    use crate::models::connections::ZeroizeWrapper;
    use crate::models::secrets::{EncryptionKey, KEY_LENGTH};
    use crate::services::secrets::encryption::EncryptionService;
    use crate::services::secrets::generate::VecGenerator;

    #[test]
    fn test_encrypts_and_decrypts_correctly() {
        let encryption_service = build_service();

        let payload = vec![5; 256];

        let encrypted = encryption_service.encrypt(&payload).unwrap();

        let decrypted = encryption_service
            .decrypt(&encrypted)
            .unwrap();

        assert_eq!(&payload, decrypted.get_value());
    }

    fn build_service() -> EncryptionService {
        let key: Vec<u8> = vec![2; KEY_LENGTH];

        EncryptionService {
            aad: ZeroizeWrapper::new(vec![1; 10]),
            encryption_key:
            RwLock::new(XChaCha20Poly1305::new(
                GenericArray::from_slice(&key))),
            is_sealed: AtomicBool::new(false),
            vec_generator:
            Arc::new(
                VecGenerator::new(
                    Arc::new(
                        SystemRandom::new()
                    )
                )
            )
        }
    }

}
