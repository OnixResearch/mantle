use std::sync::Arc;

use async_trait::async_trait;
use futures::TryStreamExt;
use futures::stream::BoxStream;
use nix_compat::narinfo::NarInfo;
use nix_compat::narinfo::Signature;
use nix_compat::narinfo::{self};
use nix_compat::nixbase32;
use nix_compat::nixhash::NixHash;
use nix_compat::store_path::StorePath;
use reqwest::StatusCode;
use snix_castore::blobservice::BlobService;
use snix_castore::composition::CompositionContext;
use snix_castore::composition::ServiceBuilder;
use snix_castore::directoryservice::DirectoryService;
use tokio::io::AsyncBufRead;
use tokio::io::AsyncRead;
use tokio::io::{self};
use tracing::Span;
use tracing::instrument;
use tracing::warn;
use url::Url;

use super::PathInfo;
use super::PathInfoService;
use crate::nar::NarIngestionError;
use crate::nar::ingest_nar_and_hash;
use crate::pathinfoservice;

/// NixHTTPPathInfoService acts as a bridge in between the Nix HTTP Binary cache
/// protocol provided by Nix binary caches such as cache.nixos.org, and the Snix
/// Store Model.
/// It implements the [PathInfoService] trait in an interesting way:
/// Every [PathInfoService::get] fetches the .narinfo and referred NAR file,
/// inserting components into a [BlobService] and [DirectoryService], then
/// returning a [PathInfo] struct with the root.
///
/// Due to this being quite a costly operation, clients are expected to layer
/// this service with store composition, so they're only ingested once.
///
/// The client is expected to be (indirectly) using the same [BlobService] and
/// [DirectoryService], so able to fetch referred Directories and Blobs.
/// [PathInfoService::put] is not implemented and returns an error if called.
/// TODO: what about reading from nix-cache-info?
pub struct NixHTTPPathInfoService<BS, DS> {
    instance_name: String,
    base_url: url::Url,
    store_dir: String,
    http_client: reqwest_middleware::ClientWithMiddleware,

    blob_service: BS,
    directory_service: DS,

    /// An optional list of [narinfo::VerifyingKey].
    /// If the list is not empty, the .narinfo files received need to have
    /// correct signature by at least one of these.
    trusted_public_keys: Vec<narinfo::VerifyingKey>,
}

impl<BS, DS> NixHTTPPathInfoService<BS, DS> {
    pub fn try_build(
        instance_name: String,
        config: NixHTTPPathInfoServiceConfig,
        blob_service: BS,
        directory_service: DS,
    ) -> Result<Self, Error> {
        validate_store_dir(&config.params.store_dir)?;
        let mut trusted_public_keys = Vec::new();
        for s in config.params.trusted_public_keys {
            trusted_public_keys.push(narinfo::VerifyingKey::parse(&s).map_err(|e| Error::ParseTrustedPublicKey(s, e))?)
        }

        Ok(Self {
            instance_name,
            base_url: normalize_binary_cache_base_url(config.base_url),
            store_dir: config.params.store_dir,
            http_client: reqwest_middleware::ClientBuilder::new(
                reqwest::Client::builder()
                    .user_agent(crate::USER_AGENT)
                    .build()
                    .map_err(reqwest_middleware::Error::Reqwest)?,
            )
            .with(snix_tracing::propagate::reqwest::tracing_middleware())
            .build(),
            blob_service,
            directory_service,

            trusted_public_keys,
        })
    }

    #[instrument(level=tracing::Level::TRACE, skip_all,fields(path.digest=nixbase32::encode(&digest)),err)]
    fn derive_narinfo_url(&self, digest: [u8; 20]) -> Result<Url, Error> {
        let s = format!("{}.narinfo", nixbase32::encode(&digest));
        self.base_url.join(&s).map_err(|e| Error::JoinUrl(self.base_url.to_owned(), s.to_owned(), e))
    }

    async fn fetch_narinfo_text(&self, digest: [u8; 20]) -> Result<Option<String>, Error> {
        let narinfo_url = self.derive_narinfo_url(digest)?;
        let span = Span::current();
        span.record("narinfo.url", narinfo_url.to_string());

        let resp = self.http_client.get(narinfo_url).send().await.map_err(Error::Reqwest)?;
        if resp.status() == StatusCode::NOT_FOUND || resp.status() == StatusCode::FORBIDDEN {
            return Ok(None);
        }

        let narinfo_str = resp.text().await.map_err(Error::DecodeBody)?;
        Ok(Some(narinfo_str))
    }

    fn parse_and_verify_narinfo<'a>(&self, narinfo_str: &'a str) -> Result<NarInfo<'a>, Error> {
        let narinfo = NarInfo::parse_with_store_dir(narinfo_str, &self.store_dir).map_err(Error::ParseNARInfo)?;

        if !self.trusted_public_keys.is_empty() {
            let fingerprint = narinfo.fingerprint();
            let has_valid_signature = self
                .trusted_public_keys
                .iter()
                .any(|pubkey| narinfo.signatures.iter().any(|sig| pubkey.verify(&fingerprint, sig)));
            if !has_valid_signature {
                return Err(Error::NoValidSignature);
            }
        }

        Ok(narinfo)
    }
}

