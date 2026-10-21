use alloc::collections::BTreeSet;
use alloc::string::String;
use alloc::vec::Vec;

use serde::Deserialize;
use serde::Serialize;

use crate::Digest;
use crate::Error;
use crate::Identity;
use crate::Status;
use crate::safe_key;

/// Recovery is a visible discontinuity, never proof of uninterrupted correctness.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Recovery {
    pub predecessor: Digest,
    pub successor: Digest,
    pub cause: String,
    pub original_status: Status,
    pub original_observation: Digest,
    pub discarded: Vec<Identity>,
    pub preserved: Vec<Identity>,
    pub teardown_complete: bool,
    pub continuity_satisfied: bool,
    pub remaining_obligations: Vec<String>,
}

impl Recovery {
    pub fn validate(&self) -> Result<(), Error> {
        if self.predecessor == self.successor
            || !safe_key(&self.cause)
            || self.original_status == Status::Success
            || self.continuity_satisfied
        {
            return Err(Error::Oracle);
        }
        if !self.teardown_complete {
            return Err(Error::IncompleteTeardown);
        }
        if self.discarded.len() > 64
            || self.preserved.len() > 64
            || self.remaining_obligations.len() > crate::MAX_MEMBERS
        {
            return Err(Error::Bound);
        }
        crate::manifest::validate_identities(&self.discarded)?;
        crate::manifest::validate_identities(&self.preserved)?;
        let mut roles = BTreeSet::new();
        for identity in self.discarded.iter().chain(&self.preserved) {
            if !roles.insert(&identity.role) {
                return Err(Error::Duplicate);
            }
        }
        let mut obligations = BTreeSet::new();
        for obligation in &self.remaining_obligations {
            if !safe_key(obligation) || !obligations.insert(obligation) {
                return Err(Error::Duplicate);
            }
        }
        Ok(())
    }
}
