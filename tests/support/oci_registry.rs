use std::collections::BTreeMap;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Read;
use std::io::Write;
use std::net::SocketAddr;
use std::net::TcpListener;
use std::net::TcpStream;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::thread::JoinHandle;
use std::time::Duration;

use mantle::oci_projection::sha256_digest;
use mantle::oci_registry::MANTLE_SIGNATURE_DOCUMENT_MEDIA_TYPE;
use mantle::oci_registry::OCI_ARTIFACT_MANIFEST_MEDIA_TYPE;
use mantle::oci_registry::OciArtifactManifest;
use mantle::oci_registry::RegistrySignatureDocument;
use url::Url;

const ACCEPT_POLL_DELAY_MS: u64 = 2;
const READ_TIMEOUT_SECONDS: u64 = 5;
const HEADER_BYTES_MAX: usize = 65_536;
const HEADER_COUNT_MAX: usize = 128;
const MEBIBYTE_BYTES: usize = 1_048_576;
const REQUEST_BODY_LIMIT_MEBIBYTES: usize = 16;
const REQUEST_BODY_BYTES_MAX: usize = REQUEST_BODY_LIMIT_MEBIBYTES * MEBIBYTE_BYTES;
const HTTP_STATUS_OK: u16 = 200;
const HTTP_STATUS_CREATED: u16 = 201;
const HTTP_STATUS_ACCEPTED: u16 = 202;
const HTTP_STATUS_UNAUTHORIZED: u16 = 401;
const HTTP_STATUS_NOT_FOUND: u16 = 404;
const HTTP_STATUS_INTERNAL_ERROR: u16 = 500;
const CONTENT_TYPE_HEADER: &str = "content-type";
const AUTHORIZATION_HEADER: &str = "authorization";
const DIGEST_HEADER: &str = "Docker-Content-Digest";
const LOCATION_HEADER: &str = "Location";
const OCI_BLOB_CONTENT_TYPE: &str = "application/octet-stream";
const OCI_IMAGE_MANIFEST_MEDIA_TYPE: &str = "application/vnd.oci.image.manifest.v1+json";
const REGISTRY_API_CONTENT_TYPE: &str = "application/json";
const UPLOAD_ID_PREFIX: &str = "upload-";

#[derive(Clone)]
struct StoredManifest {
    media_type: String,
    digest: String,
    bytes: Vec<u8>,
}

#[derive(Default)]
struct RegistryState {
    blobs: BTreeMap<String, Vec<u8>>,
    manifests: BTreeMap<(String, String), StoredManifest>,
    uploads: BTreeMap<String, String>,
    next_upload_id: u64,
    fail_manifest_once: Option<String>,
    blob_gets: Vec<String>,
}

struct HttpRequest {
    method: String,
    target: String,
    headers: BTreeMap<String, String>,
    body: Vec<u8>,
}

struct HttpResponse {
    status: u16,
    content_type: String,
    headers: Vec<(String, String)>,
    body: Vec<u8>,
}

pub struct TestRegistry {
    address: SocketAddr,
    bearer_token: String,
    state: Arc<Mutex<RegistryState>>,
    shutdown: Arc<AtomicBool>,
    thread: Option<JoinHandle<()>>,
}

impl HttpResponse {
    fn empty(status: u16) -> Self {
        Self {
            status,
            content_type: REGISTRY_API_CONTENT_TYPE.to_string(),
            headers: Vec::new(),
            body: Vec::new(),
        }
    }

    fn bytes(status: u16, content_type: &str, body: Vec<u8>) -> Self {
        Self {
            status,
            content_type: content_type.to_string(),
            headers: Vec::new(),
            body,
        }
    }

    fn with_header(mut self, name: &str, value: impl Into<String>) -> Self {
        self.headers.push((name.to_string(), value.into()));
        self
    }
}