// r[impl cache_substitution.transport_normalization]
fn normalize_binary_cache_base_url(mut base_url: Url) -> Url {
    assert!(base_url.has_host(), "binary cache base URL must have a host");
    assert!(!base_url.cannot_be_a_base(), "binary cache URL must support relative joins");
    if !base_url.path().ends_with('/') {
        let mut path = base_url.path().to_string();
        path.push('/');
        base_url.set_path(&path);
    }
    base_url
}

// r[impl cache_substitution.transport_normalization]
fn multi_frame_zstd_decoder<R>(reader: R) -> async_compression::tokio::bufread::ZstdDecoder<R>
where R: AsyncBufRead {
    let mut decoder = async_compression::tokio::bufread::ZstdDecoder::new(reader);
    decoder.multiple_members(true);
    decoder
}

fn validate_requested_store_path_digest(requested_digest: [u8; 20], observed_digest: [u8; 20]) -> Result<(), Error> {
    if requested_digest != observed_digest {
        return Err(Error::StorePathDigestMismatch {
            requested_digest,
            observed_digest,
        });
    }
    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("wrong arguments: {0}")]
    WrongConfig(&'static str),
    #[error("serde-qs error: {0}")]
    SerdeQS(#[from] serde_qs::Error),
    #[error("unable to parse pubkey {0}")]
    ParseTrustedPublicKey(String, nix_compat::narinfo::VerifyingKeyError),

    #[error("unable to join URL {0} with {1}")]
    JoinUrl(Url, String, url::ParseError),
    #[error("reqwest error")]
    Reqwest(#[from] reqwest_middleware::Error),
    #[error("unable to decode NARInfo response as string")]
    DecodeBody(reqwest::Error),
    #[error("unable to parse NARInfo")]
    ParseNARInfo(nix_compat::narinfo::Error),
    #[error("no valid signature found")]
    NoValidSignature,
    #[error("narinfo store path digest does not match the requested digest")]
    StorePathDigestMismatch {
        requested_digest: [u8; 20],
        observed_digest: [u8; 20],
    },
    #[error("failed to request NAR, status {0}")]
    FailedToRequestNAR(reqwest::StatusCode),
    #[error("unsupported NAR compression: {0}")]
    UnsupportedNARCompression(String),
    #[error("failed to ingest NAR")]
    IngestNAR(NarIngestionError),
    #[error("NARSize mismatch, narinfo size {narinfo_size}, actual size {actual_size}")]
    NARSizeMismatch { narinfo_size: u64, actual_size: u64 },
    #[error("NARHash mismatch, narinfo NARHash {exp}, actual NARHash {act}",
        exp = NixHash::Sha256(*.narinfo_nar_sha256),
        act = NixHash::Sha256(*.actual_nar_sha256))]
    NARHashMismatch {
        narinfo_nar_sha256: [u8; 32],
        actual_nar_sha256: [u8; 32],
    },

    #[error("put not supported")]
    PutNotSupported,
    #[error("list not supported")]
    ListNotSupported,
}

