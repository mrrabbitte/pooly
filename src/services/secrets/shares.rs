use dashmap::DashSet;

use crate::models::sec::secrets::{MasterKeyShare, NUM_SHARES};

pub struct MasterKeySharesService {

    shares: DashSet<MasterKeyShare>

}

impl MasterKeySharesService {

    pub fn new() -> MasterKeySharesService {
        MasterKeySharesService {
            shares: DashSet::new()
        }
    }

    pub fn add(&self,
               share: MasterKeyShare) -> Result<(), ()> {
        if self.shares.len() >= (NUM_SHARES as usize) {
            return Err(());
        }

        self.shares.insert(share);

        Ok(())
    }

    pub fn add_all(&self,
                   shares: &Vec<MasterKeyShare>) -> Result<(), ()> {
        for share in shares {
            self.add(share.clone())?;
        }

        Ok(())
    }

    pub fn remove(&self,
                  share: &MasterKeyShare) {
        self.shares.remove(&share);
    }

    pub fn get(&self) -> &DashSet<MasterKeyShare> {
        &self.shares
    }

    pub fn clear(&self) {
        self.shares.clear();
    }

}


#[cfg(test)]
mod tests {
    use crate::models::sec::secrets::MasterKeyShare;
    use crate::services::secrets::shares::MasterKeySharesService;

    #[test]
    fn test_add_remove_get_clear() {
        let service = MasterKeySharesService::new();

        let (first, second, third) =
            (MasterKeyShare::new(vec![1, 1, 1]),
             MasterKeyShare::new(vec![2, 2, 2]),
             MasterKeyShare::new(vec![3, 3, 3]));


        service.add(first.clone()).unwrap();

        assert!(service.get().contains(&first));

        service.remove(&first);

        assert!(!service.get().contains(&first));

        service.add(first.clone()).unwrap();
        service.add(second.clone()).unwrap();
        service.add(third.clone()).unwrap();

        assert!(service.get().contains(&first));
        assert!(service.get().contains(&second));
        assert!(service.get().contains(&third));

        service.clear();

        assert!(service.get().is_empty());
    }

}