#![no_std]

extern crate alloc;

mod identity;
mod model;
mod planning;
mod validation;

pub use identity::compute_resume_bundle_identity;
pub use identity::compute_resume_policy_identity;
pub use identity::seal_resume_bundle;
pub use model::*;
pub use planning::plan_resume;
pub use validation::validate_resume_candidate;
pub use validation::validate_resume_manifest;

#[cfg(test)]
extern crate std;

#[cfg(test)]
mod tests;
