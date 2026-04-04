use super::{PathInfo, PathInfoService};
use crate::{pathinfoservice, proto};
use data_encoding::BASE64;
use futures::{StreamExt, TryStreamExt, stream::BoxStream};
use prost::Message;
use redb::{ReadableDatabase, ReadableTable, TableDefinition};
use snix_castore::composition::{CompositionContext, ServiceBuilder};
use std::{path::PathBuf, sync::Arc};
use tokio_stream::wrappers::ReceiverStream;
use tonic::async_trait;
use tracing::instrument;

const PATHINFO_TABLE: TableDefinition<[u8; 20], Vec<u8>> = TableDefinition::new("pathinfo");

enum Db {
    ReadOnly(redb::ReadOnlyDatabase),
    ReadWrite(redb::Database),
}

impl Db {
    fn begin_read(&self) -> Result<redb::ReadTransaction, redb::TransactionError> {
        match self {
            Db::ReadOnly(db) => db.begin_read(),
            Db::ReadWrite(db) => db.begin_read(),
        }
    }

    fn begin_write(&self) -> Result<redb::WriteTransaction, Error> {
        match self {
            Db::ReadOnly(_) => Err(Error::OpenedReadonly),
            Db::ReadWrite(db) => Ok(db.begin_write()?),
        }
    }
}

/// PathInfoService implementation using redb under the hood.
/// redb stores all of its data in a single file with a K/V pointing from a path's output hash to
/// its corresponding protobuf-encoded PathInfo.
#[derive(Clone)]
pub struct RedbPathInfoService {
    instance_name: String,

    /// An Arc'ed Database, read-only or writeable.
    db: Arc<Db>,
}

impl RedbPathInfoService {
    /// Constructs a new instance using the specified config.
    pub async fn new(
        instance_name: String,
        config: RedbPathInfoServiceConfig,
    ) -> Result<Self, Error> {
        if let Some(path) = config.path.clone() {
            if &path == "" {
                return Err(Error::WrongConfig("empty path is disallowed"));
            }
            if &path == "/" {
                return Err(Error::WrongConfig("cowardly refusing to open / with redb"));
            }

            if config.read_only {
                let db =
                    tokio::task::spawn_blocking(|| redb::Database::builder().open_read_only(path))
                        .await??;

                return Ok(Self {
                    instance_name,
                    db: Arc::new(Db::ReadOnly(db)),
                });
            }

            if let Some(parent) = path.parent() {
                tokio::fs::create_dir_all(parent).await?;
            }

            let db = tokio::task::spawn_blocking(move || {
                let mut builder = redb::Database::builder();
                configure_builder(&mut builder, &config);

                let db = builder.create(path)?;
                create_schema(&db)?;
                Ok::<_, Error>(db)
            })
            .await??;

            Ok(Self {
                instance_name,
                db: Arc::new(Db::ReadWrite(db)),
            })
        } else {
            Self::new_temporary(instance_name, config)
        }
    }

    /// Constructs a new instance using the in-memory backend.
    /// Sync, as there's no real IO happening.
    pub fn new_temporary(
        instance_name: String,
        config: RedbPathInfoServiceConfig,
    ) -> Result<Self, Error> {
        debug_assert!(
            config.path.is_none(),
            "Snix bug: config.path is not None, but new_temporary requested"
        );

        if config.read_only {
            return Err(Error::WrongConfig("in-memory database cannot be read-only"));
        }

        let mut builder = redb::Database::builder();
        configure_builder(&mut builder, &config);

        let db = builder.create_with_backend(redb::backends::InMemoryBackend::new())?;

        create_schema(&db)?;

        Ok(RedbPathInfoService {
            instance_name,
            db: Arc::new(Db::ReadWrite(db)),
        })
    }
}

/// Applies options from [RedbPathInfoServiceConfig] to a [redb::Builder].
fn configure_builder(builder: &mut redb::Builder, config: &RedbPathInfoServiceConfig) {
    if let Some(cache_size) = config.cache_size {
        builder.set_cache_size(cache_size);
    }
}

/// Ensures all tables are present.
/// Opens a write transaction and calls open_table on PATHINFO_TABLE, which will
/// create it if not present.
#[allow(clippy::result_large_err)]
fn create_schema(db: &redb::Database) -> Result<(), Error> {
    let txn = db.begin_write()?;
    txn.open_table(PATHINFO_TABLE)?;
    txn.commit()?;

    Ok(())
}

#[async_trait]
impl PathInfoService for RedbPathInfoService {
    #[instrument(level = "trace", skip_all, fields(path_info.digest = BASE64.encode(&digest), instance_name = %self.instance_name))]
    async fn get(&self, digest: [u8; 20]) -> Result<Option<PathInfo>, pathinfoservice::Error> {
        let db = self.db.clone();

        let path_info_bytes = match tokio::task::spawn_blocking({
            move || -> Result<_, Error> {
                let txn = db.begin_read()?;
                let table = txn.open_table(PATHINFO_TABLE)?;
                Ok(table.get(digest)?)
            }
        })
        .await??
        {
            // The PathInfo was not found, return None.
            None => return Ok(None),
            Some(path_info_data) => path_info_data.value(),
        };

        let pathinfo_proto =
            proto::PathInfo::decode(path_info_bytes.as_slice()).map_err(Error::ProtobufDecode)?;
        let path_info = PathInfo::try_from(pathinfo_proto).map_err(Error::PathInfoValidation)?;

        return Ok(Some(path_info));
    }