fn reason_phrase(status: u16) -> &'static str {
    match status {
        HTTP_STATUS_OK => "OK",
        HTTP_STATUS_CREATED => "Created",
        HTTP_STATUS_ACCEPTED => "Accepted",
        HTTP_STATUS_UNAUTHORIZED => "Unauthorized",
        HTTP_STATUS_NOT_FOUND => "Not Found",
        HTTP_STATUS_INTERNAL_ERROR => "Internal Server Error",
        _ => "Unknown",
    }
}

fn read_request(stream: &mut TcpStream) -> Result<Option<HttpRequest>, String> {
    stream
        .set_read_timeout(Some(Duration::from_secs(READ_TIMEOUT_SECONDS)))
        .map_err(|error| format!("setting registry test read timeout: {error}"))?;
    let mut reader = BufReader::new(stream);
    let mut request_line = String::new();
    if reader
        .read_line(&mut request_line)
        .map_err(|error| format!("reading registry request line: {error}"))?
        == 0
    {
        return Ok(None);
    }
    let mut parts = request_line.trim_end_matches(['\r', '\n']).split_whitespace();
    let method = parts.next().ok_or_else(|| "registry request method is missing".to_string())?.to_string();
    let target = parts.next().ok_or_else(|| "registry request target is missing".to_string())?.to_string();
    if parts.next() != Some("HTTP/1.1") || parts.next().is_some() {
        return Err("registry request line is malformed".to_string());
    }
    let headers = read_headers(&mut reader, request_line.len())?;
    let content_length = headers
        .get("content-length")
        .map(|value| value.parse::<usize>().map_err(|_| "registry content length is invalid".to_string()))
        .transpose()?
        .unwrap_or(0);
    if content_length > REQUEST_BODY_BYTES_MAX {
        return Err("registry request body exceeds the test bound".to_string());
    }
    let mut body = vec![0_u8; content_length];
    reader.read_exact(&mut body).map_err(|error| format!("reading registry request body: {error}"))?;
    assert!(headers.len() <= HEADER_COUNT_MAX, "parsed registry header count must stay bounded");
    assert_eq!(body.len(), content_length, "parsed registry body length must match its header");
    Ok(Some(HttpRequest {
        method,
        target,
        headers,
        body,
    }))
}

fn read_headers(
    reader: &mut BufReader<&mut TcpStream>,
    initial_bytes: usize,
) -> Result<BTreeMap<String, String>, String> {
    let mut headers = BTreeMap::new();
    let mut total_bytes = initial_bytes;
    loop {
        let mut line = String::new();
        let read = reader.read_line(&mut line).map_err(|error| format!("reading registry request header: {error}"))?;
        if read == 0 {
            return Err("registry request ended before its header terminator".to_string());
        }
        total_bytes = total_bytes.saturating_add(read);
        if total_bytes > HEADER_BYTES_MAX || headers.len() >= HEADER_COUNT_MAX {
            return Err("registry request headers exceed the test bound".to_string());
        }
        if line == "\r\n" {
            break;
        }
        let (name, value) = line
            .trim_end_matches(['\r', '\n'])
            .split_once(':')
            .ok_or_else(|| "registry request header is malformed".to_string())?;
        headers.insert(name.trim().to_ascii_lowercase(), value.trim().to_string());
    }
    assert!(total_bytes <= HEADER_BYTES_MAX, "parsed registry headers must stay bounded");
    assert!(headers.len() < HEADER_COUNT_MAX, "parsed registry header count must stay below its bound");
    Ok(headers)
}

