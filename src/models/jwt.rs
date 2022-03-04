use std::fmt::Display;
use jwt::AlgorithmType;
use serde::{Deserialize, Serialize};
use zeroize::Zeroize;
use crate::models::errors::AuthError;

use crate::models::updatable::{Updatable, UpdateCommand};
use crate::models::versioned::VersionHeader;

#[derive(Zeroize)]
#[zeroize(drop)]
#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Debug)]
pub struct JwtKey {

    id: String,
    kid: Option<String>,
    alg: JwtAlg,
    value: Vec<u8>

}

#[derive(Zeroize)]
#[zeroize(drop)]
#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Debug)]
pub struct JwtKeyUpdateCommand {

    header: VersionHeader,
    value: Vec<u8>

}

impl JwtKey {

    pub fn new(kid: Option<String>,
               alg: JwtAlg,
               value: Vec<u8>) -> JwtKey {
        JwtKey {
            id: JwtKey::build_id(&kid, &alg),
            kid,
            alg,
            value
        }
    }

    pub fn build_id(kid: &Option<String>,
                    alg: &JwtAlg) -> String {
        format!("{}-{:?}",
                kid.as_deref().unwrap_or_else(|| "none"), &alg)
            .to_lowercase()
    }

    pub fn get_alg(&self) -> &JwtAlg {
        &self.alg
    }

    pub fn get_value(&self) -> &[u8] {
        &self.value
    }

}

impl Updatable<JwtKeyUpdateCommand> for JwtKey {
    fn get_id(&self) -> &str {
        &self.id
    }

    fn accept(&self, update: JwtKeyUpdateCommand) -> Self {
        JwtKey {
            id: self.id.clone(),
            kid: self.kid.clone(),
            alg: self.alg.clone(),
            value: update.value.clone()
        }
    }
}

impl UpdateCommand for JwtKeyUpdateCommand {
    fn get_version_header(&self) -> &VersionHeader {
        &self.header
    }
}

#[derive(Zeroize)]
#[zeroize(drop)]
#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Debug)]
pub enum JwtAlg {

    Hs256,
    Hs384,
    Hs512,
    Rs256,
    Rs384,
    Rs512,
    Es256,
    Es384,
    Es512,

}

impl TryFrom<AlgorithmType> for JwtAlg {
    type Error = AuthError;

    fn try_from(value: AlgorithmType) -> Result<Self, Self::Error> {
        match value {
            AlgorithmType::None => Err(AuthError::UnsupportedAlgorithm),
            AlgorithmType::Hs256 => Ok(JwtAlg::Hs256),
            AlgorithmType::Hs384 => Ok(JwtAlg::Hs384),
            AlgorithmType::Hs512 => Ok(JwtAlg::Hs512),
            AlgorithmType::Rs256 => Ok(JwtAlg::Rs256),
            AlgorithmType::Rs384 => Ok(JwtAlg::Rs384),
            AlgorithmType::Rs512 => Ok(JwtAlg::Rs512),
            AlgorithmType::Es256 => Ok(JwtAlg::Es256),
            AlgorithmType::Es384 => Ok(JwtAlg::Es384),
            AlgorithmType::Es512 => Ok(JwtAlg::Es512),
            _ => Err(AuthError::UnsupportedAlgorithm)
        }
    }
}
