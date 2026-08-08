//! Import from a real filesystem.

use std::fs::FileType;
use std::os::unix::ffi::OsStringExt;
use std::os::unix::fs::MetadataExt;
use std::os::unix::fs::PermissionsExt;

use futures::StreamExt;
use futures::stream::BoxStream;
use tokio::io::BufReader;
use tokio_util::io::InspectReader;
use tracing::info_span;
use tracing::instrument;
use tracing_indicatif::span_ext::IndicatifSpanExt;
use walkdir::DirEntry;
use walkdir::WalkDir;

use super::IngestionEntry;
use super::IngestionError;
use super::ingest_entries;
use crate::B3Digest;
use crate::Node;
use crate::blobservice::BlobService;
use crate::directoryservice::DirectoryService;
use crate::refscan::ReferenceReader;
use crate::refscan::ReferenceScanner;

/// Ingests the contents at a given path into the snix store, interacting with a [BlobService] and
/// [DirectoryService]. It returns the root node or an error.
///
/// It does not follow symlinks at the root, they will be ingested as actual symlinks.
///
/// This function will walk the filesystem using `walkdir` and will consume
/// `O(#number of entries)` space.
#[instrument(skip(blob_service, directory_service, reference_scanner), fields(path), err)]
pub async fn ingest_path<BS, DS, P, P2>(
    blob_service: BS,
    directory_service: DS,
    path: P,
    reference_scanner: Option<&ReferenceScanner<P2>>,
) -> Result<Node, IngestionError<Error>>
where
    P: AsRef<std::path::Path>,
    BS: BlobService + Clone,
    DS: DirectoryService,
    P2: AsRef<[u8]> + Send + Sync,
{
    let iter = WalkDir::new(path.as_ref())
        .follow_links(false)
        .follow_root_links(false)
        .contents_first(true)
        .into_iter();

    ingest_entries(
        directory_service,
        dir_entries_to_ingestion_stream(blob_service, iter, path.as_ref(), reference_scanner),
    )
    .await
}

/// Converts an iterator of [walkdir::DirEntry]s into a stream of ingestion entries.
/// This can then be fed into [ingest_entries] to ingest all the entries into the castore.
///
/// The produced stream is buffered, so uploads can happen concurrently.
///
/// The root is the [std::path::Path] in the filesystem that is being ingested
/// into castore.
pub fn dir_entries_to_ingestion_stream<'a, BS, I, P>(
    blob_service: BS,
    walkdir_direntries: I,
    root: &'a std::path::Path,
    reference_scanner: Option<&'a ReferenceScanner<P>>,
) -> BoxStream<'a, Result<IngestionEntry, Error>>
where
    BS: BlobService + Clone + 'a,
    I: Iterator<Item = Result<DirEntry, walkdir::Error>> + Send + 'a,
    P: AsRef<[u8]> + Send + Sync,
{
    let prefix = root.parent().unwrap_or_else(|| std::path::Path::new(""));

    Box::pin(
        futures::stream::iter(walkdir_direntries)
            .map(move |x| {
                let blob_service = blob_service.clone();
                async move {
                    match x {
                        Ok(dir_entry) => {
                            dir_entry_to_ingestion_entry(blob_service, &dir_entry, prefix, reference_scanner).await
                        }
                        Err(e) => {
                            Err(Error::Stat(prefix.to_path_buf(), e.into_io_error().expect("walkdir err must be some")))
                        }
                    }
                }
            })
            .buffered(50),
    )
}

