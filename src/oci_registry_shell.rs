//! Thin HTTP/filesystem shell for bounded OCI registry publication and pull.
//!
//! r[impl kernel_bundle_oci.registry_transport]
//! r[impl kernel_bundle_oci.registry_admission]

use std::collections::BTreeMap;
use std::fs;
use std::io::Read;
use std::path::Path;
use std::time::Duration;

use crate::oci_projection::OCI_BLOB_MAX_BYTES;
use crate::oci_projection::OCI_DOCUMENT_MAX_BYTES;
use crate::oci_projection::OCI_EXPORT_REPORT_FILENAME;
use crate::oci_projection::OciDescriptor;
use crate::oci_projection::sha256_digest;
use crate::oci_projection_shell::ImportRequest;
use crate::oci_projection_shell::import_oci_layout;
use crate::oci_projection_shell::publish_pulled_layout;
use crate::oci_projection_shell::read_bounded_regular;
use crate::oci_projection_shell::read_layout_facts;
use crate::oci_registry::CREDENTIAL_MODE_ANONYMOUS;
use crate::oci_registry::CREDENTIAL_MODE_BEARER_FILE;
use crate::oci_registry::FinalizeUploadInput;
use crate::oci_registry::OCI_ARTIFACT_MANIFEST_MEDIA_TYPE;
use crate::oci_registry::OciRegistryPullReport;
use crate::oci_registry::OciRegistryPushReport;
use crate::oci_registry::PullReportFacts;
use crate::oci_registry::PushReportFacts;
use crate::oci_registry::RegistryLayoutInput;
use crate::oci_registry::RegistryPullInput;
use crate::oci_registry::RegistryTarget;
use crate::oci_registry::build_pull_report;
use crate::oci_registry::build_push_report;
use crate::oci_registry::build_registry_pull_plan;
use crate::oci_registry::build_registry_push_plan;
use crate::oci_registry::finalize_upload_url;
use crate::oci_registry::inspect_metadata_manifest;
use crate::oci_registry::pull_blob_descriptors;
use crate::oci_registry::registry_blob_url;
use crate::oci_registry::registry_manifest_url;
use crate::oci_registry::registry_upload_url;
use crate::oci_registry::registry_v2_url;

const AUTHORIZATION_HEADER: &str = "Authorization";
const CONTENT_TYPE_HEADER: &str = "Content-Type";
const DIGEST_HEADER: &str = "Docker-Content-Digest";
const LOCATION_HEADER: &str = "Location";
const BEARER_PREFIX: &str = "Bearer ";
const TOKEN_BYTES_MAX: u64 = 16_384;
const RESPONSE_PROBE_BYTES: u64 = 1;
const CONNECT_TIMEOUT_SECONDS: u64 = 10;
const REQUEST_TIMEOUT_SECONDS: u64 = 120;
const HTTP_STATUS_OK: u16 = 200;
const HTTP_STATUS_CREATED: u16 = 201;
const HTTP_STATUS_ACCEPTED: u16 = 202;
const HTTP_STATUS_NOT_FOUND: u16 = 404;
const OCI_BLOB_CONTENT_TYPE: &str = "application/octet-stream";

pub struct RegistryPushRequest<'a> {
    pub target: &'a RegistryTarget,
    pub layout_dir: &'a Path,
    pub bearer_token_file: Option<&'a Path>,
}

pub struct RegistryPullRequest<'a> {
    pub target: &'a RegistryTarget,
    pub expected_manifest_digest: &'a str,
    pub expected_metadata_manifest_digest: &'a str,
    pub output_dir: &'a Path,
    pub state_dir: &'a Path,
    pub import_report_path: &'a Path,
    pub bearer_token_file: Option<&'a Path>,
}

struct RegistryCredentials {
    authorization: Option<String>,
    mode: &'static str,
}

struct RegistryResponse {
    status: u16,
    content_type: Option<String>,
    digest: Option<String>,
    location: Option<String>,
    body: Vec<u8>,
}

