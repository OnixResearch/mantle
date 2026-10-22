//! Bounded C/C++ records in the existing local castore cache.
use std::io::Read;

use serde::Deserialize;
use serde::Serialize;
use snix_castore::Node;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;

use super::Error;
use super::RustCache;

pub const MAX_CC_RECORDS: usize = 256;
pub const MAX_CC_OBJECT_BYTES: u64 = 8_388_608;
pub const MAX_CC_DIAGNOSTIC_BYTES: usize = 262_144;
pub const MAX_CC_DEPENDENCIES: usize = 4096;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CcDependency {
    pub locator: String,
    pub digest_blake3: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
pub enum CcDepfilePart {
    Literal { text: String },
    Root { alias: String },
    Output,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CcRecord {
    pub schema: String,
    pub base_key: String,
    pub key: String,
    pub dependencies: Vec<CcDependency>,
    pub complete: bool,
    pub status: i32,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
    pub object_digest: Option<String>,
    pub object_bytes: u64,
    pub object_mode: Option<u32>,
    pub depfile: Option<Vec<CcDepfilePart>>,
    pub depfile_mode: Option<u32>,
}

impl CcRecord {
    pub fn validate(&self) -> Result<(), Error> {
        let is_known_schema = self.schema == "mantle-cc-cache-record-v1";
        let has_valid_base_key = valid_digest(&self.base_key);
        let has_valid_key = valid_digest(&self.key);
        let is_complete = self.complete;
        let has_bounded_dependencies = !self.dependencies.is_empty() && self.dependencies.len() <= MAX_CC_DEPENDENCIES;
        let has_bounded_stdout = self.stdout.len() <= MAX_CC_DIAGNOSTIC_BYTES;
        let has_bounded_stderr = self.stderr.len() <= MAX_CC_DIAGNOSTIC_BYTES;
        let has_bounded_object = self.object_bytes <= MAX_CC_OBJECT_BYTES;
        let has_valid_status = (0..=255).contains(&self.status);
        let is_valid_record = is_known_schema
            && has_valid_base_key
            && has_valid_key
            && is_complete
            && has_bounded_dependencies
            && has_bounded_stdout
            && has_bounded_stderr
            && has_bounded_object
            && has_valid_status;
        if !is_valid_record {
            return Err(Error::State("cc-cache-record-invalid".into()));
        }
        if self.object_mode.is_some_and(|mode| mode > 0o777)
            || self.depfile_mode.is_some_and(|mode| mode > 0o777)
            || self.depfile.is_some() != self.depfile_mode.is_some()
        {
            return Err(Error::State("cc-cache-output-mode-invalid".into()));
        }
        if let Some(parts) = &self.depfile
            && (parts.len() > 8192
                || serde_json::to_vec(parts).map_err(|e| Error::Json(e.to_string()))?.len() > 2_097_152)
        {
            return Err(Error::Bound("cc-depfile-template-bound".into()));
        }
        for dependency in &self.dependencies {
            let has_usable_locator = !dependency.locator.is_empty()
                && dependency.locator.len() <= 4096
                && !dependency.locator.starts_with('/')
                && !dependency.locator.split('/').any(|part| part == "..");
            let has_valid_digest = valid_digest(&dependency.digest_blake3);
            if !has_usable_locator || !has_valid_digest {
                return Err(Error::State("cc-cache-dependency-invalid".into()));
            }
        }
        let has_failure_object = self.object_digest.is_some()
            || self.object_bytes != 0
            || self.object_mode.is_some()
            || self.depfile.is_some();
        if self.status == 0 {
            let has_object = self.object_digest.as_deref().is_some_and(valid_digest) && self.object_mode.is_some();
            if !has_object {
                return Err(Error::State("cc-cache-object-missing".into()));
            }
        } else if has_failure_object {
            return Err(Error::State("cc-cache-failure-object-present".into()));
        }
        Ok(())
    }
}

pub fn valid_digest(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

impl RustCache {
    pub fn cc_lookup(&self, base_key: &str) -> Result<Option<CcRecord>, Error> {
        if !valid_digest(base_key) {
            return Err(Error::State("cc-base-key-invalid".into()));
        }
        let path = self.cache_dir.join("cc-v1").join(format!("{base_key}.json"));
        let Some(bytes) = super::read_bounded_optional(&path)? else {
            return Ok(None);
        };
        let record: CcRecord = serde_json::from_slice(&bytes).map_err(|e| Error::Json(e.to_string()))?;
        record.validate()?;
        if record.base_key != base_key {
            return Err(Error::State("cc-cache-key-mismatch".into()));
        }
        Ok(Some(record))
    }

    pub fn cc_publish(&self, record: &CcRecord, object: Option<&[u8]>) -> Result<(), Error> {
        record.validate()?;
        let _guard =
            crunch_store::StoreMutationGuard::acquire_wait(&self.state_dir).map_err(|e| Error::State(e.to_string()))?;
        let _lock = super::CacheMutationLock::acquire(&self.cache_dir.join(super::MUTATION_LOCK_FILE))?;
        let directory = self.cache_dir.join("cc-v1");
        super::create_private_directory(&directory)?;
        let path = directory.join(format!("{}.json", record.base_key));
        if !path.exists() && std::fs::read_dir(&directory).map_err(io)?.take(MAX_CC_RECORDS).count() >= MAX_CC_RECORDS {
            return Err(Error::Bound("cc-cache-record-limit".into()));
        }
        let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build().map_err(io)?;
        runtime.block_on(async {
            if let Some(bytes) = object {
                if bytes.len() as u64 != record.object_bytes
                    || Some(blake3::hash(bytes).to_hex().as_str()) != record.object_digest.as_deref()
                {
                    return Err(Error::State("cc-cache-object-mismatch".into()));
                }
                let mut writer = self.blob_service.open_write().await;
                writer.write_all(bytes).await.map_err(io)?;
                let digest = writer.close().await.map_err(io)?;
                let node = Node::File {
                    digest,
                    size: record.object_bytes,
                    executable: false,
                };
                if let Some(store) = &self.durable_store {
                    store.admit_castore_payload_root(&node).await.map_err(Error::Authority)?;
                }
            } else if record.object_digest.is_some() {
                return Err(Error::State("cc-cache-object-not-supplied".into()));
            }
            Ok(())
        })?;
        super::write_atomic_json(&path, record)
    }

    pub fn cc_object(&self, record: &CcRecord) -> Result<Option<Vec<u8>>, Error> {
        record.validate()?;
        let Some(encoded) = &record.object_digest else {
            return Ok(None);
        };
        let digest = super::digest_from_hex(encoded)?;
        let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build().map_err(io)?;
        runtime.block_on(async {
            let node = Node::File {
                digest,
                size: record.object_bytes,
                executable: false,
            };
            if let Some(store) = &self.durable_store {
                store.rehydrate_castore_payload_root(&node).await.map_err(Error::Authority)?;
            }
            let Some(reader) = self.blob_service.open_read(&digest).await.map_err(io)? else {
                return Ok(None);
            };
            let mut bytes = Vec::new();
            reader.take(MAX_CC_OBJECT_BYTES.saturating_add(1)).read_to_end(&mut bytes).await.map_err(io)?;
            if bytes.len() as u64 != record.object_bytes || blake3::hash(&bytes).to_hex().as_str() != encoded {
                return Err(Error::State("cc-cache-object-corrupt".into()));
            }
            Ok(Some(bytes))
        })
    }

    pub(super) fn cc_live_nodes(&self) -> Result<Vec<Node>, Error> {
        let directory = self.cache_dir.join("cc-v1");
        let entries = match std::fs::read_dir(directory) {
            Ok(entries) => entries,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(e) => return Err(io(e)),
        };
        let mut nodes = Vec::with_capacity(MAX_CC_RECORDS);
        for (index, entry) in entries.enumerate() {
            if index >= MAX_CC_RECORDS {
                return Err(Error::Bound("cc-cache-record-limit".into()));
            }
            let entry = entry.map_err(io)?;
            let mut file = super::open_read_nofollow(&entry.path())?;
            let mut bytes = Vec::new();
            Read::by_ref(&mut file)
                .take(super::MAX_CACHE_JSON_BYTES.saturating_add(1))
                .read_to_end(&mut bytes)
                .map_err(io)?;
            if bytes.len() as u64 > super::MAX_CACHE_JSON_BYTES {
                return Err(Error::Bound("cc-record-bytes".into()));
            }
            let record: CcRecord = serde_json::from_slice(&bytes).map_err(|e| Error::Json(e.to_string()))?;
            record.validate()?;
            if let Some(encoded) = &record.object_digest {
                nodes.push(Node::File {
                    digest: super::digest_from_hex(encoded)?,
                    size: record.object_bytes,
                    executable: false,
                });
            }
        }
        Ok(nodes)
    }
}

fn io(source: std::io::Error) -> Error {
    Error::Io {
        context: "cc-cache".into(),
        source,
    }
}
