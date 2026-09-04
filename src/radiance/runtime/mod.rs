pub(super) mod arguments;
pub(super) mod compile;
pub(super) mod execution;
pub(super) mod link;
pub(super) mod native;
pub(super) mod network;

pub(super) const PROTECTED_AUDIT_SCHEMA: &str = "mantle-radiance-protected-execution-audit-v1";
pub(super) const PROTECTED_NETWORK_POLICY: &str = "close-inherited-fds-and-trap-nonlocal-sockets-v1";

#[derive(Debug, serde::Serialize)]
pub(super) struct AuditView<'a> {
    pub schema: &'static str,
    pub network_policy: &'static str,
    pub events: &'a [crate::protected_exec::ProtectedSeccompAuditEvent],
}

#[derive(Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct AuditRecord {
    pub schema: String,
    pub network_policy: String,
    pub events: Vec<crate::protected_exec::ProtectedSeccompAuditEvent>,
}