struct RegistryHttpClient {
    agent: ureq::Agent,
    authorization: Option<String>,
}

struct ManifestResponse {
    media_type: String,
    digest: String,
    bytes: Vec<u8>,
}

#[derive(Default)]
struct PushAccounting {
    uploaded_blobs: u32,
    reused_blobs: u32,
    transferred_bytes: u64,
}

#[derive(Default)]
struct PullAccounting {
    downloaded_blobs: u32,
    downloaded_bytes: u64,
}

fn checked_response_limit(max_bytes: u64) -> Result<u64, String> {
    max_bytes
        .checked_add(RESPONSE_PROBE_BYTES)
        .ok_or_else(|| "registry response bound overflowed".to_string())
}

fn read_credentials(path: Option<&Path>) -> Result<RegistryCredentials, String> {
    let Some(path) = path else {
        return Ok(RegistryCredentials {
            authorization: None,
            mode: CREDENTIAL_MODE_ANONYMOUS,
        });
    };
    let bytes = read_bounded_regular(path, TOKEN_BYTES_MAX)?;
    let token = std::str::from_utf8(&bytes)
        .map_err(|_| "registry bearer token file is not UTF-8".to_string())?
        .trim_end_matches(['\r', '\n']);
    if token.is_empty() || token.bytes().any(|byte| byte.is_ascii_control() || byte.is_ascii_whitespace()) {
        return Err("registry bearer token is empty or contains control/whitespace bytes".to_string());
    }
    let authorization = format!("{BEARER_PREFIX}{token}");
    assert!(authorization.starts_with(BEARER_PREFIX), "authorization must use the Bearer scheme");
    assert!(
        authorization.len() <= bytes.len().saturating_add(BEARER_PREFIX.len()),
        "trimmed credential must stay bounded"
    );
    Ok(RegistryCredentials {
        authorization: Some(authorization),
        mode: CREDENTIAL_MODE_BEARER_FILE,
    })
}

impl RegistryHttpClient {
    fn new(credentials: &RegistryCredentials) -> Self {
        let agent = ureq::Agent::config_builder()
            .timeout_connect(Some(Duration::from_secs(CONNECT_TIMEOUT_SECONDS)))
            .timeout_global(Some(Duration::from_secs(REQUEST_TIMEOUT_SECONDS)))
            .max_redirects(0)
            .http_status_as_error(false)
            .proxy(None)
            .build()
            .new_agent();
        assert_eq!(agent.config().max_redirects(), 0, "registry client must not follow redirects");
        assert!(agent.config().proxy().is_none(), "registry client must ignore ambient proxies");
        Self {
            agent,
            authorization: credentials.authorization.clone(),
        }
    }

    fn get(&self, url: &str, accept: &str, max_bytes: u64) -> Result<RegistryResponse, String> {
        let result = if let Some(value) = &self.authorization {
            self.agent.get(url).header(AUTHORIZATION_HEADER, value).header("Accept", accept).call()
        } else {
            self.agent.get(url).header("Accept", accept).call()
        };
        let mut response = result.map_err(|error| format!("registry GET transport failed: {error}"))?;
        let status = response.status().as_u16();
        let content_type = response
            .headers()
            .get(CONTENT_TYPE_HEADER)
            .and_then(|value| value.to_str().ok())
            .map(ToString::to_string);
        let digest =
            response.headers().get(DIGEST_HEADER).and_then(|value| value.to_str().ok()).map(ToString::to_string);
        let mut body = Vec::new();
        response
            .body_mut()
            .as_reader()
            .take(checked_response_limit(max_bytes)?)
            .read_to_end(&mut body)
            .map_err(|error| format!("reading bounded registry response: {error}"))?;
        if u64::try_from(body.len()).map_err(|_| "registry response length overflowed".to_string())? > max_bytes {
            return Err("registry response exceeds the named byte bound".to_string());
        }
        Ok(RegistryResponse {
            status,
            content_type,
            digest,
            location: None,
            body,
        })
    }

