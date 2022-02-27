use zeroize::Zeroize;

#[derive(Zeroize)]
#[zeroize(drop)]
#[derive(Clone)]
pub struct ZeroizeWrapper {

    value: Vec<u8>

}

impl ZeroizeWrapper {

    pub fn new(value: Vec<u8>) -> ZeroizeWrapper {
        ZeroizeWrapper {
            value
        }
    }

    pub fn get_value(&self) -> &Vec<u8> {
        &self.value
    }

}