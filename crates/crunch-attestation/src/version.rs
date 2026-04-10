use serde::Deserialize;
use serde::Serialize;

use crate::Error;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(try_from = "u32", into = "u32")]
pub struct SchemaVersion(u32);

impl SchemaVersion {
    pub const V1: Self = Self(1);

    pub fn new(value: u32) -> Result<Self, Error> {
        if value == 0 {
            return Err(Error::EmptyField {
                field: "schema_version",
            });
        }

        Ok(Self(value))
    }

    pub fn get(self) -> u32 {
        self.0
    }
}

impl Default for SchemaVersion {
    fn default() -> Self {
        Self::V1
    }
}

impl TryFrom<u32> for SchemaVersion {
    type Error = Error;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl From<SchemaVersion> for u32 {
    fn from(value: SchemaVersion) -> Self {
        value.get()
    }
}