    fn head(&self, url: &str) -> Result<RegistryResponse, String> {
        let result = if let Some(value) = &self.authorization {
            self.agent.head(url).header(AUTHORIZATION_HEADER, value).call()
        } else {
            self.agent.head(url).call()
        };
        let response = result.map_err(|error| format!("registry HEAD transport failed: {error}"))?;
        let status = response.status().as_u16();
        let digest =
            response.headers().get(DIGEST_HEADER).and_then(|value| value.to_str().ok()).map(ToString::to_string);
        Ok(RegistryResponse {
            status,
            content_type: None,
            digest,
            location: None,
            body: Vec::new(),
        })
    }

    fn post_empty(&self, url: &str) -> Result<RegistryResponse, String> {
        let result = if let Some(value) = &self.authorization {
            self.agent.post(url).header(AUTHORIZATION_HEADER, value).send_empty()
        } else {
            self.agent.post(url).send_empty()
        };
        let response = result.map_err(|error| format!("registry POST transport failed: {error}"))?;
        let status = response.status().as_u16();
        let location = response
            .headers()
            .get(LOCATION_HEADER)
            .and_then(|value| value.to_str().ok())
            .map(ToString::to_string);
        Ok(RegistryResponse {
            status,
            content_type: None,
            digest: None,
            location,
            body: Vec::new(),
        })
    }

    fn put(&self, url: &str, content_type: &str, bytes: &[u8]) -> Result<RegistryResponse, String> {
        let result = if let Some(value) = &self.authorization {
            self.agent.put(url).header(AUTHORIZATION_HEADER, value).content_type(content_type).send(bytes)
        } else {
            self.agent.put(url).content_type(content_type).send(bytes)
        };
        let response = result.map_err(|error| format!("registry PUT transport failed: {error}"))?;
        let status = response.status().as_u16();
        let digest =
            response.headers().get(DIGEST_HEADER).and_then(|value| value.to_str().ok()).map(ToString::to_string);
        let location = response
            .headers()
            .get(LOCATION_HEADER)
            .and_then(|value| value.to_str().ok())
            .map(ToString::to_string);
        Ok(RegistryResponse {
            status,
            content_type: None,
            digest,
            location,
            body: Vec::new(),
        })
    }
}

fn expect_status(response: &RegistryResponse, expected: u16, operation: &str) -> Result<(), String> {
    if response.status != expected {
        return Err(format!("registry {operation} returned HTTP {}", response.status));
    }
    assert_ne!(expected, 0, "expected registry status must be explicit");
    assert!(!operation.is_empty(), "registry operation label must be explicit");
    Ok(())
}

fn checked_add_bytes(total: &mut u64, amount: u64) -> Result<(), String> {
    *total = total.checked_add(amount).ok_or_else(|| "registry transferred byte count overflowed".to_string())?;
    assert!(*total >= amount, "checked transfer total must cover the added amount");
    assert!(amount <= OCI_BLOB_MAX_BYTES, "one transfer amount must stay within the blob bound");
    Ok(())
}

fn ping(client: &RegistryHttpClient, target: &RegistryTarget) -> Result<(), String> {
    let response = client.get(&registry_v2_url(target), OCI_BLOB_CONTENT_TYPE, OCI_DOCUMENT_MAX_BYTES)?;
    expect_status(&response, HTTP_STATUS_OK, "API preflight")?;
    assert!(response.location.is_none(), "registry preflight must not return an upload location");
    assert!(response.body.len() as u64 <= OCI_DOCUMENT_MAX_BYTES, "registry preflight body must stay bounded");
    Ok(())
}

