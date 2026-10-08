#![no_std]
#![forbid(unsafe_code)]

//! Nix compatibility admission. Authentication, persistence, I/O and signing
//! belong to the host adapter; none of these decisions constitutes authority
//! to bypass store verification or the durable-attempt fence.

pub const NIX_PROTOCOL_MAJOR: u8 = 1;
pub const NIX_PROTOCOL_MINOR: u8 = 37;
pub const MAX_CONNECTIONS: u32 = 32;
pub const MAX_CONCURRENT_OPERATIONS: u32 = 64;
pub const MAX_FRAME_BYTES: u64 = 1_048_576;
pub const MAX_MESSAGE_BYTES: u64 = 4_194_304;
pub const MAX_TRANSFER_BYTES: u64 = 1_073_741_824;
pub const MAX_PARTIAL_TRANSFERS: u32 = 16;
pub const MAX_IDLE_SECS: u64 = 30;
pub const MAX_NEGOTIATION_SECS: u64 = 5;
pub const MAX_LOG_WINDOW_BYTES: u32 = 65_536;
pub const MAX_EVENT_PAGE_ITEMS: u32 = 128;
pub const MAX_CURSOR_BYTES: u32 = 256;
pub const MAX_STORE_PATHS: u32 = 256;
pub const MAX_IDENTITY_BYTES: usize = 128;

/// This is a closed table, not the wider vendored parser's operation enum.
/// BuildPaths (9) and AddToStoreNar (39) require an adapter that can actually
/// call the existing verified attempt/import services; never acknowledge them
/// merely because the wire parser recognizes them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NixOperation {
    IsValidPath,
    SetOptions,
    QueryPathInfo,
    QueryValidPaths,
    BuildPaths,
    AddToStoreNar,
    QueryMissing,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Reject {
    Version,
    Operation,
    Bound,
    Identity,
    Expired,
    Audience,
    Scope,
    Authority,
    TicketAdministration,
    NotOwner,
    Conflict,
    StaleFence,
}

impl Reject {
    /// Stable, credential-free reason; never format caller-controlled input.
    pub const fn code(self) -> &'static str {
        match self {
            Self::Version => "gateway-version-unsupported",
            Self::Operation => "gateway-operation-unsupported",
            Self::Bound => "gateway-limit-exceeded",
            Self::Identity => "gateway-identity-invalid",
            Self::Expired => "gateway-authority-expired",
            Self::Audience => "gateway-audience-mismatch",
            Self::Scope => "gateway-scope-mismatch",
            Self::Authority => "gateway-authority-denied",
            Self::TicketAdministration => "gateway-ticket-admin-denied",
            Self::NotOwner => "gateway-attempt-owner-mismatch",
            Self::Conflict => "gateway-idempotency-conflict",
            Self::StaleFence => "gateway-fence-stale",
        }
    }
}

// r[impl remote_builds.nix_compatibility_gateway]
pub const fn nix_operation(version_major: u8, version_minor: u8, code: u64) -> Result<NixOperation, Reject> {
    if version_major != NIX_PROTOCOL_MAJOR || version_minor != NIX_PROTOCOL_MINOR {
        return Err(Reject::Version);
    }
    match code {
        1 => Ok(NixOperation::IsValidPath),
        9 => Ok(NixOperation::BuildPaths),
        19 => Ok(NixOperation::SetOptions),
        26 => Ok(NixOperation::QueryPathInfo),
        31 => Ok(NixOperation::QueryValidPaths),
        39 => Ok(NixOperation::AddToStoreNar),
        40 => Ok(NixOperation::QueryMissing),
        _ => Err(Reject::Operation),
    }
}

