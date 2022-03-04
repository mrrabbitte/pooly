use std::collections::HashSet;
use std::sync::Arc;

use dashmap::mapref::one::Ref;
use hmac::digest::core_api::CoreProxy;
use hmac::digest::KeyInit;
use hmac::Hmac;
use jwt::{AlgorithmType, Claims, Header, PKeyWithDigest, Token, Unverified, VerifyingAlgorithm, VerifyWithKey};
use openssl::hash::MessageDigest;
use openssl::pkey::PKey;
use sha2::{Sha256, Sha384, Sha512};
use sled::Db;

use crate::{CacheBackedService, LocalSecretsService, UpdatableService};
use crate::models::errors::{AuthError, StorageError};
use crate::models::jwt::{JwtAlg, JwtKey, JwtKeyUpdateCommand};
use crate::models::roles::AuthOutcome;
use crate::models::versioned::Versioned;
use crate::services::clock::Clock;

const BEARER: &str = "Bearer ";
const SEPARATOR: &str = ".";

const JWT_KEYS: &str = "jwt_keys_v1";

pub struct AuthService {

    clock: Clock,
    delegate: CacheBackedService<JwtKeyUpdateCommand, JwtKey>

}

impl AuthService {

    pub fn new(clock: Clock,
               db: Arc<Db>,
               secrets_service: Arc<LocalSecretsService>) -> Result<AuthService, StorageError> {
        Ok(
            AuthService {
                clock,
                delegate: CacheBackedService::new(db, JWT_KEYS, secrets_service)?
            }
        )
    }

    pub fn extract(&self,
                   auth_header: &str) -> Result<AuthOutcome, AuthError> {
        let token = auth_header.replace(BEARER, "");

        let parsed: Token<Header, Claims, _> =
            Token::parse_unverified(&token).unwrap();

        let header = parsed.header();

        if header.algorithm == AlgorithmType::None {
            return Ok(AuthOutcome::Unauthorised);
        }

        let claims = parsed.claims();

        if !self.are_valid_claims(claims) {
            return Ok(AuthOutcome::Unauthorised);
        }

        let jwt_token: JwtTokenData = token.as_str().try_into()?;

        let id = JwtKey::build_id(
            &header.key_id,
            &header.algorithm.try_into()?
        );

        let is_verified = match self.delegate.get(&id)? {
            None => Ok(false),
            Some(value) =>
                Self::verify(value.value().get_value(), &jwt_token)
        }?;

        if !is_verified {
            return Ok(AuthOutcome::Unauthorised);
        }

        Ok(AuthOutcome::Authorised(claims.try_into()?))
    }

    /// The subject and expiration is required and checked while 'not before' is checked when present.
    #[inline]
    fn are_valid_claims(&self,
                        claims: &Claims) ->bool {
        let registered = &claims.registered;

        let now = self.clock.now_seconds();

        registered.subject.is_some()
            && registered.expiration.map(|exp| now > exp)
            .unwrap_or(false)
            && registered.not_before.map(|not_before| now < not_before)
            .unwrap_or(true)
    }

    #[inline]
    fn verify(jwt_key: &JwtKey,
              token: &JwtTokenData) -> Result<bool, AuthError> {
        let key = jwt_key.get_value();
        match jwt_key.get_alg() {
            JwtAlg::Hs256 => {
                let hmac: Hmac<Sha384> = Hmac::new_from_slice(key)
                    .map_err(|err| AuthError::HmacError)?;

                hmac.verify(token.header, token.claims, token.signature)
                    .map_err(|err| AuthError::VerificationError)
            },
            JwtAlg::Hs384 => {
                let hmac: Hmac<Sha384> = Hmac::new_from_slice(key)
                    .map_err(|err| AuthError::HmacError)?;

                hmac.verify(token.header, token.claims, token.signature)
                    .map_err(|err| AuthError::VerificationError)
            },
            JwtAlg::Hs512 => {
                let hmac: Hmac<Sha384> = Hmac::new_from_slice(key)
                    .map_err(|err| AuthError::HmacError)?;

                hmac.verify(token.header, token.claims, token.signature)
                    .map_err(|err| AuthError::VerificationError)
            },
            JwtAlg::Rs256 | JwtAlg::Es256 =>
                Self::verify_pk_digest(key, token, MessageDigest::sha256()),
            JwtAlg::Rs384 | JwtAlg::Es384 =>
                Self::verify_pk_digest(key, token, MessageDigest::sha384()),
            JwtAlg::Rs512 | JwtAlg::Es512 =>
                Self::verify_pk_digest(key, token, MessageDigest::sha512())
        }
    }

    #[inline]
    fn verify_pk_digest(key: &[u8],
                        token: &JwtTokenData,
                        digest: MessageDigest) -> Result<bool, AuthError> {
        let algo = PKeyWithDigest {
            digest,
            key: PKey::public_key_from_pem(key).map_err(|err| AuthError::PemError)?,
        };

        algo.verify(token.header, token.claims, token.signature)
            .map_err(|err| AuthError::VerificationError)
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
        let header = components.next().ok_or(AuthError::InvalidToken)?;
        let claims = components.next().ok_or(AuthError::InvalidToken)?;
        let signature = components.next().ok_or(AuthError::InvalidToken)?;

        if components.next().is_some() {
            return Err(AuthError::InvalidToken);
        }

        Ok(JwtTokenData {
            header,
            claims,
            signature
        })
    }
}

impl UpdatableService<JwtKeyUpdateCommand, JwtKey> for AuthService {
    fn get(&self, id: &str) -> Result<Option<Ref<String, Versioned<JwtKey>>>, StorageError> {
        self.get(id)
    }

    fn get_all_keys(&self) -> Result<HashSet<String>, StorageError> {
        self.get_all_keys()
    }

    fn create(&self, payload: JwtKey) -> Result<Versioned<JwtKey>, StorageError> {
        self.create(payload)
    }

    fn update(&self, id: &str, command: JwtKeyUpdateCommand) -> Result<Versioned<JwtKey>, StorageError> {
        self.update(id, command)
    }

    fn delete(&self, id: &str) -> Result<(), StorageError> {
        self.delete(id)
    }

    fn clear(&self) -> Result<(), ()> {
        self.clear()
    }
}