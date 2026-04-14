#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StoreAuditKind {
    PathInfoFallback,
    ClosureResolutionDegraded,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoreAuditEvent {
    pub kind: StoreAuditKind,
    pub detail: String,
}

impl StoreAuditEvent {
    pub fn new(kind: StoreAuditKind, detail: impl Into<String>) -> Self {
        let detail = detail.into();
        assert!(!detail.trim().is_empty(), "store audit detail must not be empty");
        Self { kind, detail }
    }
}
