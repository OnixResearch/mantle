//! Pure provider-neutral remote execution telemetry and metric admission.
//!
//! Exporter queues, clocks, sockets, HTTP, and rendering belong to shell code.
//!
//! r[impl operator_diagnostics.remote_execution_telemetry]

use serde::Deserialize;
use serde::Serialize;

use crate::scheduling::PriorityDecisionEvidence;
use crate::scheduling::ResourceFitClass;

pub const REMOTE_TELEMETRY_EVENT_SCHEMA: &str = "mantle-remote-telemetry-event-v1";
pub const REMOTE_TELEMETRY_NON_CLAIM: &str = "diagnostic runtime events do not prove compiler correctness, source reproducibility, release eligibility, CI success, or physical-target determinism";
pub const DEFAULT_REMOTE_TELEMETRY_EVENT_CAPACITY: u32 = 1_024;
pub const DEFAULT_REMOTE_TELEMETRY_BATCH_SIZE: u32 = 128;
pub const MAX_REMOTE_TELEMETRY_EVENT_CAPACITY: u32 = 16_384;
pub const MAX_REMOTE_TELEMETRY_BATCH_SIZE: u32 = 1_024;
pub const MAX_REMOTE_METRIC_LABELS: u32 = 8;
pub const MAX_REMOTE_METRIC_LABEL_BYTES: usize = 64;
const SENSITIVE_METRIC_KEY_MARKERS: [&str; 15] = [
    "id",
    "path",
    "output",
    "trace",
    "bearer",
    "token",
    "key",
    "credential",
    "secret",
    "error",
    "ticket",
    "job",
    "attempt",
    "worker",
    "provider",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RemoteTelemetryCategory {
    Route,
    Queue,
    Assignment,
    RetryFencing,
    Execution,
    Transfer,
    Admission,
    Publication,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RemoteTelemetryPhaseClass {
    Planning,
    Queued,
    Assigned,
    Running,
    Transferring,
    Admitting,
    Publishing,
    Terminal,
    NotApplicable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RemoteTelemetryResultClass {
    Accepted,
    Rejected,
    Succeeded,
    Failed,
    Retried,
    Dropped,
    Pending,
    NotApplicable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RemoteTelemetryRouteClass {
    Local,
    RemoteStdio,
    RemoteSshStdio,
    RemoteP2p,
    Substitute,
    PreflightError,
    NotApplicable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RemoteTelemetryRetryClass {
    None,
    CurrentFence,
    Retryable,
    Terminal,
    PolicyDenied,
    StaleFence,
    UnknownFence,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RemoteTelemetryTransferClass {
    None,
    Full,
    Delta,
    Streaming,
    Fallback,
    NotAvailable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RemoteTelemetryReasonClass {
    RouteSelected,
    RouteIneligible,
    QueueAdmitted,
    PrioritySelected,
    WorkerAssigned,
    AttemptTransition,
    FenceAccepted,
    RetryAllowed,
    RetryDenied,
    FenceRejected,
    ExecutionCompleted,
    ExecutionFailed,
    TransferDemand,
    TransferCredit,
    TransferResumed,
    TransferCutoff,
    TransferCompleted,
    TransferFallback,
    OutputAdmitted,
    OutputRejected,
    Published,
    PublicationFailed,
    ExporterBackpressure,
    ExporterUnavailable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RemoteTelemetryCapabilityClass {
    Exact,
    Compatible,
    Degraded,
    Ineligible,
    Unknown,
    NotApplicable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RemoteTelemetryMeasurementKind {
    Occurrences,
    QueueDepth,
    CandidateCount,
    AttemptCount,
    Bytes,
    Objects,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemoteTelemetryEvent {
    pub schema: String,
    pub category: RemoteTelemetryCategory,
    pub phase: RemoteTelemetryPhaseClass,
    pub result: RemoteTelemetryResultClass,
    pub route: RemoteTelemetryRouteClass,
    pub retry: RemoteTelemetryRetryClass,
    pub transfer: RemoteTelemetryTransferClass,
    pub reason: RemoteTelemetryReasonClass,
    pub capability: RemoteTelemetryCapabilityClass,
    pub measurement: RemoteTelemetryMeasurementKind,
    pub value: u64,
    pub non_claim: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RemoteTelemetryEventFields {
    pub category: RemoteTelemetryCategory,
    pub phase: RemoteTelemetryPhaseClass,
    pub result: RemoteTelemetryResultClass,
    pub route: RemoteTelemetryRouteClass,
    pub retry: RemoteTelemetryRetryClass,
    pub transfer: RemoteTelemetryTransferClass,
    pub reason: RemoteTelemetryReasonClass,
    pub capability: RemoteTelemetryCapabilityClass,
    pub measurement: RemoteTelemetryMeasurementKind,
    pub value: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemoteTelemetryPolicy {
    pub event_capacity: u32,
    pub batch_size: u32,
}

impl Default for RemoteTelemetryPolicy {
    fn default() -> Self {
        Self {
            event_capacity: DEFAULT_REMOTE_TELEMETRY_EVENT_CAPACITY,
            batch_size: DEFAULT_REMOTE_TELEMETRY_BATCH_SIZE,
        }
    }
}

impl RemoteTelemetryPolicy {
    pub fn validate(self) -> Result<(), RemoteTelemetryReasonCode> {
        if self.event_capacity == 0 || self.event_capacity > MAX_REMOTE_TELEMETRY_EVENT_CAPACITY {
            return Err(RemoteTelemetryReasonCode::PolicyInvalid);
        }
        if self.batch_size == 0 || self.batch_size > MAX_REMOTE_TELEMETRY_BATCH_SIZE {
            return Err(RemoteTelemetryReasonCode::PolicyInvalid);
        }
        if self.batch_size > self.event_capacity {
            return Err(RemoteTelemetryReasonCode::PolicyInvalid);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemoteTelemetryBuffer {
    pub events: Vec<RemoteTelemetryEvent>,
    pub accepted_events: u64,
    pub dropped_events: u64,
    pub last_drop_reason: Option<RemoteTelemetryReasonCode>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RemoteMetricKind {
    Counter,
    Gauge,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemoteMetricLabel {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemoteMetricDescriptor {
    pub name: String,
    pub kind: RemoteMetricKind,
    pub labels: Vec<RemoteMetricLabel>,
    pub value: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoteMetricLabelCandidate {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RemoteTelemetryReasonCode {
    EventCanonical,
    EventSchemaUnsupported,
    EventValueInvalid,
    PolicyInvalid,
    BufferAccepted,
    BufferFullDropped,
    ArithmeticOverflow,
    LabelCountExceeded,
    LabelKeySensitive,
    LabelValueSensitive,
    LabelKeyUnsupported,
    LabelValueUnsupported,
    LabelBoundsExceeded,
}

impl RemoteTelemetryReasonCode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::EventCanonical => "remote-telemetry-event-canonical",
            Self::EventSchemaUnsupported => "remote-telemetry-event-schema-unsupported",
            Self::EventValueInvalid => "remote-telemetry-event-value-invalid",
            Self::PolicyInvalid => "remote-telemetry-policy-invalid",
            Self::BufferAccepted => "remote-telemetry-buffer-accepted",
            Self::BufferFullDropped => "remote-telemetry-buffer-full-dropped",
            Self::ArithmeticOverflow => "remote-telemetry-arithmetic-overflow",
            Self::LabelCountExceeded => "remote-metric-label-count-exceeded",
            Self::LabelKeySensitive => "remote-metric-label-key-sensitive",
            Self::LabelValueSensitive => "remote-metric-label-value-sensitive",
            Self::LabelKeyUnsupported => "remote-metric-label-key-unsupported",
            Self::LabelValueUnsupported => "remote-metric-label-value-unsupported",
            Self::LabelBoundsExceeded => "remote-metric-label-bounds-exceeded",
        }
    }
}

#[expect(
    tigerstyle::too_many_parameters,
    reason = "stable shell compatibility wrapper delegates immediately to named telemetry fields"
)]
pub fn remote_telemetry_event(
    category: RemoteTelemetryCategory,
    phase: RemoteTelemetryPhaseClass,
    result: RemoteTelemetryResultClass,
    route: RemoteTelemetryRouteClass,
    retry: RemoteTelemetryRetryClass,
    transfer: RemoteTelemetryTransferClass,
    reason: RemoteTelemetryReasonClass,
    capability: RemoteTelemetryCapabilityClass,
    measurement: RemoteTelemetryMeasurementKind,
    value: u64,
) -> Result<RemoteTelemetryEvent, RemoteTelemetryReasonCode> {
    remote_telemetry_event_from_fields(RemoteTelemetryEventFields {
        category,
        phase,
        result,
        route,
        retry,
        transfer,
        reason,
        capability,
        measurement,
        value,
    })
}

pub fn remote_telemetry_event_from_fields(
    fields: RemoteTelemetryEventFields,
) -> Result<RemoteTelemetryEvent, RemoteTelemetryReasonCode> {
    if fields.value == 0 {
        return Err(RemoteTelemetryReasonCode::EventValueInvalid);
    }
    let event = RemoteTelemetryEvent {
        schema: REMOTE_TELEMETRY_EVENT_SCHEMA.to_string(),
        category: fields.category,
        phase: fields.phase,
        result: fields.result,
        route: fields.route,
        retry: fields.retry,
        transfer: fields.transfer,
        reason: fields.reason,
        capability: fields.capability,
        measurement: fields.measurement,
        value: fields.value,
        non_claim: REMOTE_TELEMETRY_NON_CLAIM.to_string(),
    };
    validate_remote_telemetry_event(&event)?;
    debug_assert_eq!(event.value, fields.value);
    debug_assert_eq!(event.non_claim, REMOTE_TELEMETRY_NON_CLAIM);
    Ok(event)
}

pub fn validate_remote_telemetry_event(event: &RemoteTelemetryEvent) -> Result<(), RemoteTelemetryReasonCode> {
    if event.schema != REMOTE_TELEMETRY_EVENT_SCHEMA {
        return Err(RemoteTelemetryReasonCode::EventSchemaUnsupported);
    }
    if event.value == 0 || event.non_claim != REMOTE_TELEMETRY_NON_CLAIM {
        return Err(RemoteTelemetryReasonCode::EventValueInvalid);
    }
    Ok(())
}

pub fn record_remote_telemetry(
    buffer: &RemoteTelemetryBuffer,
    event: RemoteTelemetryEvent,
    policy: RemoteTelemetryPolicy,
) -> RemoteTelemetryBuffer {
    let mut next = buffer.clone();
    let valid = policy.validate().and_then(|()| validate_remote_telemetry_event(&event));
    let max_events = match usize::try_from(policy.event_capacity) {
        Ok(max_events) => max_events,
        Err(_) => {
            return record_dropped_telemetry(next, RemoteTelemetryReasonCode::PolicyInvalid);
        }
    };
    if let Err(reason) = valid {
        return record_dropped_telemetry(next, reason);
    }
    if next.events.len() >= max_events {
        return record_dropped_telemetry(next, RemoteTelemetryReasonCode::BufferFullDropped);
    }
    let Some(accepted_events) = next.accepted_events.checked_add(1) else {
        return record_dropped_telemetry(next, RemoteTelemetryReasonCode::ArithmeticOverflow);
    };
    next.events.push(event);
    next.accepted_events = accepted_events;
    next.last_drop_reason = None;
    debug_assert!(next.events.len() <= max_events);
    debug_assert!(next.accepted_events >= buffer.accepted_events);
    next
}

fn record_dropped_telemetry(
    mut buffer: RemoteTelemetryBuffer,
    reason: RemoteTelemetryReasonCode,
) -> RemoteTelemetryBuffer {
    match buffer.dropped_events.checked_add(1) {
        Some(dropped_events) => buffer.dropped_events = dropped_events,
        None => {
            buffer.dropped_events = u64::MAX;
            buffer.last_drop_reason = Some(RemoteTelemetryReasonCode::ArithmeticOverflow);
            return buffer;
        }
    }
    buffer.last_drop_reason = Some(reason);
    buffer
}

pub fn telemetry_for_priority_dispatch(
    decision: &PriorityDecisionEvidence,
) -> Result<RemoteTelemetryEvent, RemoteTelemetryReasonCode> {
    let capability = match decision.resource_fit_class {
        ResourceFitClass::Exact => RemoteTelemetryCapabilityClass::Exact,
        ResourceFitClass::Compatible => RemoteTelemetryCapabilityClass::Compatible,
        ResourceFitClass::Constrained => RemoteTelemetryCapabilityClass::Degraded,
        ResourceFitClass::Unknown => RemoteTelemetryCapabilityClass::Unknown,
    };
    let event = remote_telemetry_event_from_fields(RemoteTelemetryEventFields {
        category: RemoteTelemetryCategory::Assignment,
        phase: RemoteTelemetryPhaseClass::Assigned,
        result: RemoteTelemetryResultClass::Accepted,
        route: RemoteTelemetryRouteClass::NotApplicable,
        retry: RemoteTelemetryRetryClass::None,
        transfer: RemoteTelemetryTransferClass::None,
        reason: RemoteTelemetryReasonClass::PrioritySelected,
        capability,
        measurement: RemoteTelemetryMeasurementKind::CandidateCount,
        value: u64::from(decision.competing_goal_count),
    })?;
    debug_assert_eq!(event.capability, capability);
    debug_assert_eq!(event.value, u64::from(decision.competing_goal_count));
    Ok(event)
}

pub fn metric_descriptor_for_event(
    event: &RemoteTelemetryEvent,
) -> Result<RemoteMetricDescriptor, RemoteTelemetryReasonCode> {
    validate_remote_telemetry_event(event)?;
    let candidates = vec![
        candidate("category", enum_label(event.category)?),
        candidate("phase", enum_label(event.phase)?),
        candidate("result", enum_label(event.result)?),
        candidate("route", enum_label(event.route)?),
        candidate("retry", enum_label(event.retry)?),
        candidate("transfer", enum_label(event.transfer)?),
        candidate("reason", enum_label(event.reason)?),
        candidate("capability", enum_label(event.capability)?),
    ];
    let labels = admit_remote_metric_labels(&candidates)?;
    let descriptor = RemoteMetricDescriptor {
        name: metric_name(event.category).to_string(),
        kind: RemoteMetricKind::Counter,
        labels,
        value: event.value,
    };
    debug_assert_eq!(descriptor.value, event.value);
    debug_assert!(descriptor.labels.len() <= candidates.len());
    Ok(descriptor)
}

pub fn admit_remote_metric_labels(
    candidates: &[RemoteMetricLabelCandidate],
) -> Result<Vec<RemoteMetricLabel>, RemoteTelemetryReasonCode> {
    let count = u32::try_from(candidates.len()).map_err(|_| RemoteTelemetryReasonCode::LabelCountExceeded)?;
    if count > MAX_REMOTE_METRIC_LABELS {
        return Err(RemoteTelemetryReasonCode::LabelCountExceeded);
    }
    let mut labels = Vec::with_capacity(candidates.len());
    for candidate in candidates {
        validate_metric_label_candidate(candidate)?;
        labels.push(RemoteMetricLabel {
            key: candidate.key.clone(),
            value: candidate.value.clone(),
        });
    }
    Ok(labels)
}

fn validate_metric_label_candidate(candidate: &RemoteMetricLabelCandidate) -> Result<(), RemoteTelemetryReasonCode> {
    if candidate.key.is_empty() {
        return Err(RemoteTelemetryReasonCode::LabelBoundsExceeded);
    }
    if candidate.value.is_empty() {
        return Err(RemoteTelemetryReasonCode::LabelBoundsExceeded);
    }
    if candidate.key.len() > MAX_REMOTE_METRIC_LABEL_BYTES {
        return Err(RemoteTelemetryReasonCode::LabelBoundsExceeded);
    }
    if candidate.value.len() > MAX_REMOTE_METRIC_LABEL_BYTES {
        return Err(RemoteTelemetryReasonCode::LabelBoundsExceeded);
    }
    if candidate.key.chars().any(char::is_control) {
        return Err(RemoteTelemetryReasonCode::LabelBoundsExceeded);
    }
    if candidate.value.chars().any(char::is_control) {
        return Err(RemoteTelemetryReasonCode::LabelBoundsExceeded);
    }
    if looks_sensitive_key(&candidate.key) {
        return Err(RemoteTelemetryReasonCode::LabelKeySensitive);
    }
    if looks_sensitive_value(&candidate.value) {
        return Err(RemoteTelemetryReasonCode::LabelValueSensitive);
    }
    if !is_supported_label_key(&candidate.key) {
        return Err(RemoteTelemetryReasonCode::LabelKeyUnsupported);
    }
    if !is_supported_label_value(RemoteMetricLabelRef {
        key: &candidate.key,
        value: &candidate.value,
    }) {
        return Err(RemoteTelemetryReasonCode::LabelValueUnsupported);
    }
    debug_assert!(!candidate.key.is_empty());
    debug_assert!(!candidate.value.is_empty());
    Ok(())
}

fn looks_sensitive_key(value: &str) -> bool {
    let lowercase_value = value.to_ascii_lowercase();
    SENSITIVE_METRIC_KEY_MARKERS.iter().any(|marker| lowercase_value.contains(marker))
}

fn looks_sensitive_value(value: &str) -> bool {
    let value = value.to_ascii_lowercase();
    value.contains('/')
        || value.contains("bearer ")
        || value.contains("token=")
        || value.contains("secret")
        || value.contains("private-key")
        || value.contains("traceparent")
        || value.contains("error:")
        || value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn is_supported_label_key(value: &str) -> bool {
    matches!(value, "category" | "phase" | "result" | "route" | "retry" | "transfer" | "reason" | "capability")
}

struct RemoteMetricLabelRef<'a> {
    key: &'a str,
    value: &'a str,
}

fn is_supported_label_value(label: RemoteMetricLabelRef<'_>) -> bool {
    match label.key {
        "category" => enum_values::<RemoteTelemetryCategory>().contains(&label.value),
        "phase" => enum_values::<RemoteTelemetryPhaseClass>().contains(&label.value),
        "result" => enum_values::<RemoteTelemetryResultClass>().contains(&label.value),
        "route" => enum_values::<RemoteTelemetryRouteClass>().contains(&label.value),
        "retry" => enum_values::<RemoteTelemetryRetryClass>().contains(&label.value),
        "transfer" => enum_values::<RemoteTelemetryTransferClass>().contains(&label.value),
        "reason" => enum_values::<RemoteTelemetryReasonClass>().contains(&label.value),
        "capability" => enum_values::<RemoteTelemetryCapabilityClass>().contains(&label.value),
        _ => false,
    }
}

fn enum_label<T: Serialize>(value: T) -> Result<String, RemoteTelemetryReasonCode> {
    let rendered = serde_json::to_string(&value).map_err(|_| RemoteTelemetryReasonCode::LabelValueUnsupported)?;
    rendered
        .strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
        .map(str::to_string)
        .ok_or(RemoteTelemetryReasonCode::LabelValueUnsupported)
}

trait RemoteTelemetryEnumValues {
    const VALUES: &'static [&'static str];
}

impl RemoteTelemetryEnumValues for RemoteTelemetryCategory {
    const VALUES: &'static [&'static str] = &[
        "route",
        "queue",
        "assignment",
        "retry-fencing",
        "execution",
        "transfer",
        "admission",
        "publication",
    ];
}

impl RemoteTelemetryEnumValues for RemoteTelemetryPhaseClass {
    const VALUES: &'static [&'static str] = &[
        "planning",
        "queued",
        "assigned",
        "running",
        "transferring",
        "admitting",
        "publishing",
        "terminal",
        "not-applicable",
    ];
}

impl RemoteTelemetryEnumValues for RemoteTelemetryResultClass {
    const VALUES: &'static [&'static str] = &[
        "accepted",
        "rejected",
        "succeeded",
        "failed",
        "retried",
        "dropped",
        "pending",
        "not-applicable",
    ];
}

impl RemoteTelemetryEnumValues for RemoteTelemetryRouteClass {
    const VALUES: &'static [&'static str] = &[
        "local",
        "remote-stdio",
        "remote-ssh-stdio",
        "remote-p2p",
        "substitute",
        "preflight-error",
        "not-applicable",
    ];
}

impl RemoteTelemetryEnumValues for RemoteTelemetryRetryClass {
    const VALUES: &'static [&'static str] = &[
        "none",
        "current-fence",
        "retryable",
        "terminal",
        "policy-denied",
        "stale-fence",
        "unknown-fence",
    ];
}

impl RemoteTelemetryEnumValues for RemoteTelemetryTransferClass {
    const VALUES: &'static [&'static str] = &["none", "full", "delta", "streaming", "fallback", "not-available"];
}

impl RemoteTelemetryEnumValues for RemoteTelemetryReasonClass {
    const VALUES: &'static [&'static str] = &[
        "route-selected",
        "route-ineligible",
        "queue-admitted",
        "priority-selected",
        "worker-assigned",
        "attempt-transition",
        "fence-accepted",
        "retry-allowed",
        "retry-denied",
        "fence-rejected",
        "execution-completed",
        "execution-failed",
        "transfer-demand",
        "transfer-credit",
        "transfer-resumed",
        "transfer-cutoff",
        "transfer-completed",
        "transfer-fallback",
        "output-admitted",
        "output-rejected",
        "published",
        "publication-failed",
        "exporter-backpressure",
        "exporter-unavailable",
    ];
}

impl RemoteTelemetryEnumValues for RemoteTelemetryCapabilityClass {
    const VALUES: &'static [&'static str] = &[
        "exact",
        "compatible",
        "degraded",
        "ineligible",
        "unknown",
        "not-applicable",
    ];
}

fn enum_values<T: RemoteTelemetryEnumValues>() -> &'static [&'static str] {
    T::VALUES
}

fn candidate(key: &str, value: String) -> RemoteMetricLabelCandidate {
    RemoteMetricLabelCandidate {
        key: key.to_string(),
        value,
    }
}

fn metric_name(category: RemoteTelemetryCategory) -> &'static str {
    match category {
        RemoteTelemetryCategory::Route => "mantle_remote_route_total",
        RemoteTelemetryCategory::Queue => "mantle_remote_queue_total",
        RemoteTelemetryCategory::Assignment => "mantle_remote_assignment_total",
        RemoteTelemetryCategory::RetryFencing => "mantle_remote_retry_fencing_total",
        RemoteTelemetryCategory::Execution => "mantle_remote_execution_total",
        RemoteTelemetryCategory::Transfer => "mantle_remote_transfer_total",
        RemoteTelemetryCategory::Admission => "mantle_remote_admission_total",
        RemoteTelemetryCategory::Publication => "mantle_remote_publication_total",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn event() -> RemoteTelemetryEvent {
        remote_telemetry_event(
            RemoteTelemetryCategory::Assignment,
            RemoteTelemetryPhaseClass::Assigned,
            RemoteTelemetryResultClass::Accepted,
            RemoteTelemetryRouteClass::RemoteStdio,
            RemoteTelemetryRetryClass::None,
            RemoteTelemetryTransferClass::None,
            RemoteTelemetryReasonClass::PrioritySelected,
            RemoteTelemetryCapabilityClass::Exact,
            RemoteTelemetryMeasurementKind::Occurrences,
            1,
        )
        .unwrap()
    }

    #[test]
    fn equivalent_facts_normalize_to_provider_neutral_event_and_metric() {
        let first = event();
        let second = event();
        let metric = metric_descriptor_for_event(&first).unwrap();

        assert_eq!(first, second);
        assert_eq!(metric.labels.len(), usize::try_from(MAX_REMOTE_METRIC_LABELS).unwrap());
        assert_eq!(metric.name, "mantle_remote_assignment_total");
        assert_eq!(metric.value, 1);
        assert!(!serde_json::to_string(&metric).unwrap().contains("job"));
    }

    #[test]
    fn bounded_buffer_drops_without_rewriting_existing_events() {
        let policy = RemoteTelemetryPolicy {
            event_capacity: 1,
            batch_size: 1,
        };
        let first = record_remote_telemetry(&RemoteTelemetryBuffer::default(), event(), policy);
        let second = record_remote_telemetry(&first, event(), policy);

        assert_eq!(first.events, second.events);
        assert_eq!(second.accepted_events, 1);
        assert_eq!(second.dropped_events, 1);
        assert_eq!(second.last_drop_reason, Some(RemoteTelemetryReasonCode::BufferFullDropped));
    }

    #[test]
    fn raw_identifiers_paths_outputs_traces_credentials_and_errors_are_rejected() {
        let cases = [
            ("job_id", "job-123"),
            ("phase", "/mantle/store/abc-output"),
            ("output_name", "out"),
            ("trace_id", "0123456789abcdef0123456789abcdef"),
            ("result", "Bearer abc"),
            ("result", "token=abc"),
            ("key", "builder-key"),
            ("result", "error: arbitrary builder text"),
            ("phase", "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"),
        ];
        for (key, value) in cases {
            let result = admit_remote_metric_labels(&[RemoteMetricLabelCandidate {
                key: key.to_string(),
                value: value.to_string(),
            }]);
            assert!(result.is_err(), "{key}={value} must not become a metric label");
        }
    }

    #[test]
    fn invalid_policy_and_zero_value_fail_closed() {
        let invalid = RemoteTelemetryPolicy {
            event_capacity: 1,
            batch_size: 2,
        };
        let zero = remote_telemetry_event(
            RemoteTelemetryCategory::Queue,
            RemoteTelemetryPhaseClass::Queued,
            RemoteTelemetryResultClass::Accepted,
            RemoteTelemetryRouteClass::Local,
            RemoteTelemetryRetryClass::None,
            RemoteTelemetryTransferClass::None,
            RemoteTelemetryReasonClass::QueueAdmitted,
            RemoteTelemetryCapabilityClass::Exact,
            RemoteTelemetryMeasurementKind::Occurrences,
            0,
        );

        assert_eq!(invalid.validate(), Err(RemoteTelemetryReasonCode::PolicyInvalid));
        assert_eq!(zero, Err(RemoteTelemetryReasonCode::EventValueInvalid));
    }
}