fn write_response(stream: &mut TcpStream, response: &HttpResponse) -> Result<(), String> {
    write!(
        stream,
        "HTTP/1.1 {} {}\r\nContent-Length: {}\r\nContent-Type: {}\r\nConnection: close\r\n",
        response.status,
        reason_phrase(response.status),
        response.body.len(),
        response.content_type
    )
    .map_err(|error| format!("writing registry response head: {error}"))?;
    for (name, value) in &response.headers {
        write!(stream, "{name}: {value}\r\n").map_err(|error| format!("writing registry response header: {error}"))?;
    }
    stream.write_all(b"\r\n").map_err(|error| format!("writing registry response separator: {error}"))?;
    stream
        .write_all(&response.body)
        .map_err(|error| format!("writing registry response body: {error}"))?;
    stream.flush().map_err(|error| format!("flushing registry response: {error}"))?;
    assert_eq!(
        reason_phrase(response.status) == "Unknown",
        !matches!(
            response.status,
            HTTP_STATUS_OK
                | HTTP_STATUS_CREATED
                | HTTP_STATUS_ACCEPTED
                | HTTP_STATUS_UNAUTHORIZED
                | HTTP_STATUS_NOT_FOUND
                | HTTP_STATUS_INTERNAL_ERROR
        )
    );
    assert!(response.body.len() <= REQUEST_BODY_BYTES_MAX, "registry test response must stay bounded");
    Ok(())
}

fn unauthorized() -> HttpResponse {
    HttpResponse::empty(HTTP_STATUS_UNAUTHORIZED).with_header("WWW-Authenticate", "Bearer realm=\"mantle-test\"")
}

fn split_named_path<'a>(path: &'a str, marker: &str) -> Option<(&'a str, &'a str)> {
    let remainder = path.strip_prefix("/v2/")?;
    let (repository, tail) = remainder.split_once(marker)?;
    if repository.is_empty() {
        return None;
    }
    Some((repository, tail))
}

fn parsed_target(target: &str) -> Result<Url, String> {
    Url::parse(&format!("http://registry.test{target}"))
        .map_err(|error| format!("parsing registry test request target: {error}"))
}

fn upload_digest(target: &Url) -> Option<String> {
    target.query_pairs().find_map(|(name, value)| (name == "digest").then(|| value.into_owned()))
}

fn handle_blob(request: &HttpRequest, repository: &str, digest: &str, state: &mut RegistryState) -> HttpResponse {
    let _repository = repository;
    let Some(bytes) = state.blobs.get(digest).cloned() else {
        return HttpResponse::empty(HTTP_STATUS_NOT_FOUND);
    };
    match request.method.as_str() {
        "HEAD" => HttpResponse::empty(HTTP_STATUS_OK).with_header(DIGEST_HEADER, digest),
        "GET" => {
            state.blob_gets.push(digest.to_string());
            HttpResponse::bytes(HTTP_STATUS_OK, OCI_BLOB_CONTENT_TYPE, bytes).with_header(DIGEST_HEADER, digest)
        }
        _ => HttpResponse::empty(HTTP_STATUS_NOT_FOUND),
    }
}

fn start_upload(repository: &str, state: &mut RegistryState) -> HttpResponse {
    state.next_upload_id = state.next_upload_id.saturating_add(1);
    let id = format!("{UPLOAD_ID_PREFIX}{}", state.next_upload_id);
    state.uploads.insert(id.clone(), repository.to_string());
    let location = format!("/v2/{repository}/blobs/uploads/{id}");
    assert!(location.starts_with("/v2/"), "upload location must remain registry-relative");
    assert!(state.uploads.contains_key(&id), "started upload must be tracked");
    HttpResponse::empty(HTTP_STATUS_ACCEPTED).with_header(LOCATION_HEADER, location)
}

fn finish_upload(
    request: &HttpRequest,
    repository: &str,
    upload_id: &str,
    target: &Url,
    state: &mut RegistryState,
) -> HttpResponse {
    if request.method != "PUT" || state.uploads.get(upload_id).map(String::as_str) != Some(repository) {
        return HttpResponse::empty(HTTP_STATUS_NOT_FOUND);
    }
    let Some(digest) = upload_digest(target) else {
        return HttpResponse::empty(HTTP_STATUS_NOT_FOUND);
    };
    if sha256_digest(&request.body) != digest {
        return HttpResponse::empty(HTTP_STATUS_INTERNAL_ERROR);
    }
    state.uploads.remove(upload_id);
    state.blobs.insert(digest.clone(), request.body.clone());
    let location = format!("/v2/{repository}/blobs/{digest}");
    assert!(state.blobs.contains_key(&digest), "completed upload must retain the blob");
    assert!(!state.uploads.contains_key(upload_id), "completed upload must clear transient state");
    HttpResponse::empty(HTTP_STATUS_CREATED)
        .with_header(DIGEST_HEADER, digest)
        .with_header(LOCATION_HEADER, location)
}

