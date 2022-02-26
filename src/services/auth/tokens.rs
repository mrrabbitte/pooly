use std::sync::Arc;

use jwt::algorithm::openssl::PKeyWithDigest;
use jwt::VerifyingAlgorithm;
use openssl::hash::MessageDigest;
use openssl::pkey::PKey;

use crate::AccessControlService;
use crate::data::dao::{EncryptedDao, TypedDao};
use crate::models::keys::PublicKeyPem;

pub struct TokensService {

    public_key: PublicKeyPem

}

impl TokensService {

    pub fn is_valid(&self) {


    }

}