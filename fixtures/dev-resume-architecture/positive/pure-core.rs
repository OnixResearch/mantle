#![no_std]
extern crate alloc;

use alloc::string::String;

pub struct ResumeFact {
    pub stage_id: String,
    pub digest_blake3: String,
}