fn store_manifest(request: &HttpRequest, repository: &str, reference: &str, state: &mut RegistryState) -> HttpResponse {
    if state.fail_manifest_once.as_deref() == Some(reference) {
        state.fail_manifest_once = None;
        return HttpResponse::empty(HTTP_STATUS_INTERNAL_ERROR);
    }
    let media_type = request
        .headers
        .get(CONTENT_TYPE_HEADER)
        .cloned()
        .unwrap_or_else(|| OCI_IMAGE_MANIFEST_MEDIA_TYPE.to_string());
    let digest = sha256_digest(&request.body);
    let stored = StoredManifest {
        media_type,
        digest: digest.clone(),
        bytes: request.body.clone(),
    };
    state.manifests.insert((repository.to_string(), reference.to_string()), stored.clone());
    state.manifests.insert((repository.to_string(), digest.clone()), stored);
    assert!(state.manifests.contains_key(&(repository.to_string(), reference.to_string())));
    assert!(state.manifests.contains_key(&(repository.to_string(), digest.clone())));
    HttpResponse::empty(HTTP_STATUS_CREATED).with_header(DIGEST_HEADER, digest)
}

fn get_manifest(repository: &str, reference: &str, state: &RegistryState) -> HttpResponse {
    let Some(stored) = state.manifests.get(&(repository.to_string(), reference.to_string())) else {
        return HttpResponse::empty(HTTP_STATUS_NOT_FOUND);
    };
    HttpResponse::bytes(HTTP_STATUS_OK, &stored.media_type, stored.bytes.clone())
        .with_header(DIGEST_HEADER, stored.digest.clone())
}

fn route_request(request: &HttpRequest, state: &mut RegistryState, bearer_token: &str) -> HttpResponse {
    let expected_auth = format!("Bearer {bearer_token}");
    if request.headers.get(AUTHORIZATION_HEADER) != Some(&expected_auth) {
        return unauthorized();
    }
    let Ok(target) = parsed_target(&request.target) else {
        return HttpResponse::empty(HTTP_STATUS_NOT_FOUND);
    };
    let path = target.path();
    if request.method == "GET" && path == "/v2/" {
        return HttpResponse::bytes(HTTP_STATUS_OK, REGISTRY_API_CONTENT_TYPE, b"{}".to_vec());
    }
    if let Some((repository, upload_id)) = split_named_path(path, "/blobs/uploads/") {
        if upload_id.is_empty() && request.method == "POST" {
            return start_upload(repository, state);
        }
        return finish_upload(request, repository, upload_id, &target, state);
    }
    if let Some((repository, digest)) = split_named_path(path, "/blobs/") {
        return handle_blob(request, repository, digest, state);
    }
    if let Some((repository, reference)) = split_named_path(path, "/manifests/") {
        return match request.method.as_str() {
            "PUT" => store_manifest(request, repository, reference, state),
            "GET" => get_manifest(repository, reference, state),
            _ => HttpResponse::empty(HTTP_STATUS_NOT_FOUND),
        };
    }
    HttpResponse::empty(HTTP_STATUS_NOT_FOUND)
}

fn serve_connection(stream: &mut TcpStream, state: &Arc<Mutex<RegistryState>>, token: &str) -> Result<(), String> {
    let Some(request) = read_request(stream)? else {
        return Ok(());
    };
    let response = {
        let mut state = state.lock().map_err(|_| "locking registry test state".to_string())?;
        route_request(&request, &mut state, token)
    };
    write_response(stream, &response)
}

