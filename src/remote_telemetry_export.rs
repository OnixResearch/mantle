//! Disabled-by-default Prometheus textfile and OTLP/HTTP telemetry adapters.
//!
//! This shell consumes already-redacted provider-neutral descriptors. Export
//! failures are returned as bounded health facts and never as build failures.
//!
//! r[impl operator_diagnostics.telemetry_exporter_isolation]

use std::fs;
use std::io::Read;
use std::io::Write;
use std::net::TcpStream;
use std::net::ToSocketAddrs;
use std::path::Path;
use std::path::PathBuf;
use std::time::Duration;

use crunch_build::distributed::DEFAULT_REMOTE_TELEMETRY_BATCH_SIZE;
use crunch_build::distributed::DEFAULT_REMOTE_TELEMETRY_EVENT_CAPACITY;
use crunch_build::distributed::RemoteMetricDescriptor;
use crunch_build::distributed::RemoteMetricKind;
use crunch_build::distributed::RemoteTelemetryEvent;
use crunch_build::distributed::RemoteTelemetryPolicy;
use crunch_build::distributed::metric_descriptor_for_event;
use serde::Deserialize;
use serde::Serialize;

pub const REMOTE_TELEMETRY_EXPORT_REPORT_SCHEMA: &str = "mantle-remote-telemetry-export-report-v1";
pub const REMOTE_TELEMETRY_EXPORT_NON_CLAIM: &str = "telemetry delivery health is diagnostic only and does not prove build success, output validity, reproducibility, or release eligibility";
pub const DEFAULT_OTLP_TIMEOUT_MS: u32 = 2_000;
pub const MAX_OTLP_TIMEOUT_MS: u32 = 30_000;
pub const MAX_OTLP_ENDPOINT_BYTES: usize = 2_048;
pub const MAX_OTLP_PAYLOAD_BYTES: usize = 1_048_576;
pub const MAX_OTLP_RESPONSE_BYTES: u64 = 4_096;
pub const MAX_OTLP_RESOLVED_ADDRESSES: u32 = 8;
pub const PROMETHEUS_TEMP_CREATE_ATTEMPTS_MAX: u32 = 8;
const PROMETHEUS_TEMP_RANDOM_BYTES: usize = 16;
const OTLP_HTTP_CONTENT_TYPE: &str = "application/json";
const OTLP_METRICS_PATH_DEFAULT: &str = "/v1/metrics";
const HTTP_STATUS_SUCCESS_MIN: u16 = 200;
const HTTP_STATUS_SUCCESS_MAX_EXCLUSIVE: u16 = 300;
#[cfg(target_os = "linux")]
const OPENAT2_RESOLVE_NO_SYMLINKS: u64 = 0x04;

const _: () = {
    assert!(DEFAULT_OTLP_TIMEOUT_MS > 0, "default OTLP timeout must be positive");
    assert!(DEFAULT_OTLP_TIMEOUT_MS <= MAX_OTLP_TIMEOUT_MS, "default OTLP timeout must stay within policy");
};
#[cfg(target_os = "linux")]
const _: () = assert!(OPENAT2_RESOLVE_NO_SYMLINKS > 0, "openat2 no-symlink policy must be active");

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemotePrometheusConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "no_optional_value", skip_serializing_if = "Option::is_none")]
    pub textfile_path: Option<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemoteOtlpConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "no_optional_value", skip_serializing_if = "Option::is_none")]
    pub endpoint: Option<String>,
    #[serde(default = "default_otlp_timeout_ms")]
    pub timeout_ms: u32,
}

impl Default for RemoteOtlpConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            endpoint: None,
            timeout_ms: DEFAULT_OTLP_TIMEOUT_MS,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemoteTelemetryExportConfig {
    #[serde(default = "default_remote_telemetry_policy")]
    pub telemetry: RemoteTelemetryPolicy,
    #[serde(default = "RemotePrometheusConfig::default")]
    pub prometheus: RemotePrometheusConfig,
    #[serde(default = "RemoteOtlpConfig::default")]
    pub otlp: RemoteOtlpConfig,
}

