use std::sync::Arc;
use ring::rand::{SecureRandom, SystemRandom};
use crate::models::errors::SecretsError;

pub struct VecGenerator {

    sys_random: Arc<SystemRandom>

}

impl VecGenerator {

    pub fn new(sys_random: Arc<SystemRandom>) -> VecGenerator {
        VecGenerator {
            sys_random
        }
    }

    pub fn generate_random(&self,
                       size: usize) -> Result<Vec<u8>, SecretsError> {
        let mut value = vec![0; size];

        self.sys_random.fill(&mut value)?;

        Ok(value)
    }

}