    #[instrument(level = "trace", skip_all, fields(path_info.root_node = ?path_info.node, instance_name = %self.instance_name))]
    async fn put(&self, path_info: PathInfo) -> Result<PathInfo, pathinfoservice::Error> {
        let db = self.db.clone();
        tokio::task::spawn_blocking({
            let path_info = path_info.clone();
            move || -> Result<(), Error> {
                let txn = db.begin_write()?;
                {
                    let mut table = txn.open_table(PATHINFO_TABLE)?;
                    table.insert(
                        *path_info.store_path.digest(),
                        proto::PathInfo::from(path_info).encode_to_vec(),
                    )?;
                }
                txn.commit()?;
                Ok(())
            }
        })
        .await??;

        Ok(path_info)
    }

    fn list(&self) -> BoxStream<'static, Result<PathInfo, pathinfoservice::Error>> {
        let db = self.db.clone();
        let (tx, rx) = tokio::sync::mpsc::channel(64);

        tokio::task::spawn_blocking(move || {
            // IIFE to be able to use ? for the error cases
            let result = (|| -> Result<(), Error> {
                let read_txn = db.begin_read()?;

                let table = read_txn.open_table(PATHINFO_TABLE)?;

                let table_iter = table.iter()?;

                for elem in table_iter {
                    let path_info_proto = proto::PathInfo::decode(elem?.1.value().as_slice())?;

                    let path_info = PathInfo::try_from(path_info_proto)?;

                    if tx.blocking_send(Ok(path_info)).is_err() {
                        break;
                    }
                }

                Ok(())
            })();

            if let Err(err) = result {
                let _ = tx.blocking_send(Err(err));
            }
        });

        ReceiverStream::new(rx).err_into().boxed()
    }
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("wrong arguments: {0}")]
    WrongConfig(&'static str),
    #[error("serde-qs error: {0}")]
    SerdeQS(#[from] serde_qs::Error),

    #[error("failed to decode protobuf: {0}")]
    ProtobufDecode(#[from] prost::DecodeError),
    #[error("failed to validate PathInfo: {0}")]
    PathInfoValidation(#[from] crate::proto::ValidatePathInfoError),

    #[error("unable to open write txn, database opened read-only")]
    OpenedReadonly,

    #[error("redb commit error: {0}")]
    RedbCommit(#[from] redb::CommitError),
    #[error("redb database error: {0}")]
    RedbDatabase(#[from] redb::DatabaseError),
    #[error("redb error: {0}")]
    Redb(#[from] redb::Error),
    #[error("redb storage error: {0}")]
    RedbStorage(#[from] redb::StorageError),
    #[error("redb table error: {0}")]
    RedbTable(#[from] redb::TableError),
    #[error("redb txn error: {0}")]
    RedbTransaction(#[from] redb::TransactionError),

    #[error("join error: {0}")]
    TokioJoin(#[from] tokio::task::JoinError),
    #[error("io error: {0}")]
    IO(#[from] std::io::Error),
}

#[derive(Clone, Default, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RedbPathInfoServiceConfig {
    path: Option<PathBuf>,

    /// The amount of memory (in bytes) used for caching data
    cache_size: Option<usize>,

    /// Whether to open read-only.
    #[serde(default)]
    read_only: bool,
}

impl TryFrom<url::Url> for RedbPathInfoServiceConfig {
    type Error = Box<dyn std::error::Error + Send + Sync>;
    fn try_from(url: url::Url) -> Result<Self, Self::Error> {
        if url.has_host() {
            return Err(Error::WrongConfig("no host allowed").into());
        }

        let path: Option<PathBuf> = match (url.scheme(), url.has_authority(), url.path()) {
            ("redb+memory", false, "") => None,
            ("redb+memory", false, _) => Err(Box::new(Error::WrongConfig(
                "redb+memory with path is disallowed",
            )))?,
            ("redb+memory", true, _) => Err(Box::new(Error::WrongConfig(
                "redb+memory may not have authority",
            )))?,
            ("redb", _, "") => Err(Box::new(Error::WrongConfig(
                "redb without path is disallowed, use redb+memory if you want in-memory",
            )))?,
            ("redb", true, _path) => Err(Box::new(Error::WrongConfig("authority disallowed")))?,
            ("redb", false, path) => Some(path.into()),
            (_scheme, _, _) => Err(Box::new(Error::WrongConfig("unrecognized scheme")))?,
        };

        let mut config: RedbPathInfoServiceConfig =
            serde_qs::from_str(url.query().unwrap_or_default())?;

        config.path = path;

        Ok(config)
    }
}

#[async_trait]
impl ServiceBuilder for RedbPathInfoServiceConfig {
    type Output = dyn PathInfoService;
    async fn build<'a>(
        &'a self,
        instance_name: &str,
        _context: &CompositionContext,
    ) -> Result<Arc<Self::Output>, Box<dyn std::error::Error + Send + Sync>> {
        Ok(Arc::new(
            RedbPathInfoService::new(instance_name.to_string(), self.to_owned()).await?,
        ))
    }
}