/// Converts a [walkdir::DirEntry] into an [IngestionEntry], uploading blobs to the
/// provided [BlobService].
///
/// The prefix path is stripped from the path of each entry. This is usually the parent path
/// of the path being ingested so that the last element of the stream only has one component.
pub async fn dir_entry_to_ingestion_entry<BS, P>(
    blob_service: BS,
    walkdir_direntry: &DirEntry,
    prefix: &std::path::Path,
    reference_scanner: Option<&ReferenceScanner<P>>,
) -> Result<IngestionEntry, Error>
where
    BS: BlobService,
    P: AsRef<[u8]>,
{
    let file_type = walkdir_direntry.file_type();

    let fs_path = walkdir_direntry.path().strip_prefix(prefix).expect("Snix bug: failed to strip root path prefix");

    // convert to castore PathBuf
    let path = crate::path::PathBuf::from_host_path(fs_path, false)
        .unwrap_or_else(|e| panic!("Snix bug: walkdir direntry cannot be parsed: {e}"));

    if file_type.is_dir() {
        Ok(IngestionEntry::Dir { path })
    } else if file_type.is_symlink() {
        let target = std::fs::read_link(walkdir_direntry.path())
            .map_err(|e| Error::Stat(walkdir_direntry.path().to_path_buf(), e))?
            .into_os_string()
            .into_vec();

        if let Some(reference_scanner) = &reference_scanner {
            reference_scanner.scan(&target);
        }

        Ok(IngestionEntry::Symlink { path, target })
    } else if file_type.is_file() {
        let metadata = walkdir_direntry
            .metadata()
            .map_err(|e| Error::Stat(walkdir_direntry.path().to_path_buf(), e.into()))?;

        let expected_size = metadata.size();
        let digest = upload_blob(blob_service, walkdir_direntry.path(), expected_size, reference_scanner).await?;

        Ok(IngestionEntry::Regular {
            path,
            size: expected_size,
            // If it's executable by the user, it'll become executable.
            // This matches nix's dump() function behaviour.
            executable: metadata.permissions().mode() & 64 != 0,
            digest,
        })
    } else {
        Err(Error::FileType(fs_path.to_path_buf(), file_type))
    }
}

/// Uploads the file at the provided [std::path::Path] to the [BlobService].
#[instrument(skip_all, fields(path), err)]
async fn upload_blob<BS, P>(
    blob_service: BS,
    path: impl AsRef<std::path::Path>,
    expected_size: u64,
    reference_scanner: Option<&ReferenceScanner<P>>,
) -> Result<B3Digest, Error>
where
    BS: BlobService,
    P: AsRef<[u8]>,
{
    let progress_span = info_span!("upload_blobs", "indicatif.pb_show" = tracing::field::Empty);
    progress_span.pb_set_style(&snix_tracing::PB_TRANSFER_STYLE);
    progress_span.pb_start();
    progress_span.pb_set_message(&format!("Uploading blob at {:?}", path.as_ref()));

    let file = tokio::fs::File::open(path.as_ref())
        .await
        .map_err(|e| Error::BlobRead(path.as_ref().to_path_buf(), e))?;

    progress_span.pb_set_length(expected_size);
    let reader = InspectReader::new(file, |d| {
        progress_span.pb_inc(d.len() as u64);
    });

    let mut writer = blob_service.open_write().await;
    // r[impl vendored_snix.operational_alignment]
    let copied_size = if let Some(reference_scanner) = reference_scanner {
        let mut reader = ReferenceReader::new(reference_scanner, BufReader::new(reader));
        tokio::io::copy_buf(&mut reader, &mut writer)
            .await
            .map_err(|e| Error::BlobRead(path.as_ref().to_path_buf(), e))?
    } else {
        tokio::io::copy_buf(&mut BufReader::new(reader), &mut writer)
            .await
            .map_err(|e| Error::BlobRead(path.as_ref().to_path_buf(), e))?
    };

    let digest = writer.close().await.map_err(|e| Error::BlobFinalize(path.as_ref().to_path_buf(), e))?;
    if copied_size != expected_size {
        return Err(Error::UnexpectedSize {
            path: path.as_ref().to_path_buf(),
            wanted: expected_size,
            got: copied_size,
        });
    }

    Ok(digest)
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("unsupported file type at {0}: {1:?}")]
    FileType(std::path::PathBuf, FileType),

    #[error("unable to stat {0}: {1}")]
    Stat(std::path::PathBuf, std::io::Error),

    #[error("unable to open {0}: {1}")]
    Open(std::path::PathBuf, std::io::Error),

    #[error("unable to read {0}: {1}")]
    BlobRead(std::path::PathBuf, std::io::Error),

    #[error("read unexpected size at {path}: wanted {wanted}, got {got}")]
    UnexpectedSize {
        path: std::path::PathBuf,
        wanted: u64,
        got: u64,
    },

    // TODO: proper error for blob finalize
    #[error("unable to finalize blob {0}: {1}")]
    BlobFinalize(std::path::PathBuf, std::io::Error),
}