fn run_server(listener: TcpListener, state: Arc<Mutex<RegistryState>>, shutdown: Arc<AtomicBool>, token: String) {
    while !shutdown.load(Ordering::Acquire) {
        match listener.accept() {
            Ok((mut stream, _peer)) => {
                let _result = serve_connection(&mut stream, &state, &token);
            }
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                std::thread::sleep(Duration::from_millis(ACCEPT_POLL_DELAY_MS));
            }
            Err(_) => break,
        }
    }
}

impl TestRegistry {
    pub fn start(bearer_token: &str) -> Self {
        assert!(!bearer_token.is_empty(), "test registry bearer token must be explicit");
        assert!(
            !bearer_token.bytes().any(|byte| byte.is_ascii_whitespace()),
            "test registry bearer token must be one word"
        );
        let listener = TcpListener::bind("127.0.0.1:0").expect("test registry listener should bind");
        let address = listener.local_addr().expect("test registry address should resolve");
        listener.set_nonblocking(true).expect("test registry listener should be nonblocking");
        let state = Arc::new(Mutex::new(RegistryState::default()));
        let shutdown = Arc::new(AtomicBool::new(false));
        let thread = {
            let state = Arc::clone(&state);
            let shutdown = Arc::clone(&shutdown);
            let token = bearer_token.to_string();
            std::thread::spawn(move || run_server(listener, state, shutdown, token))
        };
        Self {
            address,
            bearer_token: bearer_token.to_string(),
            state,
            shutdown,
            thread: Some(thread),
        }
    }

    pub fn url(&self) -> String {
        format!("http://{}", self.address)
    }

    pub fn write_token_file(&self, path: &std::path::Path) {
        fs_err_write(path, format!("{}\n", self.bearer_token).as_bytes());
    }

    pub fn fail_next_manifest(&self, reference: &str) {
        let mut state = self.state.lock().expect("registry test state should lock");
        state.fail_manifest_once = Some(reference.to_string());
        assert_eq!(state.fail_manifest_once.as_deref(), Some(reference));
        assert!(!reference.is_empty());
    }

    pub fn replace_tag_with_drift(&self, repository: &str, reference: &str) {
        let bytes =
            br#"{"schemaVersion":2,"mediaType":"application/vnd.oci.image.manifest.v1+json","drift":true}"#.to_vec();
        let digest = sha256_digest(&bytes);
        let stored = StoredManifest {
            media_type: OCI_IMAGE_MANIFEST_MEDIA_TYPE.to_string(),
            digest,
            bytes,
        };
        let mut state = self.state.lock().expect("registry test state should lock");
        state.manifests.insert((repository.to_string(), reference.to_string()), stored);
        assert!(state.manifests.contains_key(&(repository.to_string(), reference.to_string())));
        assert!(!reference.is_empty());
    }

    pub fn replace_artifact_tag_with_drift(&self, repository: &str, reference: &str) {
        let bytes =
            br#"{"schemaVersion":2,"mediaType":"application/vnd.oci.artifact.manifest.v1+json","drift":true}"#.to_vec();
        let digest = sha256_digest(&bytes);
        let stored = StoredManifest {
            media_type: OCI_ARTIFACT_MANIFEST_MEDIA_TYPE.to_string(),
            digest,
            bytes,
        };
        let mut state = self.state.lock().expect("registry test state should lock");
        state.manifests.insert((repository.to_string(), reference.to_string()), stored);
        assert!(state.manifests.contains_key(&(repository.to_string(), reference.to_string())));
        assert!(!reference.is_empty());
    }

