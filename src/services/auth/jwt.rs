use std::collections::HashSet;
use std::sync::Arc;

use dashmap::mapref::one::Ref;
use hmac::digest::core_api::CoreProxy;
use hmac::digest::KeyInit;
use hmac::Hmac;
use jwt::{AlgorithmType, Claims, Error, Header, PKeyWithDigest, Token, Unverified, VerifyingAlgorithm, VerifyWithKey};
use openssl::hash::MessageDigest;
use openssl::pkey::PKey;
use sha2::{Sha256, Sha384, Sha512};
use sled::Db;

use crate::{CacheBackedService, LocalSecretsService, UpdatableService};
use crate::models::auth::jwt::{JwtAlg, JwtKey, JwtKeyUpdateCommand};
use crate::models::auth::roles::{AuthOutcome, Role, RoleToken};
use crate::models::errors::{AuthError, StorageError};
use crate::models::versioning::versioned::Versioned;
use crate::services::clock::Clock;

const BEARER: &str = "Bearer ";
const SEPARATOR: &str = ".";

const JWT_KEYS: &str = "jwt_keys_v1";

pub struct JwtAuthService {

    clock: Clock,
    delegate: CacheBackedService<JwtKeyUpdateCommand, JwtKey>

}

impl JwtAuthService {

    pub fn new(clock: Clock,
               db: Arc<Db>,
               secrets_service: Arc<LocalSecretsService>) -> Result<JwtAuthService, StorageError> {
        Ok(
            JwtAuthService {
                clock,
                delegate: CacheBackedService::new(db, JWT_KEYS, secrets_service)?
            }
        )
    }

    pub fn validate_and_extract(&self,
                                auth_header: &str,
                                role: &Role) -> Result<RoleToken, AuthError> {
        let token = auth_header.replace(BEARER, "");

        let parsed: Token<Header, Claims, _> =
            Token::parse_unverified(&token).unwrap();

        let header = parsed.header();

        if header.algorithm == AlgorithmType::None {
            return Err(AuthError::NoneAlgorithmProvided);
        }

        let claims = parsed.claims();

        let invalid_claims = !self.are_valid_claims(claims);

        if invalid_claims {
            return Err(AuthError::InvalidClaims);
        }

        let jwt_token: JwtTokenData = token.as_str().try_into()?;

        let id = JwtKey::build_id(
            &header.key_id,
            &header.algorithm.try_into()?
        );

        match self.delegate.get(&id)? {
            None => Err(AuthError::UnknownKey),
            Some(value) => {
                match Self::verify(value.value().get_value(), &jwt_token) {
                    Ok(value) => {
                        if value {
                            Ok(())
                        } else {
                            Err(AuthError::VerificationError("Invalid token".into()))
                        }
                    }
                    Err(err) => Err(err)
                }
            }
        }?;

        let role_token: RoleToken = claims.try_into()?;

        if role_token.get_role().ne(role) {
            return Err(AuthError::Forbidden);
        }

        Ok(role_token)
    }

    /// The subject and expiration is required and checked while 'not before' is checked when present.
    #[inline]
    fn are_valid_claims(&self,
                        claims: &Claims) -> bool {
        let registered = &claims.registered;

        let now = self.clock.now_seconds();

        registered.subject.is_some()
            && registered.expiration
            .map(|exp| now < exp)
            .unwrap_or(false)
            && registered.not_before
            .map(|not_before| now > not_before)
            .unwrap_or(true)
    }

    #[inline]
    fn verify(jwt_key: &JwtKey,
              token: &JwtTokenData) -> Result<bool, AuthError> {
        let key = jwt_key.get_value();

        Ok(
            match jwt_key.get_alg() {
                JwtAlg::Hs256 => {
                    let hmac: Hmac<Sha256> = Hmac::new_from_slice(key)
                        .map_err(|err| AuthError::HmacError)?;

                    hmac.verify(token.header, token.claims, token.signature)?
                },
                JwtAlg::Hs384 => {
                    let hmac: Hmac<Sha384> = Hmac::new_from_slice(key)
                        .map_err(|err| AuthError::HmacError)?;

                    hmac.verify(token.header, token.claims, token.signature)?
                },
                JwtAlg::Hs512 => {
                    let hmac: Hmac<Sha512> = Hmac::new_from_slice(key)
                        .map_err(|err| AuthError::HmacError)?;

                    hmac.verify(token.header, token.claims, token.signature)?
                },
                JwtAlg::Rs256 | JwtAlg::Es256 =>
                    Self::verify_pk_digest(key, token, MessageDigest::sha256())?,
                JwtAlg::Rs384 | JwtAlg::Es384 =>
                    Self::verify_pk_digest(key, token, MessageDigest::sha384())?,
                JwtAlg::Rs512 | JwtAlg::Es512 =>
                    Self::verify_pk_digest(key, token, MessageDigest::sha512())?
            }
        )
    }

    #[inline]
    fn verify_pk_digest(key: &[u8],
                        token: &JwtTokenData,
                        digest: MessageDigest) -> Result<bool, AuthError> {
        let algo = PKeyWithDigest {
            digest,
            key: PKey::public_key_from_pem(key).map_err(|err| AuthError::PemError)?,
        };

        Ok( algo.verify(token.header, token.claims, token.signature)? )
    }
}

struct JwtTokenData<'a> {
    header: &'a str,
    claims: &'a str,
    signature: &'a str
}

impl<'a> TryFrom<&'a str> for JwtTokenData<'a> {
    type Error = AuthError;

    /// Taken from the `jwt` crate, should be deleted when access to the signature is provided.
    fn try_from(raw: &'a str) -> Result<Self, Self::Error> {
        let mut components = raw.split(SEPARATOR);
        let header = components.next().ok_or(AuthError::InvalidHeader)?;
        let claims = components.next().ok_or(AuthError::InvalidHeader)?;
        let signature = components.next().ok_or(AuthError::InvalidHeader)?;

        if components.next().is_some() {
            return Err(AuthError::InvalidHeader);
        }

        Ok(JwtTokenData {
            header,
            claims,
            signature
        })
    }
}

impl UpdatableService<JwtKeyUpdateCommand, JwtKey> for JwtAuthService {
    fn get(&self, id: &str) -> Result<Option<Ref<String, Versioned<JwtKey>>>, StorageError> {
        self.delegate.get(id)
    }

    fn get_all_keys(&self) -> Result<HashSet<String>, StorageError> {
        self.get_all_keys()
    }

    fn create(&self, payload: JwtKey) -> Result<Versioned<JwtKey>, StorageError> {
        self.delegate.create(payload)
    }

    fn update(&self, id: &str, command: JwtKeyUpdateCommand) -> Result<Versioned<JwtKey>, StorageError> {
        self.delegate.update(id, command)
    }

    fn delete(&self, id: &str) -> Result<(), StorageError> {
        self.delegate.delete(id)
    }

    fn clear(&self) -> Result<(), ()> {
        self.delegate.clear()
    }
}
