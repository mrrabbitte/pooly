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

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use ring::rand::SystemRandom;

    use crate::services::secrets::generate::VecGenerator;

    #[test]
    fn test_generates_random_vec_with_expected_size() {
        let generator = VecGenerator::new(Arc::new(SystemRandom::new()));

        assert!(generator.generate_random(0).unwrap().is_empty());
        assert_eq!(generator.generate_random(1).unwrap().len(), 1);
        assert_eq!(generator.generate_random(10).unwrap().len(), 10);
    }

}