use crunch_project_core::Lockfile;
pub use crunch_project_core::OLDEST_SUPPORTED;

use crate::Error;

pub fn upgrade_lockfile(lock: Lockfile) -> Result<Lockfile, Error> {
    crunch_project_core::upgrade_lockfile(lock).map_err(Error::from)
}
