use std::collections::HashSet;
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

    pub fn remove(&self,
                  share: MasterKeyShare) {
        self.shares.remove(&share);
    }

    pub fn get_copy(&self) -> HashSet<MasterKeyShare> {
        self.shares.iter().map(|key_share| key_share.clone()).collect()
    }

    pub fn clear(&self) {
        self.shares.clear();
    }

}