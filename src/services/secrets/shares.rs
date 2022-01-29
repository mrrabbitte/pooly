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

    pub fn get(&self) -> &DashSet<MasterKeyShare> {
        &self.shares
    }

    pub fn clear(&self) {
        self.shares.clear();
    }

}