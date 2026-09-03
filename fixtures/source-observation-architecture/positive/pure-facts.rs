#![no_std]
extern crate alloc;

use alloc::string::String;

pub struct SuppliedSourceFacts {
    pub kind: String,
    pub immutable_revision: String,
    pub content_blake3: String,
}

pub fn accepts_explicit_facts(facts: &SuppliedSourceFacts) -> bool {
    !facts.kind.is_empty() && !facts.immutable_revision.is_empty() && !facts.content_blake3.is_empty()
}
