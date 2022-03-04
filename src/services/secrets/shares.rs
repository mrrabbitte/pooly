use dashmap::DashSet;

use crate::models::secrets::MasterKeyShare;

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
               share: MasterKeyShare) {
        self.shares.insert(share);
    }

    pub fn add_all(&self,
                   shares: &Vec<MasterKeyShare>) {
        for share in shares {
            self.shares.insert(share.clone());
        }
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
    use crate::models::secrets::MasterKeyShare;
    use crate::services::secrets::shares::MasterKeySharesService;

    #[test]
    fn test_add_remove_get_clear() {
        let service = MasterKeySharesService::new();

        let (first, second, third) =
            (MasterKeyShare::new(vec![1, 1, 1]),
             MasterKeyShare::new(vec![2, 2, 2]),
             MasterKeyShare::new(vec![3, 3, 3]));


        service.add(first.clone());

        assert!(service.get().contains(&first));

        service.remove(&first);

        assert!(!service.get().contains(&first));

        service.add(first.clone());
        service.add(second.clone());
        service.add(third.clone());

        assert!(service.get().contains(&first));
        assert!(service.get().contains(&second));
        assert!(service.get().contains(&third));

        service.clear();

        assert!(service.get().is_empty());
    }

}