#[cfg(test)]
mod tests {
    use std::pin::Pin;
    use std::task::Context;
    use std::task::Poll;

    use async_trait::async_trait;
    use tokio::io::AsyncWrite;

    use super::*;
    use crate::blobservice::BlobReader;
    use crate::blobservice::BlobWriter;
    use crate::blobservice::MemoryBlobService;

    struct CopyErrorWriter;

    impl AsyncWrite for CopyErrorWriter {
        fn poll_write(self: Pin<&mut Self>, _cx: &mut Context<'_>, _buf: &[u8]) -> Poll<std::io::Result<usize>> {
            Poll::Ready(Err(std::io::Error::other("injected copy failure")))
        }

        fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
            Poll::Ready(Ok(()))
        }

        fn poll_shutdown(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
            Poll::Ready(Ok(()))
        }
    }

    #[async_trait]
    impl BlobWriter for CopyErrorWriter {
        async fn close(&mut self) -> std::io::Result<B3Digest> {
            Err(std::io::Error::other("copy failure must stop before close"))
        }
    }

    #[derive(Clone)]
    struct CopyErrorBlobService;

    #[async_trait]
    impl BlobService for CopyErrorBlobService {
        async fn has(&self, _digest: &B3Digest) -> std::io::Result<bool> {
            Ok(false)
        }

        async fn open_read(&self, _digest: &B3Digest) -> std::io::Result<Option<Box<dyn BlobReader>>> {
            Ok(None)
        }

        async fn open_write(&self) -> Box<dyn BlobWriter> {
            Box::new(CopyErrorWriter)
        }
    }

    #[tokio::test]
    async fn upload_blob_copies_normal_and_empty_files() {
        // r[verify vendored_snix.operational_alignment]
        let tmp = tempfile::tempdir().unwrap();
        let normal_path = tmp.path().join("normal");
        let empty_path = tmp.path().join("empty");
        let normal_bytes = b"normal file";
        std::fs::write(&normal_path, normal_bytes).unwrap();
        std::fs::write(&empty_path, b"").unwrap();
        let blob_service = MemoryBlobService::default();

        let normal_digest =
            upload_blob::<_, &[u8]>(blob_service.clone(), &normal_path, normal_bytes.len().try_into().unwrap(), None)
                .await
                .unwrap();
        let empty_digest = upload_blob::<_, &[u8]>(blob_service.clone(), &empty_path, 0, None).await.unwrap();

        assert_eq!(normal_digest, blake3::hash(normal_bytes).into());
        assert_eq!(empty_digest, blake3::hash(&[]).into());
        assert!(blob_service.has(&normal_digest).await.unwrap());
        assert!(blob_service.has(&empty_digest).await.unwrap());
    }

    #[tokio::test]
    async fn upload_blob_propagates_copy_error_before_finalize() {
        // r[verify vendored_snix.operational_alignment]
        const DATA_SIZE_BYTES: u64 = 4;
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("copy-error");
        std::fs::write(&path, b"data").unwrap();

        let error = upload_blob::<_, &[u8]>(CopyErrorBlobService, &path, DATA_SIZE_BYTES, None).await.unwrap_err();

        assert!(matches!(error, Error::BlobRead(_, _)));
        assert!(!matches!(error, Error::BlobFinalize(_, _)));
    }

    #[tokio::test]
    async fn upload_blob_rejects_short_read_against_expected_size() {
        const EXPECTED_SIZE: u64 = 5;
        const ACTUAL_SIZE: u64 = 4;

        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("shrunk");
        std::fs::write(&path, b"shrt").unwrap();
        let blob_service = MemoryBlobService::default();

        let err = upload_blob::<_, &[u8]>(blob_service, &path, EXPECTED_SIZE, None).await.unwrap_err();

        assert!(matches!(err, Error::UnexpectedSize {
            wanted: EXPECTED_SIZE,
            got: ACTUAL_SIZE,
            ..
        }));
    }
}