/// Every length is checked before the shell allocates or reads the payload.
/// A transfer may contain multiple bounded frames but has its own total limit.
// r[impl remote_builds.gateway_bounds_and_recovery]
pub const fn admit_lengths(frame_bytes: u64, message_bytes: u64, transferred_bytes: u64) -> Result<(), Reject> {
    if frame_bytes > MAX_FRAME_BYTES || message_bytes > MAX_MESSAGE_BYTES || transferred_bytes > MAX_TRANSFER_BYTES {
        return Err(Reject::Bound);
    }
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CredentialKind {
    VerifiedUcan,
    VerifiedCompatibilityTicket,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Permission {
    Build,
    Upload,
    StoreRead,
    StatusRead,
    LogRead,
    Cancel,
    Publish,
    EventRead,
    ResultRead,
    UsageRead,
    Admin,
}

impl Permission {
    const fn bit(self) -> u16 {
        match self {
            Self::Build => 1,
            Self::Upload => 2,
            Self::StoreRead => 4,
            Self::StatusRead => 8,
            Self::LogRead => 16,
            Self::Cancel => 32,
            Self::Publish => 64,
            Self::UsageRead => 128,
            Self::Admin => 256,
            Self::EventRead => 512,
            Self::ResultRead => 1024,
        }
    }
}

/// Shell-created facts from independently verified credential/holder proofs;
/// never build these by parsing the caller's claimed permissions directly.
#[derive(Debug, Clone, Copy)]
pub struct Authority<'a> {
    pub kind: CredentialKind,
    pub subject: &'a str,
    pub scope: &'a str,
    pub audience: &'a str,
    pub expires_unix_s: u64,
    pub permissions: u16,
}

impl Authority<'_> {
    pub const fn permits(&self, permission: Permission) -> bool {
        self.permissions & permission.bit() != 0
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Attempt<'a> {
    pub id: &'a str,
    pub subject: &'a str,
    pub scope: &'a str,
    pub fence: u64,
}

#[derive(Debug, Clone, Copy)]
pub struct Submission<'a> {
    pub key: &'a str,
    pub subject: &'a str,
    pub scope: &'a str,
    pub request_digest: &'a str,
    pub attempt_id: &'a str,
}

#[derive(Debug, Clone, Copy)]
pub enum Request<'a> {
    Build {
        drv_path: &'a str,
        key: &'a str,
        digest: &'a str,
        existing: Option<Submission<'a>>,
    },
    Upload {
        store_path: &'a str,
        bytes: u64,
    },
    StoreInfo {
        store_path: &'a str,
    },
    Status {
        attempt: Attempt<'a>,
    },
    Log {
        attempt: Attempt<'a>,
        bytes: u32,
    },
    EventPage {
        attempt: Attempt<'a>,
        items: u32,
    },
    SignedResult {
        attempt: Attempt<'a>,
    },
    Cancel {
        attempt: Attempt<'a>,
        fence: u64,
    },
    Publish {
        attempt: Attempt<'a>,
        fence: u64,
    },
    Usage {
        items: u32,
    },
    Admin,
}

/// These typed commands are interpreted only by host adapters that enforce
/// CAS, PathInfo, lease, fencing, result signature and publication checks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Command<'a> {
    Submit {
        drv_path: &'a str,
        key: &'a str,
        digest: &'a str,
    },
    Existing {
        attempt_id: &'a str,
    },
    ImportVerified {
        store_path: &'a str,
        bytes: u64,
    },
    QueryStore {
        store_path: &'a str,
    },
    QueryStatus {
        attempt_id: &'a str,
    },
    QueryLog {
        attempt_id: &'a str,
        bytes: u32,
    },
    QueryEvents {
        attempt_id: &'a str,
        items: u32,
    },
    QuerySignedResult {
        attempt_id: &'a str,
    },
    Cancel {
        attempt_id: &'a str,
        fence: u64,
    },
    PublishVerified {
        attempt_id: &'a str,
        fence: u64,
    },
    QueryUsage {
        items: u32,
    },
    Admin,
}

fn valid_identity(value: &str) -> bool {
    !value.is_empty() && value.len() <= MAX_IDENTITY_BYTES && value.bytes().all(|byte| byte.is_ascii_graphic())
}