fn blob_exists(client: &RegistryHttpClient, target: &RegistryTarget, digest: &str) -> Result<bool, String> {
    let response = client.head(&registry_blob_url(target, digest)?)?;
    match response.status {
        HTTP_STATUS_OK => {
            if response.digest.as_deref().is_some_and(|value| value != digest) {
                return Err("registry blob existence response returned another digest".to_string());
            }
            Ok(true)
        }
        HTTP_STATUS_NOT_FOUND => Ok(false),
        _ => Err(format!("registry blob existence probe returned HTTP {}", response.status)),
    }
}

fn upload_blob(client: &RegistryHttpClient, target: &RegistryTarget, digest: &str, bytes: &[u8]) -> Result<(), String> {
    if digest != sha256_digest(bytes) {
        return Err("registry blob upload bytes do not match the requested digest".to_string());
    }
    let started = client.post_empty(&registry_upload_url(target))?;
    expect_status(&started, HTTP_STATUS_ACCEPTED, "blob upload start")?;
    let location = started
        .location
        .as_deref()
        .ok_or_else(|| "registry blob upload start omitted Location".to_string())?;
    let final_url = finalize_upload_url(FinalizeUploadInput {
        target,
        location,
        digest,
    })?;
    let completed = client.put(&final_url, OCI_BLOB_CONTENT_TYPE, bytes)?;
    expect_status(&completed, HTTP_STATUS_CREATED, "blob upload completion")?;
    if completed.digest.as_deref() != Some(digest) {
        return Err("registry blob upload completion returned another digest".to_string());
    }
    assert!(completed.location.is_some(), "completed registry blob upload must return a location");
    assert_eq!(digest, sha256_digest(bytes), "completed registry upload bytes must retain identity");
    Ok(())
}

fn publish_manifest(
    client: &RegistryHttpClient,
    target: &RegistryTarget,
    reference: &str,
    media_type: &str,
    bytes: &[u8],
) -> Result<String, String> {
    let digest = sha256_digest(bytes);
    let response = client.put(&registry_manifest_url(target, reference)?, media_type, bytes)?;
    expect_status(&response, HTTP_STATUS_CREATED, "manifest publication")?;
    if response.digest.as_deref() != Some(digest.as_str()) {
        return Err("registry manifest publication returned another digest".to_string());
    }
    assert!(!reference.is_empty(), "published manifest reference must not be empty");
    assert!(!bytes.is_empty(), "published manifest bytes must not be empty");
    Ok(digest)
}

fn get_manifest(
    client: &RegistryHttpClient,
    target: &RegistryTarget,
    reference: &str,
    media_type: &str,
    expected_digest: Option<&str>,
) -> Result<ManifestResponse, String> {
    let response = client.get(&registry_manifest_url(target, reference)?, media_type, OCI_DOCUMENT_MAX_BYTES)?;
    expect_status(&response, HTTP_STATUS_OK, "manifest fetch")?;
    if response.content_type.as_deref() != Some(media_type) {
        return Err("registry manifest response media type is missing or unsupported".to_string());
    }
    let digest = sha256_digest(&response.body);
    if response.digest.as_deref() != Some(digest.as_str()) {
        return Err("registry manifest response digest header does not match its bytes".to_string());
    }
    if expected_digest.is_some_and(|expected| expected != digest) {
        return Err("registry manifest tag resolved to an unexpected digest".to_string());
    }
    assert!(!response.body.is_empty(), "verified registry manifest body must not be empty");
    assert!(response.body.len() as u64 <= OCI_DOCUMENT_MAX_BYTES, "verified manifest must stay bounded");
    Ok(ManifestResponse {
        media_type: media_type.to_string(),
        digest,
        bytes: response.body,
    })
}