#[async_trait]
impl<BS, DS> PathInfoService for NixHTTPPathInfoService<BS, DS>
where
    BS: BlobService + Send + Sync + Clone + 'static,
    DS: DirectoryService + Send + Sync + Clone + 'static,
{
    #[instrument(skip_all, err, fields(path.digest=nixbase32::encode(&digest), instance_name=%self.instance_name))]
    async fn get(&self, digest: [u8; 20]) -> Result<Option<PathInfo>, pathinfoservice::Error> {
        let Some(narinfo_str) = self.fetch_narinfo_text(digest).await? else {
            return Ok(None);
        };
        let narinfo = self.parse_and_verify_narinfo(&narinfo_str)?;
        validate_requested_store_path_digest(digest, *narinfo.store_path.digest())?;
        let span = Span::current();

        // To construct the full PathInfo, we also need to populate the node field,
        // and for this we need to download the NAR file and ingest it into castore.
        // FUTUREWORK: Keep some database around mapping from narsha256 to
        // (unnamed) rootnode, so we can use that and avoid downloading the same
        // NAR a second time.

        // create a request for the NAR file itself.
        let nar_url = self
            .base_url
            .join(narinfo.url)
            .map_err(|e| Error::JoinUrl(self.base_url.clone(), narinfo.url.to_owned(), e))?;
        span.record("nar.url", nar_url.to_string());

        let resp = self.http_client.get(nar_url.clone()).send().await.map_err(Error::Reqwest)?;

        // if the request is not successful, return an error.
        if !resp.status().is_success() {
            Err(Error::FailedToRequestNAR(resp.status()))?;
        }

        // get a reader of the response body.
        let r = tokio_util::io::StreamReader::new(resp.bytes_stream().map_err(|e| {
            let e = e.without_url();
            warn!(e=%e, "failed to get response body");
            io::Error::new(io::ErrorKind::BrokenPipe, e.to_string())
        }));

        // handle decompression, depending on the compression field.
        let mut r: Box<dyn AsyncRead + Send + Unpin> = match narinfo.compression {
            None => Box::new(r) as Box<dyn AsyncRead + Send + Unpin>,
            Some("bzip2") => {
                Box::new(async_compression::tokio::bufread::BzDecoder::new(r)) as Box<dyn AsyncRead + Send + Unpin>
            }
            Some("gzip") => {
                Box::new(async_compression::tokio::bufread::GzipDecoder::new(r)) as Box<dyn AsyncRead + Send + Unpin>
            }
            Some("xz") => {
                Box::new(async_compression::tokio::bufread::XzDecoder::new(r)) as Box<dyn AsyncRead + Send + Unpin>
            }
            Some("zstd") => Box::new(multi_frame_zstd_decoder(r)) as Box<dyn AsyncRead + Send + Unpin>,
            Some(comp_str) => Err(Error::UnsupportedNARCompression(comp_str.to_owned()))?,
        };

        let (root_node, nar_hash, nar_size) =
            ingest_nar_and_hash(self.blob_service.clone(), &self.directory_service, &mut r, &narinfo.ca)
                .await
                .map_err(Error::IngestNAR)?;

        // ensure the ingested narhash and narsize do actually match.
        if narinfo.nar_size != nar_size {
            Err(Error::NARSizeMismatch {
                narinfo_size: narinfo.nar_size,
                actual_size: nar_size,
            })?
        }
        if narinfo.nar_hash != nar_hash {
            Err(Error::NARHashMismatch {
                narinfo_nar_sha256: narinfo.nar_hash,
                actual_nar_sha256: nar_hash,
            })?
        }

        Ok(Some(PathInfo {
            store_path: narinfo.store_path.to_owned(),
            node: root_node,
            references: narinfo.references.iter().map(StorePath::to_owned).collect(),
            nar_size: narinfo.nar_size,
            nar_sha256: narinfo.nar_hash,
            deriver: narinfo.deriver.as_ref().map(StorePath::to_owned),
            signatures: narinfo
                .signatures
                .into_iter()
                .map(|s| Signature::<String>::new(s.name().to_string(), s.bytes().to_owned()))
                .collect(),
            ca: narinfo.ca,
        }))
    }

    #[instrument(skip_all, err, fields(path.digest=nixbase32::encode(&digest), instance_name=%self.instance_name))]
    async fn get_references(&self, digest: [u8; 20]) -> Result<Option<Vec<StorePath<String>>>, pathinfoservice::Error> {
        let Some(narinfo_str) = self.fetch_narinfo_text(digest).await? else {
            return Ok(None);
        };
        let narinfo = self.parse_and_verify_narinfo(&narinfo_str)?;
        validate_requested_store_path_digest(digest, *narinfo.store_path.digest())?;
        Ok(Some(narinfo.references.iter().map(StorePath::to_owned).collect()))
    }

    #[instrument(skip_all, err, fields(path.digest=nixbase32::encode(&digest), instance_name=%self.instance_name))]
    async fn has(&self, digest: [u8; 20]) -> Result<bool, pathinfoservice::Error> {
        let narinfo_url = self.derive_narinfo_url(digest)?;

        let span = Span::current();
        span.record("narinfo.url", narinfo_url.to_string());

        let resp = self.http_client.head(narinfo_url).send().await.map_err(Error::Reqwest)?;

        // In the case of a 404, return a NotFound.
        // We also return a NotFound in case of a 403 - this is to match the behaviour as Nix,
        // when querying nix-cache.s3.amazonaws.com directly, rather than cache.nixos.org.
        if resp.status() == StatusCode::NOT_FOUND || resp.status() == StatusCode::FORBIDDEN {
            Ok(false)
        } else {
            Ok(true)
        }
    }

    #[instrument(skip_all, fields(path_info=?_path_info, instance_name=%self.instance_name))]
    async fn put(&self, _path_info: PathInfo) -> Result<PathInfo, pathinfoservice::Error> {
        Err(Box::new(Error::PutNotSupported))
    }

    fn list(&self) -> BoxStream<'static, Result<PathInfo, pathinfoservice::Error>> {
        Box::pin(futures::stream::once(async { Err(Error::ListNotSupported)? }))
    }
}

#[derive(serde::Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct NixHTTPPathInfoServiceConfig {
    base_url: Url,

    #[serde(flatten)]
    params: NixHTTPPathInfoServiceParams,
}

#[derive(serde::Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct NixHTTPPathInfoServiceParams {
    #[serde(default = "default_store_dir")]
    store_dir: String,
    #[serde(default = "default_blob_service")]
    blob_service: String,
    #[serde(default = "default_directory_service")]
    directory_service: String,
    #[serde(default)]
    /// An optional list of [narinfo::VerifyingKey].
    /// If not empty, the .narinfo files received need to have correct signature by at least one of
    /// these.
    trusted_public_keys: Vec<String>,
}

fn default_store_dir() -> String {
    nix_compat::store_path::STORE_DIR.to_string()
}
fn default_blob_service() -> String {
    "&root".to_string()
}
fn default_directory_service() -> String {
    "&root".to_string()
}

