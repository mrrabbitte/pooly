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

    #[serde(with="base64")]
    value: Vec<u8>

}

impl JwtKeyUpdateCommand {

    pub fn new(header: VersionHeader,
               value: Vec<u8>) -> JwtKeyUpdateCommand {
        JwtKeyUpdateCommand {
            header,
            value
        }
    }

}

#[derive(Zeroize)]
#[zeroize(drop)]
#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Debug)]
pub struct JwtKeyCreateCommand {

    kid: Option<String>,
    alg: JwtAlg,

    #[serde(with="base64")]
    value: Vec<u8>

}

impl JwtKeyCreateCommand {

    pub fn new(kid: Option<String>,
               alg: JwtAlg,
               value: Vec<u8>) -> JwtKeyCreateCommand {
        JwtKeyCreateCommand {
            kid,
            alg,
            value
        }
    }

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

impl From<JwtKeyCreateCommand> for JwtKey {
    fn from(command: JwtKeyCreateCommand) -> Self {
        JwtKey::new(command.kid.clone(), command.alg.clone(), command.value.clone())
    }
}

mod base64 {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S: Serializer>(value: &Vec<u8>, serializer: S) -> Result<S::Ok, S::Error> {
        String::serialize(&base64::encode(value), serializer)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Vec<u8>, D::Error> {
        base64::decode(String::deserialize(deserializer)?.as_bytes())
            .map_err(|e| serde::de::Error::custom(e))
    }
}

#[cfg(test)]
mod tests {
    use std::fmt::Debug;

    use serde::{Deserialize, Serialize};
    use serde_json;

    use crate::models::jwt::{JwtAlg, JwtKey, JwtKeyCreateCommand, JwtKeyUpdateCommand};
    use crate::models::updatable::Updatable;
    use crate::models::versioned::VersionHeader;

    #[test]
    fn test_udpate_command() {
        let old = JwtKey::new(Some("kid-1".into()), JwtAlg::Es256, vec![1, 2, 3]);

        let command_value = vec![3,4,5];
        let command =
            JwtKeyUpdateCommand::new(VersionHeader::zero_version(), command_value.clone());

        let new = old.accept(command);

        assert_eq!(&old.id, &new.id);
        assert_eq!(&old.kid, &new.kid);
        assert_eq!(&old.alg, &new.alg);

        assert_ne!(&old.value, &new.value);
        assert_eq!(&new.value, &command_value);
    }

    #[test]
    fn test_serde() {
        check_serde(
            &JwtKeyCreateCommand::new(
                Some("kid-3".into()), JwtAlg::Rs256, vec![1, 4]));

        check_serde(&JwtKeyUpdateCommand::new(VersionHeader::zero_version(),
                                             vec![12,3]));

        check_serde(&JwtKey::new(None, JwtAlg::Rs256, vec![23, 4]));
    }

    fn check_serde<T: Serialize + PartialEq + Debug + for<'de> Deserialize<'de>>(value: &T) {
        assert_eq!(value,
                   &serde_json::from_str::<T>(&serde_json::to_string(value).unwrap()).unwrap());
    }

}