fn get_blob(
    client: &RegistryHttpClient,
    target: &RegistryTarget,
    descriptor: &OciDescriptor,
) -> Result<Vec<u8>, String> {
    if descriptor.size == 0 || descriptor.size > OCI_BLOB_MAX_BYTES {
        return Err("registry blob descriptor size is outside the named bound".to_string());
    }
    let response =
        client.get(&registry_blob_url(target, &descriptor.digest)?, OCI_BLOB_CONTENT_TYPE, descriptor.size)?;
    expect_status(&response, HTTP_STATUS_OK, "blob fetch")?;
    if response.digest.as_deref().is_some_and(|value| value != descriptor.digest) {
        return Err("registry blob response returned another digest".to_string());
    }
    if descriptor.digest != sha256_digest(&response.body) || descriptor.size != response.body.len() as u64 {
        return Err("registry blob response bytes do not match their descriptor".to_string());
    }
    assert!(!response.body.is_empty(), "verified registry blob must not be empty");
    assert_eq!(descriptor.size, response.body.len() as u64, "verified registry blob size must match");
    Ok(response.body)
}

fn load_push_plan(layout_dir: &Path) -> Result<crate::oci_registry::RegistryPushPlan, String> {
    let facts = read_layout_facts(layout_dir)?;
    let export_report_bytes =
        read_bounded_regular(&layout_dir.join(OCI_EXPORT_REPORT_FILENAME), OCI_DOCUMENT_MAX_BYTES)?;
    let input = RegistryLayoutInput {
        oci_layout_bytes: facts.oci_layout_bytes,
        index_bytes: facts.index_bytes,
        descriptor_blobs: facts.blobs,
        export_report_bytes,
    };
    let plan = build_registry_push_plan(input)?;
    assert!(!plan.blobs.is_empty(), "validated registry push plan must retain blobs");
    assert!(!plan.main_manifest_bytes.is_empty(), "validated registry push plan must retain a manifest");
    Ok(plan)
}

fn publish_blobs(
    client: &RegistryHttpClient,
    target: &RegistryTarget,
    blobs: &[crate::oci_registry::RegistryBlob],
) -> Result<PushAccounting, String> {
    let mut accounting = PushAccounting::default();
    for blob in blobs {
        if blob_exists(client, target, &blob.digest)? {
            accounting.reused_blobs = accounting.reused_blobs.saturating_add(1);
            continue;
        }
        upload_blob(client, target, &blob.digest, &blob.bytes)?;
        accounting.uploaded_blobs = accounting.uploaded_blobs.saturating_add(1);
        checked_add_bytes(&mut accounting.transferred_bytes, blob.bytes.len() as u64)?;
    }
    assert_eq!(
        accounting.uploaded_blobs.saturating_add(accounting.reused_blobs) as usize,
        blobs.len(),
        "push accounting must cover every planned blob"
    );
    assert!(accounting.uploaded_blobs > 0 || accounting.reused_blobs > 0, "push accounting must not be empty");
    Ok(accounting)
}

fn verify_published_manifests(
    client: &RegistryHttpClient,
    target: &RegistryTarget,
    plan: &crate::oci_registry::RegistryPushPlan,
) -> Result<(), String> {
    let metadata = get_manifest(
        client,
        target,
        &plan.metadata_manifest_descriptor.digest,
        OCI_ARTIFACT_MANIFEST_MEDIA_TYPE,
        Some(&plan.metadata_manifest_descriptor.digest),
    )?;
    let main = get_manifest(
        client,
        target,
        &plan.main_manifest_descriptor.digest,
        &plan.main_manifest_descriptor.media_type,
        Some(&plan.main_manifest_descriptor.digest),
    )?;
    if metadata.bytes != plan.metadata_manifest_bytes || main.bytes != plan.main_manifest_bytes {
        return Err("registry immutable manifest verification returned different bytes".to_string());
    }
    assert_eq!(metadata.digest, plan.metadata_manifest_descriptor.digest);
    assert_eq!(main.digest, plan.main_manifest_descriptor.digest);
    Ok(())
}

