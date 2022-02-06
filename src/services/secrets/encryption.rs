use std::sync::{Arc, RwLock};

use chacha20poly1305::aead::{Aead, NewAead, Payload};
use chacha20poly1305::aead::generic_array::GenericArray;
use chacha20poly1305::XChaCha20Poly1305;
#[cfg(test)]
use mockall::automock;

use crate::models::connections::ZeroizeWrapper;
use crate::models::errors::SecretsError;
use crate::models::secrets::{EncryptedPayload, EncryptionKey, KEY_LENGTH};
use crate::services::secrets::generate::VecGenerator;

const NONCE_SIZE: usize = 24;

pub struct EncryptionService {

    key_with_aad: RwLock<KeyWithAad>,
    vec_generator: Arc<VecGenerator>

}

impl EncryptionService {

    pub fn new(vec_generator: Arc<VecGenerator>) -> EncryptionService {
        EncryptionService {
            key_with_aad: RwLock::new(KeyWithAad {
                aad: ZeroizeWrapper::new(Vec::new()),
                key: XChaCha20Poly1305::new(GenericArray::from_slice(&vec![0; KEY_LENGTH]))
            }),
            vec_generator
        }
    }

    pub fn encrypt(&self,
                   payload: &Vec<u8>) -> Result<EncryptedPayload, SecretsError> {
        let nonce = self.vec_generator.generate_random(NONCE_SIZE)?;

        let key_with_aad = self.key_with_aad.read()?;

        let encrypted = key_with_aad.key
            .encrypt(&GenericArray::from_slice(&nonce),
                     Payload {
                         msg: payload,
                         aad: key_with_aad.aad.get_value()
                     })?;

        Ok(EncryptedPayload::new(nonce, encrypted))
    }

    pub fn decrypt(&self,
                   target: &EncryptedPayload) -> Result<ZeroizeWrapper, SecretsError> {
        let key_with_aad = self.key_with_aad.read()?;

        let decrypted = key_with_aad.key
                .decrypt(&GenericArray::from_slice(target.get_nonce()),
                     Payload {
                         msg: target.get_payload(),
                         aad: key_with_aad.aad.get_value()
                     })?;

        Ok(ZeroizeWrapper::new(decrypted))
    }

    pub fn set_key(&self,
                   aad: ZeroizeWrapper,
                   enc_key: EncryptionKey) -> Result<(), SecretsError> {
        *self.key_with_aad.write()? =
            KeyWithAad {
                aad,
                key: XChaCha20Poly1305::new(
                    GenericArray::from_slice(enc_key.get_value()))
            };

        Ok(())
    }

}

struct KeyWithAad {

    aad: ZeroizeWrapper,
    key: XChaCha20Poly1305

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
    use crate::services::secrets::encryption::{EncryptionService, KeyWithAad};
    use crate::services::secrets::generate::VecGenerator;

    #[test]
    fn test_encrypts_and_decrypts_correctly() {
        check_encrypt_decrypt(&default_build_service(), &vec![5; 256]);
    }

    #[test]
    fn test_changes_key_correctly() {
        let mut key = vec![2; KEY_LENGTH];

        let encryption_service = build_service(key.clone());

        let payload = vec![1; 112];

        check_encrypt_decrypt(&encryption_service, &payload);

        let encrypted_with_old = encryption_service.encrypt(&payload).unwrap();

        key[0] = key[0] + 1;

        encryption_service.set_key(ZeroizeWrapper::new(vec![1; 10]),
                                   EncryptionKey::new(key.clone()))
            .unwrap();

        assert!(encryption_service.decrypt(&encrypted_with_old).is_err());

        check_encrypt_decrypt(&encryption_service, &payload);
    }

    fn check_encrypt_decrypt(encryption_service: &EncryptionService,
                             payload: &Vec<u8>) {
        let encrypted = encryption_service.encrypt(&payload).unwrap();

        let decrypted = encryption_service
            .decrypt(&encrypted)
            .unwrap();

        assert_eq!(payload, decrypted.get_value());
    }

    fn default_build_service() -> EncryptionService {
        build_service(vec![2; KEY_LENGTH])
    }

    fn build_service(key: Vec<u8>) -> EncryptionService {
        EncryptionService {
            key_with_aad:
            RwLock::new(KeyWithAad {
                aad: ZeroizeWrapper::new(vec![1; 10]),
                key: XChaCha20Poly1305::new(
                GenericArray::from_slice(&key))
            }),
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