pub fn valid_store_path(value: &str, derivation: bool) -> bool {
    if !value.starts_with("/nix/store/") || value.len() > 255 || !value.is_ascii() {
        return false;
    }
    let name = &value[11..];
    if name.len() < 34 || name.as_bytes()[32] != b'-' {
        return false;
    }
    if !name[..32].bytes().all(|byte| b"0123456789abcdfghijklmnpqrsvwxyz".contains(&byte)) {
        return false;
    }
    if !name[33..].bytes().all(|byte| byte.is_ascii_alphanumeric() || b"+._?=-".contains(&byte)) {
        return false;
    }
    !derivation || name.ends_with(".drv")
}

fn owned(attempt: Attempt<'_>, authority: Authority<'_>) -> Result<(), Reject> {
    if !valid_identity(attempt.id) || attempt.fence == 0 {
        return Err(Reject::Identity);
    }
    if attempt.subject != authority.subject || attempt.scope != authority.scope {
        return Err(Reject::NotOwner);
    }
    Ok(())
}

/// Authentication and state lookup MUST precede this pure admission call.
/// The caller must atomically commit the returned submission key with its
/// subject, scope, request digest and durable attempt before acknowledging it.
// r[impl remote_builds.gateway_functional_core]
// r[impl remote_builds.granular_service_authority]
pub fn admit<'a>(
    request: Request<'a>,
    authority: Authority<'_>,
    now_unix_s: u64,
    audience: &str,
    scope: &str,
) -> Result<Command<'a>, Reject> {
    if !valid_identity(authority.subject) || !valid_identity(authority.scope) || !valid_identity(authority.audience) {
        return Err(Reject::Identity);
    }
    if now_unix_s >= authority.expires_unix_s {
        return Err(Reject::Expired);
    }
    if authority.audience != audience {
        return Err(Reject::Audience);
    }
    if authority.scope != scope {
        return Err(Reject::Scope);
    }
    let needed = match request {
        Request::Build { .. } => Permission::Build,
        Request::Upload { .. } => Permission::Upload,
        Request::StoreInfo { .. } => Permission::StoreRead,
        Request::Status { .. } => Permission::StatusRead,
        Request::Log { .. } => Permission::LogRead,
        Request::EventPage { .. } => Permission::EventRead,
        Request::SignedResult { .. } => Permission::ResultRead,
        Request::Cancel { .. } => Permission::Cancel,
        Request::Publish { .. } => Permission::Publish,
        Request::Usage { .. } => Permission::UsageRead,
        Request::Admin => Permission::Admin,
    };
    if needed == Permission::Admin && authority.kind == CredentialKind::VerifiedCompatibilityTicket {
        return Err(Reject::TicketAdministration);
    }
    if !authority.permits(needed) {
        return Err(Reject::Authority);
    }
    match request {
        Request::Build {
            drv_path,
            key,
            digest,
            existing,
        } => {
            if !valid_store_path(drv_path, true) || !valid_identity(key) || !valid_identity(digest) {
                return Err(Reject::Identity);
            }
            if let Some(previous) = existing {
                if previous.key != key
                    || previous.subject != authority.subject
                    || previous.scope != scope
                    || previous.request_digest != digest
                {
                    return Err(Reject::Conflict);
                }
                if !valid_identity(previous.attempt_id) {
                    return Err(Reject::Identity);
                }
                return Ok(Command::Existing {
                    attempt_id: previous.attempt_id,
                });
            }
            Ok(Command::Submit { drv_path, key, digest })
        }
        Request::Upload { store_path, bytes } => {
            if !valid_store_path(store_path, false) {
                return Err(Reject::Identity);
            }
            admit_lengths(0, 0, bytes)?;
            Ok(Command::ImportVerified { store_path, bytes })
        }
        Request::StoreInfo { store_path } => {
            if !valid_store_path(store_path, false) {
                return Err(Reject::Identity);
            }
            Ok(Command::QueryStore { store_path })
        }
        Request::Status { attempt } => {
            owned(attempt, authority)?;
            Ok(Command::QueryStatus { attempt_id: attempt.id })
        }
        Request::Log { attempt, bytes } => {
            owned(attempt, authority)?;
            if bytes > MAX_LOG_WINDOW_BYTES {
                return Err(Reject::Bound);
            }
            Ok(Command::QueryLog {
                attempt_id: attempt.id,
                bytes,
            })
        }
        Request::EventPage { attempt, items } => {
            owned(attempt, authority)?;
            if items == 0 || items > MAX_EVENT_PAGE_ITEMS {
                return Err(Reject::Bound);
            }
            Ok(Command::QueryEvents {
                attempt_id: attempt.id,
                items,
            })
        }
        Request::SignedResult { attempt } => {
            owned(attempt, authority)?;
            Ok(Command::QuerySignedResult { attempt_id: attempt.id })
        }
        Request::Cancel { attempt, fence } => {
            owned(attempt, authority)?;
            if fence != attempt.fence {
                return Err(Reject::StaleFence);
            }
            Ok(Command::Cancel {
                attempt_id: attempt.id,
                fence,
            })
        }
        Request::Publish { attempt, fence } => {
            owned(attempt, authority)?;
            if fence != attempt.fence {
                return Err(Reject::StaleFence);
            }
            Ok(Command::PublishVerified {
                attempt_id: attempt.id,
                fence,
            })
        }
        Request::Usage { items } => {
            if items > MAX_EVENT_PAGE_ITEMS {
                return Err(Reject::Bound);
            }
            Ok(Command::QueryUsage { items })
        }
        Request::Admin => Ok(Command::Admin),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DRV: &str = "/nix/store/0123456789abcdfghijklmnpqrsvwxyz-hello.drv";
    const ATTEMPT: Attempt<'static> = Attempt {
        id: "attempt-1",
        subject: "alice",
        scope: "project-a",
        fence: 3,
    };

    fn authority(kind: CredentialKind, permission: Permission) -> Authority<'static> {
        Authority {
            kind,
            subject: "alice",
            scope: "project-a",
            audience: "private-gateway",
            expires_unix_s: 100,
            permissions: permission.bit(),
        }
    }

    fn check(
        request: Request<'static>,
        kind: CredentialKind,
        permission: Permission,
    ) -> Result<Command<'static>, Reject> {
        admit(request, authority(kind, permission), 99, "private-gateway", "project-a")
    }

    #[test]
    fn closed_wire_table_and_allocation_bounds() {
        for code in [1, 9, 19, 26, 31, 39, 40] {
            assert!(nix_operation(1, 37, code).is_ok());
        }
        for code in [0, 7, 20, 36, 38, 43, 46, u64::MAX] {
            assert_eq!(nix_operation(1, 37, code), Err(Reject::Operation));
        }
        assert_eq!(nix_operation(1, 35, 9), Err(Reject::Version));
        assert_eq!(nix_operation(2, 37, 9), Err(Reject::Version));
        assert_eq!(admit_lengths(MAX_FRAME_BYTES, MAX_MESSAGE_BYTES, MAX_TRANSFER_BYTES), Ok(()));
        assert_eq!(admit_lengths(MAX_FRAME_BYTES + 1, 0, 0), Err(Reject::Bound));
        assert_eq!(admit_lengths(0, MAX_MESSAGE_BYTES + 1, 0), Err(Reject::Bound));
        assert_eq!(admit_lengths(0, 0, MAX_TRANSFER_BYTES + 1), Err(Reject::Bound));
    }

    #[test]
    fn concrete_build_is_idempotent_only_for_matching_owner_scope_and_digest() {
        let build = |existing| Request::Build {
            drv_path: DRV,
            key: "request-1",
            digest: "blake3-1",
            existing,
        };
        let kind = CredentialKind::VerifiedUcan;
        assert_eq!(
            check(build(None), kind, Permission::Build),
            Ok(Command::Submit {
                drv_path: DRV,
                key: "request-1",
                digest: "blake3-1"
            })
        );
        let prior = Submission {
            key: "request-1",
            subject: "alice",
            scope: "project-a",
            request_digest: "blake3-1",
            attempt_id: "attempt-1",
        };
        assert_eq!(
            check(build(Some(prior)), kind, Permission::Build),
            Ok(Command::Existing {
                attempt_id: "attempt-1"
            })
        );
        for wrong in [
            Submission {
                subject: "bob",
                ..prior
            },
            Submission {
                scope: "project-b",
                ..prior
            },
            Submission {
                request_digest: "different",
                ..prior
            },
            Submission { key: "other", ..prior },
        ] {
            assert_eq!(check(build(Some(wrong)), kind, Permission::Build), Err(Reject::Conflict));
        }
        assert_eq!(
            check(
                Request::Build {
                    drv_path: "/tmp/hello.drv",
                    key: "key",
                    digest: "digest",
                    existing: None
                },
                kind,
                Permission::Build
            ),
            Err(Reject::Identity)
        );
        assert!(!valid_store_path("/nix/store/0123456789abcdfghijklmnpqrsvwxyé-hello.drv", true));
        assert!(!valid_store_path("/nix/store/0123456789abcdfghijklmnpqrsvwxyz-dir/file.drv", true));
    }

    #[test]
    fn authorities_are_distinct_and_ticket_cannot_admin() {
        let kind = CredentialKind::VerifiedCompatibilityTicket;
        assert_eq!(check(Request::Admin, kind, Permission::Admin), Err(Reject::TicketAdministration));
        assert_eq!(
            check(Request::Status { attempt: ATTEMPT }, kind, Permission::StatusRead),
            Ok(Command::QueryStatus {
                attempt_id: "attempt-1"
            })
        );
        for request in [
            Request::Build {
                drv_path: DRV,
                key: "key",
                digest: "digest",
                existing: None,
            },
            Request::Upload {
                store_path: DRV,
                bytes: 1,
            },
            Request::StoreInfo { store_path: DRV },
            Request::Log {
                attempt: ATTEMPT,
                bytes: 1,
            },
            Request::EventPage {
                attempt: ATTEMPT,
                items: 1,
            },
            Request::SignedResult { attempt: ATTEMPT },
            Request::Cancel {
                attempt: ATTEMPT,
                fence: 3,
            },
            Request::Publish {
                attempt: ATTEMPT,
                fence: 3,
            },
            Request::Usage { items: 1 },
        ] {
            assert_eq!(check(request, kind, Permission::StatusRead), Err(Reject::Authority));
        }
        assert_eq!(check(Request::Admin, CredentialKind::VerifiedUcan, Permission::Admin), Ok(Command::Admin));
    }

    #[test]
    fn store_reconnect_cancel_and_publication_require_exact_facts() {
        let kind = CredentialKind::VerifiedUcan;
        assert_eq!(
            check(
                Request::Upload {
                    store_path: DRV,
                    bytes: 15
                },
                kind,
                Permission::Upload
            ),
            Ok(Command::ImportVerified {
                store_path: DRV,
                bytes: 15
            })
        );
        assert_eq!(
            check(Request::StoreInfo { store_path: DRV }, kind, Permission::StoreRead),
            Ok(Command::QueryStore { store_path: DRV })
        );
        assert_eq!(
            check(
                Request::Log {
                    attempt: ATTEMPT,
                    bytes: MAX_LOG_WINDOW_BYTES
                },
                kind,
                Permission::LogRead
            ),
            Ok(Command::QueryLog {
                attempt_id: "attempt-1",
                bytes: MAX_LOG_WINDOW_BYTES
            })
        );
        assert_eq!(
            check(
                Request::Cancel {
                    attempt: ATTEMPT,
                    fence: 3
                },
                kind,
                Permission::Cancel
            ),
            Ok(Command::Cancel {
                attempt_id: "attempt-1",
                fence: 3
            })
        );
        assert_eq!(
            check(
                Request::Publish {
                    attempt: ATTEMPT,
                    fence: 3
                },
                kind,
                Permission::Publish
            ),
            Ok(Command::PublishVerified {
                attempt_id: "attempt-1",
                fence: 3
            })
        );
        assert_eq!(
            check(
                Request::Usage {
                    items: MAX_EVENT_PAGE_ITEMS
                },
                kind,
                Permission::UsageRead
            ),
            Ok(Command::QueryUsage {
                items: MAX_EVENT_PAGE_ITEMS
            })
        );
        assert_eq!(
            check(
                Request::Cancel {
                    attempt: ATTEMPT,
                    fence: 2
                },
                kind,
                Permission::Cancel
            ),
            Err(Reject::StaleFence)
        );
        assert_eq!(
            check(
                Request::Publish {
                    attempt: ATTEMPT,
                    fence: 4
                },
                kind,
                Permission::Publish
            ),
            Err(Reject::StaleFence)
        );
        assert_eq!(
            check(
                Request::Log {
                    attempt: ATTEMPT,
                    bytes: MAX_LOG_WINDOW_BYTES + 1
                },
                kind,
                Permission::LogRead
            ),
            Err(Reject::Bound)
        );
        assert_eq!(
            check(
                Request::Upload {
                    store_path: DRV,
                    bytes: MAX_TRANSFER_BYTES + 1
                },
                kind,
                Permission::Upload
            ),
            Err(Reject::Bound)
        );
        let foreign = Attempt {
            subject: "bob",
            ..ATTEMPT
        };
        assert_eq!(check(Request::Status { attempt: foreign }, kind, Permission::StatusRead), Err(Reject::NotOwner));
    }

    #[test]
    fn event_pages_and_signed_results_require_independent_owner_scope_and_bounds() {
        let kind = CredentialKind::VerifiedCompatibilityTicket;
        assert_eq!(
            check(
                Request::EventPage {
                    attempt: ATTEMPT,
                    items: MAX_EVENT_PAGE_ITEMS
                },
                kind,
                Permission::EventRead
            ),
            Ok(Command::QueryEvents {
                attempt_id: ATTEMPT.id,
                items: MAX_EVENT_PAGE_ITEMS
            })
        );
        assert_eq!(
            check(Request::SignedResult { attempt: ATTEMPT }, kind, Permission::ResultRead),
            Ok(Command::QuerySignedResult { attempt_id: ATTEMPT.id })
        );
        for items in [0, MAX_EVENT_PAGE_ITEMS + 1] {
            assert_eq!(
                check(
                    Request::EventPage {
                        attempt: ATTEMPT,
                        items
                    },
                    kind,
                    Permission::EventRead
                ),
                Err(Reject::Bound)
            );
        }
        assert_eq!(
            check(Request::SignedResult { attempt: ATTEMPT }, kind, Permission::EventRead),
            Err(Reject::Authority)
        );
        assert_eq!(
            check(
                Request::EventPage {
                    attempt: ATTEMPT,
                    items: 1
                },
                kind,
                Permission::ResultRead
            ),
            Err(Reject::Authority)
        );
        let foreign = Attempt {
            subject: "bob",
            ..ATTEMPT
        };
        assert_eq!(
            check(
                Request::EventPage {
                    attempt: foreign,
                    items: 1
                },
                kind,
                Permission::EventRead
            ),
            Err(Reject::NotOwner)
        );
        assert_eq!(
            check(Request::SignedResult { attempt: foreign }, kind, Permission::ResultRead),
            Err(Reject::NotOwner)
        );
    }

    #[test]
    fn expired_or_wrong_audience_never_leaks_identity() {
        let auth = authority(CredentialKind::VerifiedUcan, Permission::StatusRead);
        let request = Request::Status { attempt: ATTEMPT };
        assert_eq!(admit(request, auth, 100, "private-gateway", "project-a"), Err(Reject::Expired));
        assert_eq!(admit(request, auth, 99, "other", "project-a"), Err(Reject::Audience));
        assert_eq!(admit(request, auth, 99, "private-gateway", "project-b"), Err(Reject::Scope));
        for error in [
            Reject::Version,
            Reject::Operation,
            Reject::Bound,
            Reject::Identity,
            Reject::Expired,
            Reject::Audience,
            Reject::Scope,
            Reject::Authority,
            Reject::TicketAdministration,
            Reject::NotOwner,
            Reject::Conflict,
            Reject::StaleFence,
        ] {
            assert!(!error.code().contains("alice"));
            assert!(!error.code().contains("ticket") || error == Reject::TicketAdministration);
        }
    }
}