pub fn push_registry_layout(request: RegistryPushRequest<'_>) -> Result<OciRegistryPushReport, String> {
    let credentials = read_credentials(request.bearer_token_file)?;
    let client = RegistryHttpClient::new(&credentials);
    ping(&client, request.target)?;
    let plan = load_push_plan(request.layout_dir)?;
    let mut accounting = publish_blobs(&client, request.target, &plan.blobs)?;
    let metadata_digest = publish_manifest(
        &client,
        request.target,
        &request.target.metadata_reference,
        OCI_ARTIFACT_MANIFEST_MEDIA_TYPE,
        &plan.metadata_manifest_bytes,
    )?;
    if metadata_digest != plan.metadata_manifest_descriptor.digest {
        return Err("published Mantle metadata manifest identity drifted".to_string());
    }
    checked_add_bytes(&mut accounting.transferred_bytes, plan.metadata_manifest_bytes.len() as u64)?;
    let main_digest = publish_manifest(
        &client,
        request.target,
        &request.target.reference,
        &plan.main_manifest_descriptor.media_type,
        &plan.main_manifest_bytes,
    )?;
    if main_digest != plan.main_manifest_descriptor.digest {
        return Err("published OCI image manifest identity drifted".to_string());
    }
    checked_add_bytes(&mut accounting.transferred_bytes, plan.main_manifest_bytes.len() as u64)?;
    verify_published_manifests(&client, request.target, &plan)?;
    build_push_report(PushReportFacts {
        target: request.target,
        plan: &plan,
        uploaded_blobs: accounting.uploaded_blobs,
        reused_blobs: accounting.reused_blobs,
        transferred_bytes: accounting.transferred_bytes,
        credential_mode: credentials.mode,
    })
}

fn pull_manifests(
    client: &RegistryHttpClient,
    request: &RegistryPullRequest<'_>,
) -> Result<(ManifestResponse, ManifestResponse), String> {
    let main = get_manifest(
        client,
        request.target,
        &request.target.reference,
        crate::oci_projection::OCI_MANIFEST_MEDIA_TYPE,
        Some(request.expected_manifest_digest),
    )?;
    let metadata = get_manifest(
        client,
        request.target,
        &request.target.metadata_reference,
        OCI_ARTIFACT_MANIFEST_MEDIA_TYPE,
        Some(request.expected_metadata_manifest_digest),
    )?;
    assert_eq!(main.digest, request.expected_manifest_digest, "main tag must resolve immutably");
    assert_eq!(metadata.digest, request.expected_metadata_manifest_digest, "metadata tag must resolve immutably");
    Ok((main, metadata))
}

fn response_descriptor(response: &ManifestResponse) -> OciDescriptor {
    assert!(!response.bytes.is_empty(), "manifest response bytes must not be empty");
    assert_eq!(response.digest, sha256_digest(&response.bytes), "manifest response digest must match bytes");
    OciDescriptor {
        media_type: response.media_type.clone(),
        digest: response.digest.clone(),
        size: response.bytes.len() as u64,
        annotations: BTreeMap::new(),
        platform: None,
    }
}

fn download_pull_blobs(
    client: &RegistryHttpClient,
    target: &RegistryTarget,
    descriptors: &[OciDescriptor],
    initial_bytes: u64,
) -> Result<(BTreeMap<String, Vec<u8>>, PullAccounting), String> {
    let mut output = BTreeMap::new();
    let mut accounting = PullAccounting {
        downloaded_blobs: 0,
        downloaded_bytes: initial_bytes,
    };
    for descriptor in descriptors {
        let bytes = get_blob(client, target, descriptor)?;
        checked_add_bytes(&mut accounting.downloaded_bytes, bytes.len() as u64)?;
        accounting.downloaded_blobs = accounting.downloaded_blobs.saturating_add(1);
        output.insert(descriptor.digest.clone(), bytes);
    }
    if output.len() != descriptors.len() {
        return Err("registry pull descriptor list contained duplicate digests".to_string());
    }
    assert_eq!(accounting.downloaded_blobs as usize, output.len(), "pull accounting must cover each blob");
    assert!(accounting.downloaded_bytes >= initial_bytes, "pull byte accounting must be monotonic");
    Ok((output, accounting))
}

