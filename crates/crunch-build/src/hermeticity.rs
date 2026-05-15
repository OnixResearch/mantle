#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HermeticityMode {
    Practical,
    Strict,
    Impure,
}

impl HermeticityMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Practical => "practical",
            Self::Strict => "strict",
            Self::Impure => "impure",
        }
    }

    pub fn is_strict(self) -> bool {
        matches!(self, Self::Strict)
    }

    pub fn is_impure(self) -> bool {
        matches!(self, Self::Impure)
    }
}

impl std::fmt::Display for HermeticityMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HermeticityAuditKind {
    HostToolFallback,
    PathInfoFallback,
    ClosureResolutionDegraded,
    EnvironmentOverride,
    FetchToolFallback,
    ImpureModeSelected,
}

impl HermeticityAuditKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::HostToolFallback => "host-tool-fallback",
            Self::PathInfoFallback => "pathinfo-fallback",
            Self::ClosureResolutionDegraded => "closure-resolution-degraded",
            Self::EnvironmentOverride => "environment-override",
            Self::FetchToolFallback => "fetch-tool-fallback",
            Self::ImpureModeSelected => "impure-mode-selected",
        }
    }
}

impl std::fmt::Display for HermeticityAuditKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HermeticityAuditEvent {
    pub kind: HermeticityAuditKind,
    pub detail: String,
}

impl HermeticityAuditEvent {
    pub fn new(kind: HermeticityAuditKind, detail: impl Into<String>) -> Self {
        let detail = detail.into();
        assert!(!detail.trim().is_empty(), "hermeticity audit detail must not be empty");
        Self { kind, detail }
    }
}

impl From<crunch_store::StoreAuditKind> for HermeticityAuditKind {
    fn from(value: crunch_store::StoreAuditKind) -> Self {
        match value {
            crunch_store::StoreAuditKind::PathInfoFallback => Self::PathInfoFallback,
            crunch_store::StoreAuditKind::ClosureResolutionDegraded => Self::ClosureResolutionDegraded,
        }
    }
}

impl From<crunch_store::StoreAuditEvent> for HermeticityAuditEvent {
    fn from(value: crunch_store::StoreAuditEvent) -> Self {
        Self::new(value.kind.into(), value.detail)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hermeticity_mode_display_uses_stable_strings() {
        assert_eq!(HermeticityMode::Practical.as_str(), "practical");
        assert_eq!(HermeticityMode::Strict.to_string(), "strict");
        assert_eq!(HermeticityMode::Impure.as_str(), "impure");
        assert!(HermeticityMode::Strict.is_strict());
        assert!(!HermeticityMode::Practical.is_strict());
        assert!(HermeticityMode::Impure.is_impure());
        assert!(!HermeticityMode::Strict.is_impure());
    }

    #[test]
    fn hermeticity_audit_event_preserves_kind_and_detail() {
        let event = HermeticityAuditEvent::new(HermeticityAuditKind::HostToolFallback, "used host bwrap");
        assert_eq!(event.kind.as_str(), "host-tool-fallback");
        assert_eq!(event.detail, "used host bwrap");
    }

    #[test]
    fn store_audit_kinds_map_to_public_hermeticity_kinds() {
        let pathinfo = HermeticityAuditKind::from(crunch_store::StoreAuditKind::PathInfoFallback);
        let closure = HermeticityAuditKind::from(crunch_store::StoreAuditKind::ClosureResolutionDegraded);
        assert_eq!(pathinfo, HermeticityAuditKind::PathInfoFallback);
        assert_eq!(closure, HermeticityAuditKind::ClosureResolutionDegraded);
    }
}