fn validate_store_dir(store_dir: &str) -> Result<(), Error> {
    if !store_dir.starts_with('/') {
        return Err(Error::WrongConfig("store_dir must be absolute"));
    }
    if store_dir.ends_with('/') {
        return Err(Error::WrongConfig("store_dir must not end with a slash"));
    }
    Ok(())
}

impl NixHTTPPathInfoServiceConfig {
    pub fn with_store_dir(mut self, store_dir: String) -> Result<Self, Error> {
        validate_store_dir(&store_dir)?;
        self.params.store_dir = store_dir;
        Ok(self)
    }
}

impl TryFrom<Url> for NixHTTPPathInfoServiceConfig {
    type Error = Box<dyn std::error::Error + Send + Sync>;
    fn try_from(url: Url) -> Result<Self, Self::Error> {
        let scheme =
            url.scheme().strip_prefix("nix+").ok_or_else(|| Error::WrongConfig("scheme must start with nix+"))?;

        if !url.has_authority() {
            Err(Error::WrongConfig("url must have authority component"))?
        }
        if !url.has_host() {
            Err(Error::WrongConfig("url must have host component"))?
        }
        if !["http", "https"].contains(&scheme) {
            Err(Error::WrongConfig("unknown scheme"))?
        }

        Ok(NixHTTPPathInfoServiceConfig {
            // Stringify the URL and remove the nix+ prefix.
            // We can't use `url.set_scheme(rest)`, as it disallows
            // setting something http(s) that previously wasn't.
            // Also make sure to drop the query, we don't want to leak our
            // config to the remote HTTP endpoint we query.
            base_url: {
                let mut url: Url =
                    url.to_string().strip_prefix("nix+").unwrap().parse().expect("stripped URL to parse again");
                url.set_query(None);
                url
            },
            params: serde_qs::from_str(url.query().unwrap_or_default())?,
        })
    }
}

#[async_trait]
impl ServiceBuilder for NixHTTPPathInfoServiceConfig {
    type Output = dyn PathInfoService;
    async fn build<'a>(
        &'a self,
        instance_name: &str,
        context: &CompositionContext,
    ) -> Result<Arc<Self::Output>, Box<dyn std::error::Error + Send + Sync + 'static>> {
        let (blob_service, directory_service) = futures::join!(
            context.resolve::<dyn BlobService>(&self.params.blob_service),
            context.resolve::<dyn DirectoryService>(&self.params.directory_service)
        );
        let svc = NixHTTPPathInfoService::try_build(
            instance_name.to_string(),
            self.to_owned(),
            blob_service?,
            directory_service?,
        )?;
        Ok(Arc::new(svc))
    }
}

#[cfg(test)]
mod tests {
    use std::io::Read;
    use std::io::Write;
    use std::net::TcpListener;
    use std::net::TcpStream;
    use std::sync::Arc;
    use std::sync::Mutex;
    use std::sync::atomic::AtomicBool;
    use std::sync::atomic::Ordering;
    use std::thread;
    use std::thread::JoinHandle;
    use std::time::Duration;

    use rstest::rstest;
    use snix_castore::blobservice::MemoryBlobService;
    use snix_castore::directoryservice::RedbDirectoryService;
    use snix_castore::directoryservice::RedbDirectoryServiceConfig;
    use url::Url;

    use super::Error;
    use super::NixHTTPPathInfoService;
    use super::NixHTTPPathInfoServiceConfig;
    use super::NixHTTPPathInfoServiceParams;
    use super::default_store_dir;
    use super::multi_frame_zstd_decoder;
    use super::normalize_binary_cache_base_url;
    use super::validate_requested_store_path_digest;
    use crate::pathinfoservice::PathInfoService;

    const CACHE_NIXOS_PUBLIC_KEY: &str = "cache.nixos.org-1:6NCHdD59X431o0gWypbMrAURkbJ16ZPMQFGspcDShjY=";
    const TEST_NARINFO: &str = r#"StorePath: /nix/store/00bgd045z0d4icpbc2yyz4gx48ak44la-net-tools-1.60_p20170221182432
URL: nar/1094wph9z4nwlgvsd53abfz8i117ykiv5dwnq9nnhz846s7xqd7d.nar.xz
Compression: xz
FileHash: sha256:1094wph9z4nwlgvsd53abfz8i117ykiv5dwnq9nnhz846s7xqd7d
FileSize: 114980
NarHash: sha256:0lxjvvpr59c2mdram7ympy5ay741f180kv3349hvfc3f8nrmbqf6
NarSize: 464152
References: 7gx4kiv5m0i7d7qkixq2cwzbr10lvxwc-glibc-2.27
Deriver: unknown-deriver
Sig: cache.nixos.org-1:sn5s/RrqEI+YG6/PjwdbPjcAC7rcta7sJU4mFOawGvJBLsWkyLtBrT2EuFt/LJjWkTZ+ZWOI9NTtjo/woMdvAg==
Sig: hydra.other.net-1:JXQ3Z/PXf0EZSFkFioa4FbyYpbbTbHlFBtZf4VqU0tuMTWzhMD7p9Q7acJjLn3jofOtilAAwRILKIfVuyrbjAA==
"#;

