use zeroize::Zeroize;

#[derive(Zeroize)]
#[zeroize(drop)]
#[derive(Clone)]
pub struct PublicKeyPem {

    value: Vec<u8>

}

impl PublicKeyPem {

    pub fn get_value(&self) -> &[u8] {
        &self.value
    }

}