fn cleanup_failed_pull(output_dir: &Path, import_report_path: &Path) {
    let _remove_output_result = fs::remove_dir_all(output_dir);
    let _remove_report_result = fs::remove_file(import_report_path);
}

pub fn pull_registry_layout(request: RegistryPullRequest<'_>) -> Result<OciRegistryPullReport, String> {
    if request.output_dir.exists() || request.import_report_path.exists() {
        return Err("OCI registry pull requires absent layout and import-report destinations".to_string());
    }
    let credentials = read_credentials(request.bearer_token_file)?;
    let client = RegistryHttpClient::new(&credentials);
    ping(&client, request.target)?;
    let (main, metadata) = pull_manifests(&client, &request)?;
    let main_descriptor = response_descriptor(&main);
    let (metadata_document, _) =
        inspect_metadata_manifest(&metadata.bytes, request.expected_metadata_manifest_digest, &main_descriptor)?;
    let descriptors = pull_blob_descriptors(&main.bytes, &metadata_document)?;
    let initial_bytes = (main.bytes.len() as u64)
        .checked_add(metadata.bytes.len() as u64)
        .ok_or_else(|| "registry manifest byte accounting overflowed".to_string())?;
    let (downloaded_blobs, accounting) = download_pull_blobs(&client, request.target, &descriptors, initial_bytes)?;
    let plan = build_registry_pull_plan(RegistryPullInput {
        expected_manifest_digest: request.expected_manifest_digest.to_string(),
        main_manifest_media_type: main.media_type,
        main_manifest_bytes: main.bytes,
        metadata_manifest_digest: request.expected_metadata_manifest_digest.to_string(),
        metadata_manifest_bytes: metadata.bytes,
        downloaded_blobs,
    })?;
    publish_pulled_layout(&plan, request.output_dir)?;
    let import_report = match import_oci_layout(&ImportRequest {
        layout_dir: request.output_dir,
        report_path: request.import_report_path,
        state_dir: request.state_dir,
    }) {
        Ok(report) => report,
        Err(error) => {
            cleanup_failed_pull(request.output_dir, request.import_report_path);
            return Err(format!("admitting pulled OCI layout: {error}"));
        }
    };
    build_pull_report(PullReportFacts {
        target: request.target,
        plan: &plan,
        expected_manifest_digest: request.expected_manifest_digest,
        expected_metadata_manifest_digest: request.expected_metadata_manifest_digest,
        downloaded_blobs: accounting.downloaded_blobs,
        downloaded_bytes: accounting.downloaded_bytes,
        credential_mode: credentials.mode,
        import_report: &import_report,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn credential_reader_rejects_whitespace_and_accepts_bounded_token() {
        let temporary = tempfile::tempdir().unwrap();
        let valid = temporary.path().join("valid-token");
        let invalid = temporary.path().join("invalid-token");
        fs::write(&valid, b"gallery-token\n").unwrap();
        fs::write(&invalid, b"gallery token\n").unwrap();
        let credentials = read_credentials(Some(&valid)).expect("bounded bearer token should load");
        assert_eq!(credentials.mode, CREDENTIAL_MODE_BEARER_FILE);
        assert_eq!(credentials.authorization.as_deref(), Some("Bearer gallery-token"));
        assert!(read_credentials(Some(&invalid)).is_err());
    }

    #[test]
    fn registry_client_disables_redirects_and_ambient_proxies() {
        let credentials = RegistryCredentials {
            authorization: None,
            mode: CREDENTIAL_MODE_ANONYMOUS,
        };
        let client = RegistryHttpClient::new(&credentials);
        assert_eq!(client.agent.config().max_redirects(), 0);
        assert!(client.agent.config().proxy().is_none());
    }
}