    pub fn tamper_metadata_blob(&self, repository: &str, metadata_manifest_digest: &str) {
        let mut state = self.state.lock().expect("registry test state should lock");
        let stored = state
            .manifests
            .get(&(repository.to_string(), metadata_manifest_digest.to_string()))
            .cloned()
            .expect("metadata manifest should exist by digest");
        let manifest: OciArtifactManifest =
            serde_json::from_slice(&stored.bytes).expect("metadata manifest should parse");
        let descriptor = manifest.blobs.first().expect("metadata manifest should contain blobs");
        let bytes = state.blobs.get_mut(&descriptor.digest).expect("metadata blob should exist");
        let first = bytes.first_mut().expect("metadata blob should not be empty");
        *first ^= 1;
        assert_ne!(sha256_digest(bytes), descriptor.digest);
        assert_eq!(manifest.subject.media_type, OCI_IMAGE_MANIFEST_MEDIA_TYPE);
    }

    pub fn replace_signature_with_invalid_bytes(
        &self,
        repository: &str,
        signature_reference: &str,
        signature_manifest_digest: &str,
    ) -> String {
        let mut state = self.state.lock().expect("registry test state should lock");
        let stored = state
            .manifests
            .get(&(repository.to_string(), signature_manifest_digest.to_string()))
            .cloned()
            .expect("signature manifest should exist by digest");
        let mut manifest: OciArtifactManifest =
            serde_json::from_slice(&stored.bytes).expect("signature manifest should parse");
        let old_descriptor = manifest.blobs.first().expect("signature manifest should contain a document").clone();
        let document_bytes = state.blobs.get(&old_descriptor.digest).expect("signature document should exist");
        let mut document: RegistrySignatureDocument =
            serde_json::from_slice(document_bytes).expect("signature document should parse");
        let encoded = &mut document.signatures[0].signature;
        let mut bytes = encoded.as_bytes().to_vec();
        let mutation_index = bytes.len().checked_sub(3).expect("signature text should be bounded");
        bytes[mutation_index] = if bytes[mutation_index] == b'A' { b'B' } else { b'A' };
        *encoded = String::from_utf8(bytes).expect("mutated signature should remain UTF-8");
        let new_document_bytes = serde_json::to_vec(&document).expect("mutated signature document should serialize");
        let new_document_digest = sha256_digest(&new_document_bytes);
        manifest.blobs[0].digest = new_document_digest.clone();
        manifest.blobs[0].size = new_document_bytes.len() as u64;
        manifest.blobs[0].media_type = MANTLE_SIGNATURE_DOCUMENT_MEDIA_TYPE.to_string();
        let manifest_bytes = serde_json::to_vec(&manifest).expect("mutated signature manifest should serialize");
        let manifest_digest = sha256_digest(&manifest_bytes);
        state.blobs.insert(new_document_digest, new_document_bytes);
        let replacement = StoredManifest {
            media_type: stored.media_type,
            digest: manifest_digest.clone(),
            bytes: manifest_bytes,
        };
        state
            .manifests
            .insert((repository.to_string(), signature_reference.to_string()), replacement.clone());
        state.manifests.insert((repository.to_string(), manifest_digest.clone()), replacement);
        assert!(state.manifests.contains_key(&(repository.to_string(), manifest_digest.clone())));
        assert_ne!(manifest_digest, signature_manifest_digest);
        manifest_digest
    }

    pub fn blob_get_count(&self) -> usize {
        let state = self.state.lock().expect("registry test state should lock");
        assert!(state.blob_gets.len() <= REQUEST_BODY_BYTES_MAX);
        assert!(state.next_upload_id > 0 || state.blob_gets.is_empty());
        state.blob_gets.len()
    }
}

fn fs_err_write(path: &std::path::Path, bytes: &[u8]) {
    std::fs::write(path, bytes).unwrap_or_else(|error| panic!("writing {}: {error}", path.display()));
}

impl Drop for TestRegistry {
    fn drop(&mut self) {
        self.shutdown.store(true, Ordering::Release);
        let _wake_result = TcpStream::connect(self.address);
        if let Some(thread) = self.thread.take() {
            thread.join().expect("test registry thread should stop");
        }
    }
}