impl Default for RemoteTelemetryExportConfig {
    fn default() -> Self {
        Self {
            telemetry: default_remote_telemetry_policy(),
            prometheus: RemotePrometheusConfig::default(),
            otlp: RemoteOtlpConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RemoteTelemetryAdapterStatus {
    Disabled,
    Succeeded,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemoteTelemetryAdapterHealth {
    pub status: RemoteTelemetryAdapterStatus,
    pub reason_code: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemoteTelemetryExportReport {
    pub schema: String,
    pub events_received: u64,
    pub metrics_admitted: u64,
    pub metrics_rejected: u64,
    pub events_dropped: u64,
    pub intake: RemoteTelemetryAdapterHealth,
    pub prometheus: RemoteTelemetryAdapterHealth,
    pub otlp: RemoteTelemetryAdapterHealth,
    pub non_claim: String,
}

#[derive(Debug)]
struct BoundedMetricAdmission {
    descriptors: Vec<RemoteMetricDescriptor>,
    events_received: u64,
    metrics_rejected: u64,
    events_dropped: u64,
    intake: RemoteTelemetryAdapterHealth,
}

pub fn validate_remote_telemetry_export_config(config: &RemoteTelemetryExportConfig) -> Result<(), String> {
    config.telemetry.validate().map_err(|reason| reason.as_str().to_string())?;
    if config.prometheus.enabled {
        let path = config
            .prometheus
            .textfile_path
            .as_deref()
            .ok_or_else(|| "remote-telemetry-prometheus-path-required".to_string())?;
        if path.as_os_str().is_empty() {
            return Err("remote-telemetry-prometheus-path-required".to_string());
        }
    }
    if config.otlp.timeout_ms == 0 || config.otlp.timeout_ms > MAX_OTLP_TIMEOUT_MS {
        return Err("remote-telemetry-otlp-timeout-invalid".to_string());
    }
    if config.otlp.enabled {
        let endpoint = config
            .otlp
            .endpoint
            .as_deref()
            .ok_or_else(|| "remote-telemetry-otlp-endpoint-required".to_string())?;
        validate_otlp_endpoint(endpoint)?;
    }
    assert!(config.otlp.timeout_ms > 0, "validated OTLP timeout must be positive");
    assert!(config.otlp.timeout_ms <= MAX_OTLP_TIMEOUT_MS, "validated OTLP timeout must stay bounded");
    Ok(())
}

pub fn export_remote_telemetry(
    config: &RemoteTelemetryExportConfig,
    events: &[RemoteTelemetryEvent],
) -> RemoteTelemetryExportReport {
    let config_error = validate_remote_telemetry_export_config(config).err();
    let mut admission = bounded_metric_admission(config, events, config_error.as_deref());
    let metrics_admitted = match checked_len_as_u64(admission.descriptors.len()) {
        Some(count) => count,
        None => {
            admission.descriptors.clear();
            admission.events_dropped = admission.events_received;
            admission.intake =
                adapter_health(RemoteTelemetryAdapterStatus::Failed, "remote-telemetry-intake-counter-overflow");
            0
        }
    };
    let prometheus = export_prometheus(config, &admission.descriptors, config_error.as_deref());
    let otlp = export_otlp(config, &admission.descriptors, config_error.as_deref());
    assert!(metrics_admitted <= admission.events_received, "admitted metrics cannot exceed received events");
    assert!(
        admission.events_dropped <= admission.events_received,
        "dropped events cannot exceed received events"
    );
    RemoteTelemetryExportReport {
        schema: REMOTE_TELEMETRY_EXPORT_REPORT_SCHEMA.to_string(),
        events_received: admission.events_received,
        metrics_admitted,
        metrics_rejected: admission.metrics_rejected,
        events_dropped: admission.events_dropped,
        intake: admission.intake,
        prometheus,
        otlp,
        non_claim: REMOTE_TELEMETRY_EXPORT_NON_CLAIM.to_string(),
    }
}

pub fn render_prometheus_metrics(descriptors: &[RemoteMetricDescriptor]) -> Result<String, String> {
    let mut rendered = String::new();
    for descriptor in descriptors {
        validate_metric_name(&descriptor.name)?;
        rendered.push_str(&descriptor.name);
        if !descriptor.labels.is_empty() {
            rendered.push('{');
            for (index, label) in descriptor.labels.iter().enumerate() {
                if index > 0 {
                    rendered.push(',');
                }
                rendered.push_str(&label.key);
                rendered.push_str("=\"");
                rendered.push_str(&escape_prometheus_label(&label.value));
                rendered.push('"');
            }
            rendered.push('}');
        }
        rendered.push(' ');
        rendered.push_str(&descriptor.value.to_string());
        rendered.push('\n');
    }
    if rendered.len() > MAX_OTLP_PAYLOAD_BYTES {
        return Err("remote-telemetry-prometheus-payload-oversized".to_string());
    }
    assert!(rendered.len() <= MAX_OTLP_PAYLOAD_BYTES, "admitted Prometheus text must stay bounded");
    assert_eq!(rendered.lines().count(), descriptors.len(), "each descriptor must render one metric line");
    Ok(rendered)
}

pub fn render_otlp_metrics_json(descriptors: &[RemoteMetricDescriptor]) -> Result<Vec<u8>, String> {
    let metrics = descriptors
        .iter()
        .map(|descriptor| {
            let attributes = descriptor
                .labels
                .iter()
                .map(|label| {
                    serde_json::json!({
                        "key": label.key,
                        "value": { "stringValue": label.value },
                    })
                })
                .collect::<Vec<_>>();
            let data_point = serde_json::json!({
                "attributes": attributes,
                "asInt": descriptor.value.to_string(),
            });
            match descriptor.kind {
                RemoteMetricKind::Counter => serde_json::json!({
                    "name": descriptor.name,
                    "sum": {
                        "aggregationTemporality": 1,
                        "isMonotonic": true,
                        "dataPoints": [data_point],
                    },
                }),
                RemoteMetricKind::Gauge => serde_json::json!({
                    "name": descriptor.name,
                    "gauge": { "dataPoints": [data_point] },
                }),
            }
        })
        .collect::<Vec<_>>();
    let payload = serde_json::to_vec(&serde_json::json!({
        "resourceMetrics": [{
            "resource": {
                "attributes": [{
                    "key": "service.name",
                    "value": { "stringValue": "mantle" },
                }],
            },
            "scopeMetrics": [{
                "scope": { "name": "mantle.remote" },
                "metrics": metrics,
            }],
        }],
    }))
    .map_err(|error| format!("remote-telemetry-otlp-serialize-failed: {error}"))?;
    if payload.len() > MAX_OTLP_PAYLOAD_BYTES {
        return Err("remote-telemetry-otlp-payload-oversized".to_string());
    }
    assert!(!payload.is_empty(), "OTLP JSON payload must not be empty");
    assert!(payload.len() <= MAX_OTLP_PAYLOAD_BYTES, "OTLP JSON payload must stay bounded");
    Ok(payload)
}

fn bounded_metric_admission(
    config: &RemoteTelemetryExportConfig,
    events: &[RemoteTelemetryEvent],
    config_error: Option<&str>,
) -> BoundedMetricAdmission {
    let Some(events_received) = checked_len_as_u64(events.len()) else {
        return failed_metric_admission(0, "remote-telemetry-intake-counter-overflow");
    };
    let Some(capacity) = usize::try_from(config.telemetry.event_capacity).ok() else {
        return failed_metric_admission(events_received, "remote-telemetry-intake-capacity-invalid");
    };
    let Some(batch_size) = usize::try_from(config.telemetry.batch_size).ok() else {
        return failed_metric_admission(events_received, "remote-telemetry-intake-batch-size-invalid");
    };
    if let Some(reason) = config_error {
        return failed_metric_admission(events_received, reason);
    }
    let bounded_count = events.len().min(capacity).min(batch_size);
    assert!(bounded_count <= events.len(), "bounded intake cannot exceed available events");
    assert!(bounded_count <= capacity, "bounded intake cannot exceed configured capacity");
    let mut descriptors = Vec::with_capacity(bounded_count);
    let mut metrics_rejected = 0_u64;
    for event in events.iter().take(bounded_count) {
        match metric_descriptor_for_event(event) {
            Ok(descriptor) => descriptors.push(descriptor),
            Err(_) => {
                let Some(next) = metrics_rejected.checked_add(1) else {
                    return failed_metric_admission(events_received, "remote-telemetry-intake-counter-overflow");
                };
                metrics_rejected = next;
            }
        }
    }
    let events_dropped = match events.len().checked_sub(bounded_count).and_then(|count| u64::try_from(count).ok()) {
        Some(count) => count,
        None => {
            return failed_metric_admission(events_received, "remote-telemetry-intake-counter-overflow");
        }
    };
    let intake = if events_dropped > 0 {
        adapter_health(RemoteTelemetryAdapterStatus::Failed, "remote-telemetry-intake-capacity-exceeded")
    } else if metrics_rejected > 0 {
        adapter_health(RemoteTelemetryAdapterStatus::Failed, "remote-telemetry-intake-event-rejected")
    } else {
        adapter_health(RemoteTelemetryAdapterStatus::Succeeded, "remote-telemetry-intake-admitted")
    };
    BoundedMetricAdmission {
        descriptors,
        events_received,
        metrics_rejected,
        events_dropped,
        intake,
    }
}

fn failed_metric_admission(events_received: u64, reason: &str) -> BoundedMetricAdmission {
    BoundedMetricAdmission {
        descriptors: Vec::new(),
        events_received,
        metrics_rejected: 0,
        events_dropped: events_received,
        intake: adapter_health(RemoteTelemetryAdapterStatus::Failed, reason),
    }
}

fn checked_len_as_u64(length: usize) -> Option<u64> {
    u64::try_from(length).ok()
}

fn export_prometheus(
    config: &RemoteTelemetryExportConfig,
    descriptors: &[RemoteMetricDescriptor],
    config_error: Option<&str>,
) -> RemoteTelemetryAdapterHealth {
    if !config.prometheus.enabled {
        return adapter_health(RemoteTelemetryAdapterStatus::Disabled, "remote-telemetry-prometheus-disabled");
    }
    if let Some(reason) = config_error {
        return adapter_health(RemoteTelemetryAdapterStatus::Failed, reason);
    }
    let Some(path) = config.prometheus.textfile_path.as_deref() else {
        return adapter_health(RemoteTelemetryAdapterStatus::Failed, "remote-telemetry-prometheus-path-required");
    };
    let result = render_prometheus_metrics(descriptors)
        .and_then(|rendered| write_prometheus_textfile(path, rendered.as_bytes()));
    match result {
        Ok(()) => adapter_health(RemoteTelemetryAdapterStatus::Succeeded, "remote-telemetry-prometheus-exported"),
        Err(_) => adapter_health(RemoteTelemetryAdapterStatus::Failed, "remote-telemetry-prometheus-export-failed"),
    }
}

fn export_otlp(
    config: &RemoteTelemetryExportConfig,
    descriptors: &[RemoteMetricDescriptor],
    config_error: Option<&str>,
) -> RemoteTelemetryAdapterHealth {
    if !config.otlp.enabled {
        return adapter_health(RemoteTelemetryAdapterStatus::Disabled, "remote-telemetry-otlp-disabled");
    }
    if let Some(reason) = config_error {
        return adapter_health(RemoteTelemetryAdapterStatus::Failed, reason);
    }
    assert!(config.telemetry.batch_size > 0, "validated OTLP batch size must be positive");
    assert!(config.otlp.timeout_ms > 0, "validated OTLP timeout must be positive");
    let Some(endpoint) = config.otlp.endpoint.as_deref() else {
        return adapter_health(RemoteTelemetryAdapterStatus::Failed, "remote-telemetry-otlp-endpoint-required");
    };
    let result = export_otlp_batches(OtlpBatchRequest {
        endpoint,
        timeout_ms: config.otlp.timeout_ms,
        batch_size: config.telemetry.batch_size,
        descriptors,
    });
    match result {
        Ok(()) => adapter_health(RemoteTelemetryAdapterStatus::Succeeded, "remote-telemetry-otlp-exported"),
        Err(error) => adapter_health(RemoteTelemetryAdapterStatus::Failed, bounded_otlp_error_reason(&error)),
    }
}

#[cfg(target_os = "linux")]
fn write_prometheus_textfile(path: &Path, bytes: &[u8]) -> Result<(), String> {
    use std::ffi::CString;
    use std::os::fd::AsRawFd;
    use std::os::fd::FromRawFd;
    use std::os::unix::ffi::OsStrExt;

    let parent = path.parent().ok_or_else(|| "remote-telemetry-prometheus-parent-missing".to_string())?;
    let destination_name =
        path.file_name().ok_or_else(|| "remote-telemetry-prometheus-file-name-missing".to_string())?;
    assert!(!parent.as_os_str().is_empty(), "Prometheus destination parent must not be empty");
    assert!(!destination_name.is_empty(), "Prometheus destination name must not be empty");
    let destination_name = CString::new(destination_name.as_bytes())
        .map_err(|_| "remote-telemetry-prometheus-file-name-invalid".to_string())?;
    let directory = open_directory_no_symlinks(parent)?;
    let directory_fd = directory.as_raw_fd();
    for _ in 0..PROMETHEUS_TEMP_CREATE_ATTEMPTS_MAX {
        let random: [u8; PROMETHEUS_TEMP_RANDOM_BYTES] = rand::random();
        let temp_name = format!(".mantle-remote-telemetry-{}.tmp", data_encoding::HEXLOWER.encode(&random));
        let temp_name =
            CString::new(temp_name).map_err(|_| "remote-telemetry-prometheus-temp-name-invalid".to_string())?;
        let raw_fd = unsafe {
            libc::openat(
                directory_fd,
                temp_name.as_ptr(),
                libc::O_WRONLY | libc::O_CREAT | libc::O_EXCL | libc::O_NOFOLLOW | libc::O_CLOEXEC,
                libc::S_IRUSR | libc::S_IWUSR,
            )
        };
        if raw_fd < 0 {
            let error = std::io::Error::last_os_error();
            if error.kind() == std::io::ErrorKind::AlreadyExists {
                continue;
            }
            return Err(format!("remote-telemetry-prometheus-temp-create-failed: {error}"));
        }
        let mut temp = unsafe { fs::File::from_raw_fd(raw_fd) };
        let write_result = temp
            .write_all(bytes)
            .and_then(|()| temp.sync_all())
            .map_err(|error| format!("remote-telemetry-prometheus-write-sync-failed: {error}"));
        drop(temp);
        if let Err(error) = write_result {
            unlink_relative_file(directory_fd, &temp_name);
            return Err(error);
        }
        let rename_status =
            unsafe { libc::renameat(directory_fd, temp_name.as_ptr(), directory_fd, destination_name.as_ptr()) };
        if rename_status != 0 {
            let error = std::io::Error::last_os_error();
            unlink_relative_file(directory_fd, &temp_name);
            return Err(format!("remote-telemetry-prometheus-publish-failed: {error}"));
        }
        return directory
            .sync_all()
            .map_err(|error| format!("remote-telemetry-prometheus-directory-sync-failed: {error}"));
    }
    Err("remote-telemetry-prometheus-temp-create-attempts-exhausted".to_string())
}

#[cfg(target_os = "linux")]
fn open_directory_no_symlinks(path: &Path) -> Result<fs::File, String> {
    use std::ffi::CString;
    use std::os::fd::FromRawFd;
    use std::os::unix::ffi::OsStrExt;

    #[repr(C)]
    struct OpenHow {
        flags: u64,
        mode: u64,
        resolve: u64,
    }

    let path = CString::new(path.as_os_str().as_bytes())
        .map_err(|_| "remote-telemetry-prometheus-parent-invalid".to_string())?;
    assert!(!path.as_bytes().is_empty(), "Prometheus parent path must not be empty");
    let how = OpenHow {
        flags: u64::try_from(libc::O_RDONLY | libc::O_DIRECTORY | libc::O_CLOEXEC)
            .map_err(|_| "remote-telemetry-prometheus-open-flags-invalid".to_string())?,
        mode: 0,
        resolve: OPENAT2_RESOLVE_NO_SYMLINKS,
    };
    let raw_fd = unsafe {
        libc::syscall(libc::SYS_openat2, libc::AT_FDCWD, path.as_ptr(), &how, std::mem::size_of::<OpenHow>())
    };
    if raw_fd < 0 {
        return Err(format!(
            "remote-telemetry-prometheus-parent-no-follow-open-failed: {}",
            std::io::Error::last_os_error()
        ));
    }
    let raw_fd = i32::try_from(raw_fd).map_err(|_| "remote-telemetry-prometheus-parent-fd-invalid".to_string())?;
    Ok(unsafe { fs::File::from_raw_fd(raw_fd) })
}

#[cfg(target_os = "linux")]
fn unlink_relative_file(directory_fd: std::os::fd::RawFd, name: &std::ffi::CStr) {
    let _ = unsafe { libc::unlinkat(directory_fd, name.as_ptr(), 0) };
}

#[cfg(not(target_os = "linux"))]
fn write_prometheus_textfile(_path: &Path, _bytes: &[u8]) -> Result<(), String> {
    Err("remote-telemetry-prometheus-unsupported-platform".to_string())
}

struct OtlpBatchRequest<'a> {
    endpoint: &'a str,
    timeout_ms: u32,
    batch_size: u32,
    descriptors: &'a [RemoteMetricDescriptor],
}

fn export_otlp_batches(request: OtlpBatchRequest<'_>) -> Result<(), String> {
    let batch_event_count =
        usize::try_from(request.batch_size).map_err(|_| "remote-telemetry-otlp-batch-size-invalid".to_string())?;
    if batch_event_count == 0 {
        return Err("remote-telemetry-otlp-batch-size-invalid".to_string());
    }
    if request.descriptors.is_empty() {
        let payload = render_otlp_metrics_json(request.descriptors)?;
        return post_otlp_http(request.endpoint, request.timeout_ms, &payload);
    }
    for batch in request.descriptors.chunks(batch_event_count) {
        let payload = render_otlp_metrics_json(batch)?;
        post_otlp_http(request.endpoint, request.timeout_ms, &payload)?;
    }
    Ok(())
}

fn post_otlp_http(endpoint: &str, timeout_ms: u32, payload: &[u8]) -> Result<(), String> {
    let url = validate_otlp_endpoint(endpoint)?;
    let host = url.host_str().ok_or_else(|| "remote-telemetry-otlp-host-missing".to_string())?;
    let service_number = url.port_or_known_default().ok_or_else(|| "remote-telemetry-otlp-port-missing".to_string())?;
    let request_timeout_ms = Duration::from_millis(u64::from(timeout_ms));
    let address_count_max = usize::try_from(MAX_OTLP_RESOLVED_ADDRESSES)
        .map_err(|_| "remote-telemetry-otlp-address-limit-invalid".to_string())?;
    let addresses = (host, service_number)
        .to_socket_addrs()
        .map_err(|_| "remote-telemetry-otlp-resolve-failed".to_string())?
        .take(address_count_max)
        .collect::<Vec<_>>();
    if addresses.is_empty() {
        return Err("remote-telemetry-otlp-resolve-empty".to_string());
    }
    if addresses.iter().any(|address| !address.ip().is_loopback()) {
        return Err("remote-telemetry-otlp-resolved-non-loopback-rejected".to_string());
    }
    assert!(addresses.len() <= address_count_max, "resolved OTLP addresses must stay bounded");
    assert!(
        addresses.iter().all(|address| address.ip().is_loopback()),
        "admitted OTLP addresses must be loopback"
    );
    let mut stream = addresses
        .iter()
        .find_map(|address| TcpStream::connect_timeout(address, request_timeout_ms).ok())
        .ok_or_else(|| "remote-telemetry-otlp-connect-failed".to_string())?;
    stream
        .set_read_timeout(Some(request_timeout_ms))
        .and_then(|()| stream.set_write_timeout(Some(request_timeout_ms)))
        .map_err(|_| "remote-telemetry-otlp-timeout-config-failed".to_string())?;
    let path = if url.path().is_empty() || url.path() == "/" {
        OTLP_METRICS_PATH_DEFAULT
    } else {
        url.path()
    };
    let host_header = if url.port().is_some() {
        format!("{host}:{service_number}")
    } else {
        host.to_string()
    };
    let request_head = format!(
        "POST {path} HTTP/1.1\r\nHost: {host_header}\r\nContent-Type: {OTLP_HTTP_CONTENT_TYPE}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        payload.len()
    );
    stream
        .write_all(request_head.as_bytes())
        .and_then(|()| stream.write_all(payload))
        .and_then(|()| stream.flush())
        .map_err(|_| "remote-telemetry-otlp-send-failed".to_string())?;
    let mut response = Vec::new();
    Read::by_ref(&mut stream)
        .take(MAX_OTLP_RESPONSE_BYTES)
        .read_to_end(&mut response)
        .map_err(|_| "remote-telemetry-otlp-response-read-failed".to_string())?;
    let status_line = response
        .split(|byte| *byte == b'\n')
        .next()
        .and_then(|line| std::str::from_utf8(line).ok())
        .ok_or_else(|| "remote-telemetry-otlp-response-malformed".to_string())?;
    let status = status_line
        .split_ascii_whitespace()
        .nth(1)
        .and_then(|value| value.parse::<u16>().ok())
        .ok_or_else(|| "remote-telemetry-otlp-status-malformed".to_string())?;
    if !(HTTP_STATUS_SUCCESS_MIN..HTTP_STATUS_SUCCESS_MAX_EXCLUSIVE).contains(&status) {
        return Err("remote-telemetry-otlp-status-rejected".to_string());
    }
    Ok(())
}

fn validate_otlp_endpoint(endpoint: &str) -> Result<url::Url, String> {
    if endpoint.is_empty() || endpoint.len() > MAX_OTLP_ENDPOINT_BYTES {
        return Err("remote-telemetry-otlp-endpoint-bounds".to_string());
    }
    let url = url::Url::parse(endpoint).map_err(|_| "remote-telemetry-otlp-endpoint-invalid".to_string())?;
    if url.scheme() != "http" {
        return Err("remote-telemetry-otlp-http-only".to_string());
    }
    if !url.username().is_empty() || url.password().is_some() {
        return Err("remote-telemetry-otlp-credentials-or-query-rejected".to_string());
    }
    if url.query().is_some() || url.fragment().is_some() {
        return Err("remote-telemetry-otlp-credentials-or-query-rejected".to_string());
    }
    let host = url.host().ok_or_else(|| "remote-telemetry-otlp-host-missing".to_string())?;
    let is_loopback = match host {
        url::Host::Ipv4(address) => address.is_loopback(),
        url::Host::Ipv6(address) => address.is_loopback(),
        url::Host::Domain(_) => false,
    };
    if !is_loopback {
        return Err("remote-telemetry-otlp-plaintext-non-loopback-rejected".to_string());
    }
    assert_eq!(url.scheme(), "http", "validated OTLP endpoint must use HTTP");
    assert!(is_loopback, "validated plaintext OTLP endpoint must be loopback");
    Ok(url)
}

fn bounded_otlp_error_reason(error: &str) -> &str {
    if error.starts_with("remote-telemetry-otlp-") && error.len() <= MAX_REMOTE_METRIC_REASON_BYTES {
        return error;
    }
    "remote-telemetry-otlp-export-failed"
}

const MAX_REMOTE_METRIC_REASON_BYTES: usize = 128;

fn adapter_health(status: RemoteTelemetryAdapterStatus, reason_code: &str) -> RemoteTelemetryAdapterHealth {
    RemoteTelemetryAdapterHealth {
        status,
        reason_code: reason_code.to_string(),
    }
}

fn validate_metric_name(name: &str) -> Result<(), String> {
    if name.is_empty() || !name.bytes().all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_') {
        return Err("remote-telemetry-prometheus-metric-name-invalid".to_string());
    }
    Ok(())
}

fn escape_prometheus_label(value: &str) -> String {
    value.replace('\\', "\\\\").replace('\n', "\\n").replace('"', "\\\"")
}

fn no_optional_value<T>() -> Option<T> {
    None
}

fn default_remote_telemetry_policy() -> RemoteTelemetryPolicy {
    RemoteTelemetryPolicy {
        event_capacity: DEFAULT_REMOTE_TELEMETRY_EVENT_CAPACITY,
        batch_size: DEFAULT_REMOTE_TELEMETRY_BATCH_SIZE,
    }
}

fn default_otlp_timeout_ms() -> u32 {
    DEFAULT_OTLP_TIMEOUT_MS
}

#[cfg(test)]
mod tests {
    use std::net::TcpListener;
    use std::thread;

    use crunch_build::distributed::RemoteTelemetryCapabilityClass;
    use crunch_build::distributed::RemoteTelemetryCategory;
    use crunch_build::distributed::RemoteTelemetryMeasurementKind;
    use crunch_build::distributed::RemoteTelemetryPhaseClass;
    use crunch_build::distributed::RemoteTelemetryReasonClass;
    use crunch_build::distributed::RemoteTelemetryResultClass;
    use crunch_build::distributed::RemoteTelemetryRetryClass;
    use crunch_build::distributed::RemoteTelemetryRouteClass;
    use crunch_build::distributed::RemoteTelemetryTransferClass;
    use crunch_build::distributed::remote_telemetry_event;

    use super::*;

    const TEST_HTTP_RESPONSE: &[u8] = b"HTTP/1.1 200 OK\r\nContent-Length: 0\r\nConnection: close\r\n\r\n";
    const TEST_HTTP_HEADER_DELIMITER: &[u8] = b"\r\n\r\n";
    const TEST_REQUEST_BUFFER_BYTES: usize = 4_096;
    const TEST_REQUEST_READ_ATTEMPTS_MAX: u32 = 16;
    const TEST_UNAVAILABLE_PORT: u16 = 1;

    fn event() -> RemoteTelemetryEvent {
        remote_telemetry_event(
            RemoteTelemetryCategory::Queue,
            RemoteTelemetryPhaseClass::Queued,
            RemoteTelemetryResultClass::Accepted,
            RemoteTelemetryRouteClass::RemoteStdio,
            RemoteTelemetryRetryClass::None,
            RemoteTelemetryTransferClass::None,
            RemoteTelemetryReasonClass::QueueAdmitted,
            RemoteTelemetryCapabilityClass::Compatible,
            RemoteTelemetryMeasurementKind::Occurrences,
            1,
        )
        .unwrap()
    }

    #[test]
    fn adapters_are_disabled_by_default() {
        let config = RemoteTelemetryExportConfig::default();
        let report = export_remote_telemetry(&config, &[event()]);

        assert_eq!(report.metrics_admitted, 1);
        assert_eq!(report.metrics_rejected, 0);
        assert_eq!(report.events_dropped, 0);
        assert_eq!(report.intake.status, RemoteTelemetryAdapterStatus::Succeeded);
        assert_eq!(report.prometheus.status, RemoteTelemetryAdapterStatus::Disabled);
        assert_eq!(report.otlp.status, RemoteTelemetryAdapterStatus::Disabled);
    }

    #[test]
    fn prometheus_adapter_writes_only_bounded_class_labels() {
        let state = tempfile::tempdir().unwrap();
        let collector = state.path().join("collector");
        fs::create_dir(&collector).unwrap();
        let path = collector.join("mantle.prom");
        let config = RemoteTelemetryExportConfig {
            prometheus: RemotePrometheusConfig {
                enabled: true,
                textfile_path: Some(path.clone()),
            },
            ..RemoteTelemetryExportConfig::default()
        };
        let report = export_remote_telemetry(&config, &[event()]);
        let rendered = fs::read_to_string(path).unwrap();

        assert_eq!(report.prometheus.status, RemoteTelemetryAdapterStatus::Succeeded);
        assert!(rendered.contains("mantle_remote_queue_total"));
        assert!(rendered.contains("category=\"queue\""));
        assert!(!rendered.contains("job_id"));
        assert!(!rendered.contains("trace"));
    }

    #[test]
    fn event_capacity_bounds_admission_and_reports_drops() {
        let config = RemoteTelemetryExportConfig {
            telemetry: RemoteTelemetryPolicy {
                event_capacity: 1,
                batch_size: 1,
            },
            ..RemoteTelemetryExportConfig::default()
        };
        let report = export_remote_telemetry(&config, &[event(), event(), event()]);

        assert_eq!(report.events_received, 3);
        assert_eq!(report.metrics_admitted, 1);
        assert_eq!(report.metrics_rejected, 0);
        assert_eq!(report.events_dropped, 2);
        assert_eq!(report.intake.status, RemoteTelemetryAdapterStatus::Failed);
        assert_eq!(report.intake.reason_code, "remote-telemetry-intake-capacity-exceeded");
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn prometheus_adapter_rejects_symlinked_parent_before_writing() {
        use std::os::unix::fs::symlink;

        let state = tempfile::tempdir().unwrap();
        let real_parent = state.path().join("real-collector");
        let linked_parent = state.path().join("linked-collector");
        fs::create_dir(&real_parent).unwrap();
        symlink(&real_parent, &linked_parent).unwrap();
        let config = RemoteTelemetryExportConfig {
            prometheus: RemotePrometheusConfig {
                enabled: true,
                textfile_path: Some(linked_parent.join("mantle.prom")),
            },
            ..RemoteTelemetryExportConfig::default()
        };
        let report = export_remote_telemetry(&config, &[event()]);

        assert_eq!(report.prometheus.status, RemoteTelemetryAdapterStatus::Failed);
        assert_eq!(report.prometheus.reason_code, "remote-telemetry-prometheus-export-failed");
        assert!(!real_parent.join("mantle.prom").exists());
    }

    #[test]
    fn otlp_adapter_posts_bounded_json_to_opted_in_collector() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let server = thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let request = read_complete_test_http_request(&mut stream);
            stream.write_all(TEST_HTTP_RESPONSE).unwrap();
            request
        });
        let config = RemoteTelemetryExportConfig {
            otlp: RemoteOtlpConfig {
                enabled: true,
                endpoint: Some(format!("http://{address}/v1/metrics")),
                timeout_ms: DEFAULT_OTLP_TIMEOUT_MS,
            },
            ..RemoteTelemetryExportConfig::default()
        };
        let report = export_remote_telemetry(&config, &[event()]);
        let request = String::from_utf8(server.join().unwrap()).unwrap();

        assert_eq!(report.otlp.status, RemoteTelemetryAdapterStatus::Succeeded, "{}", report.otlp.reason_code);
        assert!(request.starts_with("POST /v1/metrics HTTP/1.1"));
        assert!(request.contains("mantle_remote_queue_total"));
        assert!(!request.contains("job_id"));
    }

    fn read_complete_test_http_request(stream: &mut TcpStream) -> Vec<u8> {
        let mut request = Vec::new();
        for _ in 0..TEST_REQUEST_READ_ATTEMPTS_MAX {
            let mut chunk = [0_u8; TEST_REQUEST_BUFFER_BYTES];
            let count = stream.read(&mut chunk).unwrap();
            assert!(count > 0);
            request.extend_from_slice(&chunk[..count]);
            let Some(header_end) = request
                .windows(TEST_HTTP_HEADER_DELIMITER.len())
                .position(|window| window == TEST_HTTP_HEADER_DELIMITER)
            else {
                continue;
            };
            let body_start = header_end + TEST_HTTP_HEADER_DELIMITER.len();
            let headers = std::str::from_utf8(&request[..header_end]).unwrap();
            let content_length = headers
                .lines()
                .find_map(|line| line.strip_prefix("Content-Length: "))
                .and_then(|value| value.parse::<usize>().ok())
                .unwrap();
            let request_end = body_start.checked_add(content_length).unwrap();
            if request.len() >= request_end {
                return request;
            }
        }
        panic!("bounded test server did not receive the complete request")
    }

    #[test]
    fn exporter_outage_is_reported_without_returning_a_build_error() {
        let config = RemoteTelemetryExportConfig {
            otlp: RemoteOtlpConfig {
                enabled: true,
                endpoint: Some(format!("http://127.0.0.1:{TEST_UNAVAILABLE_PORT}/v1/metrics")),
                timeout_ms: 1,
            },
            ..RemoteTelemetryExportConfig::default()
        };
        let report = export_remote_telemetry(&config, &[event()]);

        assert_eq!(report.metrics_admitted, 1);
        assert_eq!(report.otlp.status, RemoteTelemetryAdapterStatus::Failed);
        assert_eq!(report.otlp.reason_code, "remote-telemetry-otlp-connect-failed");
        assert_eq!(report.non_claim, REMOTE_TELEMETRY_EXPORT_NON_CLAIM);
    }

    #[test]
    fn credentials_queries_https_and_missing_destinations_fail_validation() {
        let invalid_endpoints = [
            "https://collector.example/v1/metrics",
            "http://user:secret@127.0.0.1/v1/metrics",
            "http://127.0.0.1/v1/metrics?token=secret",
            "http://192.0.2.1/v1/metrics",
            "http://collector.example/v1/metrics",
        ];
        for endpoint in invalid_endpoints {
            let config = RemoteTelemetryExportConfig {
                otlp: RemoteOtlpConfig {
                    enabled: true,
                    endpoint: Some(endpoint.to_string()),
                    timeout_ms: DEFAULT_OTLP_TIMEOUT_MS,
                },
                ..RemoteTelemetryExportConfig::default()
            };
            assert!(validate_remote_telemetry_export_config(&config).is_err());
        }
        let prometheus = RemoteTelemetryExportConfig {
            prometheus: RemotePrometheusConfig {
                enabled: true,
                textfile_path: None,
            },
            ..RemoteTelemetryExportConfig::default()
        };
        assert_eq!(
            validate_remote_telemetry_export_config(&prometheus),
            Err("remote-telemetry-prometheus-path-required".to_string())
        );
    }
}