    #[derive(Default, Debug)]
    struct RequestCounts {
        narinfo_gets: u32,
        nar_gets: u32,
    }

    fn test_service_config(base_url: Url) -> NixHTTPPathInfoServiceConfig {
        NixHTTPPathInfoServiceConfig {
            base_url,
            params: NixHTTPPathInfoServiceParams {
                store_dir: default_store_dir(),
                blob_service: "&root".to_string(),
                directory_service: "&root".to_string(),
                trusted_public_keys: vec![],
            },
        }
    }

    fn spawn_test_server(
        narinfo_body: &'static str,
    ) -> (Url, Arc<Mutex<RequestCounts>>, Arc<AtomicBool>, JoinHandle<()>) {
        let listener = TcpListener::bind("127.0.0.1:0").expect("listener");
        listener.set_nonblocking(true).expect("nonblocking listener");
        let addr = listener.local_addr().expect("listener addr");
        let counts = Arc::new(Mutex::new(RequestCounts::default()));
        let stop = Arc::new(AtomicBool::new(false));
        let counts_ref = counts.clone();
        let stop_ref = stop.clone();

        let handle = thread::spawn(move || {
            while !stop_ref.load(Ordering::SeqCst) {
                match listener.accept() {
                    Ok((mut stream, _peer)) => handle_connection(&mut stream, narinfo_body, &counts_ref),
                    Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(10))
                    }
                    Err(err) => panic!("accept failed: {err}"),
                }
            }
        });

        (format!("http://{addr}/").parse().expect("base url"), counts, stop, handle)
    }

    fn handle_connection(stream: &mut TcpStream, narinfo_body: &str, counts: &Arc<Mutex<RequestCounts>>) {
        let mut buf = [0u8; 4096];
        let bytes_read = stream.read(&mut buf).expect("read request");
        let request = String::from_utf8_lossy(&buf[..bytes_read]);
        let path = request.lines().next().and_then(|line| line.split_whitespace().nth(1)).unwrap_or("/");

        let (status_line, body) = if path.ends_with(".narinfo") {
            counts.lock().expect("request counts").narinfo_gets += 1;
            ("HTTP/1.1 200 OK", narinfo_body)
        } else if path.ends_with(".nar") || path.ends_with(".nar.xz") {
            counts.lock().expect("request counts").nar_gets += 1;
            ("HTTP/1.1 500 Internal Server Error", "unexpected nar request")
        } else {
            ("HTTP/1.1 200 OK", "wake")
        };

        let response = format!(
            "{status_line}\r\nContent-Length: {}\r\nContent-Type: text/plain\r\nConnection: close\r\n\r\n{body}",
            body.len()
        );
        stream.write_all(response.as_bytes()).expect("write response");
        stream.flush().expect("flush response");
    }

    fn stop_test_server(base_url: &Url, stop: Arc<AtomicBool>, handle: JoinHandle<()>) {
        stop.store(true, Ordering::SeqCst);
        let host = base_url.host_str().expect("host");
        let port = base_url.port_or_known_default().expect("port");
        let _ = TcpStream::connect((host, port));
        handle.join().expect("server join");
    }

    fn test_store_path() -> nix_compat::store_path::StorePath<String> {
        nix_compat::store_path::StorePath::from_absolute_path(
            b"/nix/store/00bgd045z0d4icpbc2yyz4gx48ak44la-net-tools-1.60_p20170221182432",
        )
        .expect("store path")
    }

    fn test_reference_path() -> nix_compat::store_path::StorePath<String> {
        nix_compat::store_path::StorePath::from_absolute_path(b"/nix/store/7gx4kiv5m0i7d7qkixq2cwzbr10lvxwc-glibc-2.27")
            .expect("reference path")
    }

    #[test]
    fn cache_base_normalization_preserves_subpaths_for_endpoint_joins() {
        // r[verify cache_substitution.transport_normalization]
        let without_slash: Url = "https://cache.example.test/nested/cache".parse().unwrap();
        let with_slash: Url = "https://cache.example.test/nested/cache/".parse().unwrap();
        let normalized_without_slash = normalize_binary_cache_base_url(without_slash);
        let normalized_with_slash = normalize_binary_cache_base_url(with_slash);

        assert_eq!(normalized_without_slash, normalized_with_slash);
        assert_eq!(
            normalized_without_slash.join("path.narinfo").unwrap().as_str(),
            "https://cache.example.test/nested/cache/path.narinfo"
        );
    }

    #[tokio::test]
    async fn zstd_decoder_consumes_all_frames_and_rejects_a_malformed_later_frame() {
        // r[verify cache_substitution.transport_normalization]
        use tokio::io::AsyncReadExt;

        async fn compress_frame(bytes: &[u8]) -> Vec<u8> {
            assert!(!bytes.is_empty(), "zstd test frame input must not be empty");
            let mut encoder = async_compression::tokio::bufread::ZstdEncoder::new(tokio::io::BufReader::new(bytes));
            let mut compressed = Vec::new();
            encoder.read_to_end(&mut compressed).await.unwrap();
            assert!(!compressed.is_empty(), "zstd test frame output must not be empty");
            compressed
        }

        let first = compress_frame(b"first-").await;
        let second = compress_frame(b"second").await;
        let mut complete = first.clone();
        complete.extend_from_slice(&second);
        let mut decoder = multi_frame_zstd_decoder(tokio::io::BufReader::new(complete.as_slice()));
        let mut decoded = Vec::new();
        decoder.read_to_end(&mut decoded).await.unwrap();

        assert_eq!(decoded, b"first-second");
        let mut malformed = first;
        malformed.extend_from_slice(b"not-a-zstd-frame");
        let mut decoder = multi_frame_zstd_decoder(tokio::io::BufReader::new(malformed.as_slice()));
        let mut rejected = Vec::new();
        assert!(decoder.read_to_end(&mut rejected).await.is_err());
        assert_ne!(rejected, b"first-second");
    }

    #[test]
    fn narinfo_parsing_uses_the_configured_store_directory() {
        const MANTLE_STORE_DIR: &str = "/mantle/store";
        let body = TEST_NARINFO.replace(nix_compat::store_path::STORE_DIR, MANTLE_STORE_DIR);
        let base_url: Url = "http://127.0.0.1/".parse().unwrap();
        let custom_config = test_service_config(base_url.clone()).with_store_dir(MANTLE_STORE_DIR.to_string()).unwrap();
        let custom_service = NixHTTPPathInfoService::try_build(
            "custom-store".to_string(),
            custom_config,
            MemoryBlobService::default(),
            RedbDirectoryService::new_temporary("custom-store".to_string(), RedbDirectoryServiceConfig::default())
                .unwrap(),
        )
        .unwrap();
        let parsed = custom_service.parse_and_verify_narinfo(&body).unwrap();
        assert_eq!(
            parsed.store_path.to_absolute_path_with_prefix(MANTLE_STORE_DIR),
            "/mantle/store/00bgd045z0d4icpbc2yyz4gx48ak44la-net-tools-1.60_p20170221182432"
        );

        let default_service = NixHTTPPathInfoService::try_build(
            "default-store".to_string(),
            test_service_config(base_url),
            MemoryBlobService::default(),
            RedbDirectoryService::new_temporary("default-store".to_string(), RedbDirectoryServiceConfig::default())
                .unwrap(),
        )
        .unwrap();
        assert!(default_service.parse_and_verify_narinfo(&body).is_err());
        assert!(
            test_service_config("http://127.0.0.1/".parse().unwrap())
                .with_store_dir("relative".to_string())
                .is_err()
        );
        assert!(
            test_service_config("http://127.0.0.1/".parse().unwrap())
                .with_store_dir("/mantle/store/".to_string())
                .is_err()
        );
    }

    #[test]
    fn requested_store_path_digest_validation_accepts_only_an_exact_match() {
        let store_path = test_store_path();
        assert!(validate_requested_store_path_digest(*store_path.digest(), *store_path.digest()).is_ok());

        let wrong_digest = [0u8; 20];
        let error =
            validate_requested_store_path_digest(wrong_digest, *store_path.digest()).expect_err("digest mismatch");
        assert!(matches!(
            error,
            Error::StorePathDigestMismatch {
                requested_digest,
                observed_digest,
            } if requested_digest == wrong_digest && observed_digest == *store_path.digest()
        ));
    }

    #[tokio::test]
    async fn matching_signed_narinfo_returns_references() {
        // r[verify cache_substitution.requested_path_identity]
        let (base_url, counts, stop, handle) = spawn_test_server(TEST_NARINFO);
        let mut config = test_service_config(base_url.clone());
        config.params.trusted_public_keys = vec![CACHE_NIXOS_PUBLIC_KEY.to_string()];
        let service = NixHTTPPathInfoService::try_build(
            "signed-match".to_string(),
            config,
            MemoryBlobService::default(),
            RedbDirectoryService::new_temporary("signed-match".to_string(), RedbDirectoryServiceConfig::default())
                .unwrap(),
        )
        .unwrap();

        let references = service
            .get_references(*test_store_path().digest())
            .await
            .expect("signed narinfo")
            .expect("matching PathInfo references");

        assert_eq!(references, vec![test_reference_path()]);
        let counts = counts.lock().unwrap();
        assert_eq!(counts.narinfo_gets, 1);
        assert_eq!(counts.nar_gets, 0);
        drop(counts);
        stop_test_server(&base_url, stop, handle);
    }

    #[tokio::test]
    async fn malformed_narinfo_returns_no_pathinfo_and_fetches_no_nar() {
        // r[verify cache_substitution.requested_path_identity]
        let (base_url, counts, stop, handle) = spawn_test_server("malformed narinfo");
        let service = NixHTTPPathInfoService::try_build(
            "malformed".to_string(),
            test_service_config(base_url.clone()),
            MemoryBlobService::default(),
            RedbDirectoryService::new_temporary("malformed".to_string(), RedbDirectoryServiceConfig::default())
                .unwrap(),
        )
        .unwrap();

        let error = service.get(*test_store_path().digest()).await.expect_err("malformed metadata must fail");

        assert!(error.to_string().contains("unable to parse NARInfo"));
        let counts = counts.lock().unwrap();
        assert_eq!(counts.narinfo_gets, 1);
        assert_eq!(counts.nar_gets, 0);
        drop(counts);
        stop_test_server(&base_url, stop, handle);
    }

    #[tokio::test]
    async fn get_references_fetches_only_narinfo_metadata() {
        let (base_url, counts, stop, handle) = spawn_test_server(TEST_NARINFO);
        let config = test_service_config(base_url.clone());
        let blob_service = MemoryBlobService::default();
        let directory_service =
            RedbDirectoryService::new_temporary("test".to_string(), RedbDirectoryServiceConfig::default())
                .expect("directory service");
        let service = NixHTTPPathInfoService::try_build("test".to_string(), config, blob_service, directory_service)
            .expect("nix http service");

        let refs = service
            .get_references(*test_store_path().digest())
            .await
            .expect("metadata lookup")
            .expect("narinfo refs");

        assert_eq!(refs, vec![test_reference_path()]);
        let counts = counts.lock().expect("request counts");
        assert_eq!(counts.narinfo_gets, 1);
        assert_eq!(counts.nar_gets, 0);
        drop(counts);
        stop_test_server(&base_url, stop, handle);
    }

    #[tokio::test]
    async fn get_references_rejects_mismatched_store_path_before_nar_fetch() {
        // r[verify cache_substitution.requested_path_identity]
        let (base_url, counts, stop, handle) = spawn_test_server(TEST_NARINFO);
        let mut config = test_service_config(base_url.clone());
        config.params.trusted_public_keys = vec![CACHE_NIXOS_PUBLIC_KEY.to_string()];
        let blob_service = MemoryBlobService::default();
        let directory_service =
            RedbDirectoryService::new_temporary("test".to_string(), RedbDirectoryServiceConfig::default())
                .expect("directory service");
        let service = NixHTTPPathInfoService::try_build("test".to_string(), config, blob_service, directory_service)
            .expect("nix http service");

        let wrong_digest = [0u8; 20];
        let error = service.get_references(wrong_digest).await.expect_err("digest mismatch");
        assert!(error.to_string().contains("narinfo store path digest does not match"));
        let counts = counts.lock().expect("request counts");
        assert_eq!(counts.narinfo_gets, 1);
        assert_eq!(counts.nar_gets, 0);
        drop(counts);
        stop_test_server(&base_url, stop, handle);
    }

    #[tokio::test]
    async fn get_rejects_mismatched_store_path_before_nar_fetch() {
        let (base_url, counts, stop, handle) = spawn_test_server(TEST_NARINFO);
        let config = test_service_config(base_url.clone());
        let blob_service = MemoryBlobService::default();
        let directory_service =
            RedbDirectoryService::new_temporary("test".to_string(), RedbDirectoryServiceConfig::default())
                .expect("directory service");
        let service = NixHTTPPathInfoService::try_build("test".to_string(), config, blob_service, directory_service)
            .expect("nix http service");

        let wrong_digest = [0u8; 20];
        let error = service.get(wrong_digest).await.expect_err("digest mismatch");
        assert!(error.to_string().contains("narinfo store path digest does not match"));
        let counts = counts.lock().expect("request counts");
        assert_eq!(counts.narinfo_gets, 1);
        assert_eq!(counts.nar_gets, 0);
        drop(counts);
        stop_test_server(&base_url, stop, handle);
    }

    #[tokio::test]
    async fn get_still_fetches_nar_payload_for_full_pathinfo() {
        let (base_url, counts, stop, handle) = spawn_test_server(TEST_NARINFO);
        let config = test_service_config(base_url.clone());
        let blob_service = MemoryBlobService::default();
        let directory_service =
            RedbDirectoryService::new_temporary("test".to_string(), RedbDirectoryServiceConfig::default())
                .expect("directory service");
        let service = NixHTTPPathInfoService::try_build("test".to_string(), config, blob_service, directory_service)
            .expect("nix http service");

        let err = service.get(*test_store_path().digest()).await.expect_err("full get should request nar payload");
        assert!(format!("{err}").contains("failed to request NAR"));
        let counts = counts.lock().expect("request counts");
        assert_eq!(counts.narinfo_gets, 1);
        assert_eq!(counts.nar_gets, 1);
        drop(counts);
        stop_test_server(&base_url, stop, handle);
    }

    #[rstest]
    /// Correct Scheme for the cache.nixos.org binary cache.
    #[case::correct_nix_https("nix+https://cache.nixos.org", Some(
        NixHTTPPathInfoServiceConfig {
            base_url: "https://cache.nixos.org".try_into().unwrap(),
            params: NixHTTPPathInfoServiceParams {
                store_dir: default_store_dir(),
                blob_service: "&root".to_string(),
                directory_service: "&root".to_string(),
                trusted_public_keys: vec![]
            }
        }
    ))]
    /// Correct Scheme for the cache.nixos.org binary cache (HTTP URL).
    #[case::correct_nix_http("nix+http://cache.nixos.org", Some(
        NixHTTPPathInfoServiceConfig {
            base_url: "http://cache.nixos.org".try_into().unwrap(),
            params: NixHTTPPathInfoServiceParams {
                store_dir: default_store_dir(),
                blob_service: "&root".to_string(),
                directory_service: "&root".to_string(),
                trusted_public_keys: vec![]
            }
        }
    ))]
    /// Correct Scheme for Nix HTTP Binary cache, with a subpath.
    #[case::correct_nix_http_with_subpath("nix+http://192.0.2.1/foo", Some(
        NixHTTPPathInfoServiceConfig {
            base_url: "http://192.0.2.1/foo".try_into().unwrap(),
            params: NixHTTPPathInfoServiceParams {
                store_dir: default_store_dir(),
                blob_service: "&root".to_string(),
                directory_service: "&root".to_string(),
                trusted_public_keys: vec![]
            }
        }
    ))]
    /// Correct Scheme for Nix HTTP Binary cache, with a subpath and port.
    #[case::correct_nix_http_with_subpath_and_port("nix+http://[::1]:8080/foo", Some(
        NixHTTPPathInfoServiceConfig {
            base_url: "http://[::1]:8080/foo".try_into().unwrap(),
            params: NixHTTPPathInfoServiceParams {
                store_dir: default_store_dir(),
                blob_service: "&root".to_string(),
                directory_service: "&root".to_string(),
                trusted_public_keys: vec![]
            }
        }

    ))]
    /// Correct Scheme for the cache.nixos.org binary cache, and correct trusted public key set
    #[case::correct_nix_https_with_trusted_public_key(
        "nix+https://cache.nixos.org?trusted_public_keys[0]=cache.nixos.org-1:6NCHdD59X431o0gWypbMrAURkbJ16ZPMQFGspcDShjY=", Some(
        NixHTTPPathInfoServiceConfig {
            base_url: "https://cache.nixos.org".try_into().unwrap(),
            params: NixHTTPPathInfoServiceParams {
                blob_service: "&root".to_string(),
                directory_service: "&root".to_string(),
                store_dir: default_store_dir(),
                trusted_public_keys: vec![
                    "cache.nixos.org-1:6NCHdD59X431o0gWypbMrAURkbJ16ZPMQFGspcDShjY=".to_string()
                ]
            }
        }
    ))]
    /// Correct Scheme for the cache.nixos.org binary cache, and two correct trusted public keys set
    #[case::correct_nix_https_with_two_trusted_public_keys(
        "nix+https://cache.nixos.org?trusted_public_keys[0]=cache.nixos.org-1:6NCHdD59X431o0gWypbMrAURkbJ16ZPMQFGspcDShjY=&trusted_public_keys[1]=foo:jp4fCEx9tBEId/L0ZsVJ26k0wC0fu7vJqLjjIGFkup8=", Some(
        NixHTTPPathInfoServiceConfig {
            base_url: "https://cache.nixos.org".try_into().unwrap(),
            params: NixHTTPPathInfoServiceParams {
                blob_service: "&root".to_string(),
                directory_service: "&root".to_string(),
                store_dir: default_store_dir(),
                trusted_public_keys: vec![
                    "cache.nixos.org-1:6NCHdD59X431o0gWypbMrAURkbJ16ZPMQFGspcDShjY=".to_string(),
                    "foo:jp4fCEx9tBEId/L0ZsVJ26k0wC0fu7vJqLjjIGFkup8=".to_string()
                ]
            }
        }
    ))]
    #[case::wrong_scheme("nix+grpc://example.com", None)]
    #[case::missing_host("nix+http:///", None)]
    #[case::missing_authority("nix+http:", None)]
    /// Correct cache.nixos.org binary cache URL, but wrong `trusted_public_keys` param usage
    /// (should be list)
    #[case::trusted_public_keys_no_sequence(
        "nix+https://cache.nixos.org?trusted_public_keys=cache.nixos.org-1:6NCHdD59X431o0gWypbMrAURkbJ16ZPMQFGspcDShjY=",
        None
    )]
    /// Correct cache.nixos.org binary cache URL, but wrong param name
    #[case::trusted_public_keys_wrong_pubkey(
        "nix+https://cache.nixos.org?trustedpublickeys=cache.nixos.org-1:6NCHdD59X431o0gWypbMrAURkbJ16ZPMQFGspcDShjY=",
        None
    )]
    fn parse_url(#[case] url_str: &str, #[case] exp_config: Option<NixHTTPPathInfoServiceConfig>) {
        let url: Url = url_str.parse().expect("url to parse");

        match (NixHTTPPathInfoServiceConfig::try_from(url), exp_config) {
            (Ok(_), None) => panic!("parsing url unexpectedly succeeded"),
            (Ok(config), Some(exp_config)) => assert_eq!(exp_config, config),
            (Err(_), None) => {}
            (Err(e), Some(_)) => panic!("parsing url unexpectedly failed: {e}"),
        }
    }
}
