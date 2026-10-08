# ADR 0101: Limit Nix gateway to bounded concrete operations

## Status

Proposed (2026-10-01). Pure operation admission, a bounded parser and an isolated read-only store adapter have focused tests with an installed Nix 2.36 private-socket client. Signed input-addressed derivation cache reads and exact-derivation projection are proved; opcode 9 explicitly rejects even a cached build request. A bounded challenge-prefaced ticket presentation has a scoped test against Mantle's real compatibility-ticket verifier, but no production consumer or ticket redemption. An isolated typed replay-ledger port has restart, race and corrupt-state tests, not a UCAN verifier integration. No production Nix gateway, authenticated build endpoint or public deployment is claimed.

## Context

Mantle already owns durable remote attempts, fences, verified store imports and signing. The Nix `ssh-ng` worker protocol has a much wider operation vocabulary and no intrinsic Mantle authorization scope. In the pinned `vendor/nix-compat` snapshot, the server handler acknowledges a limited set of queries and NAR upload, but does not implement `BuildPaths` or `BuildPathsWithResults`. Its initialization advertises `Trust::Trusted`, collection decoding calls `Vec::with_capacity` on an unbounded caller-supplied count, referrer/realisation queries can return fabricated empty lists, and its error path serializes debug errors. These are concrete reasons not to expose that handler to untrusted clients as-is.

## Decision drivers

- Admission must reject an unknown protocol version or operation before reaching a store, evaluator or scheduler.
- Lengths and collection counts must be bounded before allocating; upload must retain a total-transfer limit and reject incomplete framing.
- A Nix compatibility ticket must never imply service-administration authority. UCAN authority needs independently verified holder, proof, policy, replay and freshness facts.
- Durable attempt ownership, idempotency key, request digest and current fence are checked across disconnects and retries.
- Mantle's existing PathInfo, CAS, output signing and resource lease checks remain authoritative.

## Decision

Use an explicit **Nix worker protocol 1.37** and operation table: 1 `IsValidPath`, 19 `SetOptions`, 26 `QueryPathInfo`, 31 `QueryValidPaths`, 40 `QueryMissing`, 9 `BuildPaths`, and 39 `AddToStoreNar`. Other opcodes, including evaluator/administration requests, `BuildDerivation`, `BuildPathsWithResults`, referrers and realisations are rejected, not silently simulated. Recognition of an opcode in the bounded parser is not a promise of a working service: `QueryMissing` requires actual signed PathInfo and recursive CAS checks, while `BuildPaths` and `AddToStoreNar` require verified attempt/import adapters. Incoming build requests identify existing admitted derivation and store-input identities; the gateway never evaluates flakes, resolves registries, fetches sources or spawns arbitrary host tools.

`QueryValidPaths` admits only `substitute = false`; the gateway never
turns a path query into a source fetch or substitution. Build mode is
normal only, and NAR uploads cannot set `ultimate`, request repair or
bypass signature checks.

The isolated bounded parser negotiated protocol 1.37 with an installed Nix
2.36.0 client over a private Unix socket, reporting `trusted:false`. Both
`nix path-info` and `nix-store --realise` first send opcode 40
(`QueryMissing`); the latter sends a derived target of the form `drv!*`.
Before opcode 40 was admitted, this rejected the client. The isolated
read-only adapter now verifies configured trusted signatures over PathInfo,
the selected logical `/nix/store` prefix, declared CA path identity,
complete recursive CAS and the measured NAR digest and size before omitting
a requested path from `unknown`. An installed Nix 2.36 `path-info` client
consumed QueryMissing and QueryPathInfo replies and printed a genuinely
signed, measured store path; absent, tampered, wrong-CA-identity,
incomplete-CAS and signed-but-wrong-NAR PathInfo fixtures cannot succeed.

For an input-addressed derived target, the internal cache projection
requires signed, canonical `.drv` ATerm bytes in measured complete CAS,
a strongly admitted signed action result for the entire declared output
set, and present outputs matching signed identity and measured content.
It reports only already-present outputs (`willBuild = 0`,
`willSubstitute = 0`) and rejects missing, forged or ambiguous evidence.
The unsigned local CA mapping file cannot confer this authority. A real
Nix 2.36 `nix-store --realise` client consumes the signed cached `drv!*`
query, then sends opcode 9; the private gateway denies `BuildPaths` and
the client exits unsuccessfully. SO_PEERCRED authorizes owner-only reads,
not Build. Native Nix's worker handshake has no bearer or UCAN field;
Build requires independently verified ticket/UCAN ingress bound to that
connection and a real eligible worker before any success reply.

The separate read-only native BuildPaths projection accepts only a
signed/measured input-addressed `.drv` that round-trips through the
existing Crunch-to-Nix converter, all signed/measured source inputs and
exactly the complete declared output set; malformed, duplicated,
unknown or incomplete selections reject before any worker request.
This prepares typed derivations but does not produce an eligible
`ConcreteBuildRequest`, redeem authority or return opcode-9 success.

Unresolved CA `.drv` output paths (`path = None`) receive a typed denial:
the pinned `nix_compat::Derivation::from_aterm_bytes` uses `validate(true)`.
Supporting them requires a reviewed repo-owned Snix importer or upstream
`parse_unresolved` revision plus distinct CA registry/policy proof after
the pinned fork sync, never a local vendored parser bypass. The gateway
must not guess buildable/substitutable paths or treat metadata presence
as content availability. Nix clients send bounded settings overrides
during handshake; the parser consumes but never applies them to policy.

Named policy limits: 32 connections, 64 concurrent operations, 1 MiB frame, 4 MiB message, 1 GiB partial transfer, 16 concurrent partial transfers, 5-second negotiation, 30-second idle, 256 store paths per request, 64 KiB log window, 128 event items per page and 256-byte cursor. Enforce each limit in the transport **before allocation**; these constants in the pure core do not make an unsafe decoder safe. Unknown versions and malformed lengths must close without a side effect. The private endpoint is the only initial rollout; public enablement additionally requires the archived credential-hardening evidence and an OnixOS-owned deployment fixture. Valence's profile is a separate external dependency and must not be asserted as integrated.

Shell authentication produces verified, redacted authority facts for a pure `no_std` admission core. The shell must not trust caller-declared permissions. Submission identity binds key, subject, project and request digest to a committed attempt; duplicates with identical facts recover its public ID, conflicts reject. Reconnect reads by owner and authorized scope; cancellation and publication require a current fence. Import commands direct the shell to existing verified CAS/PathInfo services, never turn uploaded bytes into authority. Signed, bounded completion events and opaque authenticated cursors need explicit production journal and signer integration, not inferred from transport success.

A safe implementation may import a reviewed, owner-owned upstream `nix-compat` revision with bounded collection parsing, supported build operations and explicit trust/error controls, or implement a minimal independently bounded daemon parser in Mantle's shell. Vendored Nix/Snix sources are not hand-edited merely to unblock this gateway. Either implementation must pass malformed length, partial upload, real Nix client, replay, stale-fence and credential-redaction fixtures before enablement.

### Narrow authority integration proposal (not an admitted credential)

The existing verifier is sibling `ucan` revision
`c483c7b58c42ec6636e9b8b8c0f73a2b609bf21e`, already pinned and
source-snapshotted by Basalt (`basalt/vendor/ucan-revision.txt` and
`vendor/ucan.snapshot`). `basalt::ucan::verify_holder_signed_invocation`
checks the compact token, delegation and proof chain, audience-holder
signature, exact operation-profile admission, resource/ability and caveats,
then calls the supplied `HolderInvocationReplayAdmission` as its last
state-consuming step. Its `HolderInvocationAuthorized` result has private
construction. The verifier does **not** supply trusted keys, proof storage,
revocation freshness, clock, request canonicalization, caveat policy or a
durable nonce ledger. The weaker audience-only invocation API is not
acceptable for this gateway.

Propose one reviewed `mantle-gateway-build.v1` policy contract with explicit
abilities for submit, status, log-read, event-read, cancel, result-read and
usage-read; separate store-read and import abilities; **no** admin ability
on a compatibility ticket. The configured gateway service DID and issuer
trust anchors must be operator-pinned, not derived from a presented token.
Construct each requested resource from validated, delimiter-separated
project/attempt identities beneath that DID, with a trailing delimiter on
delegatable prefixes: UCAN resource containment is a *byte-prefix* match,
not a path-aware match. Bind the holder-signed `OperationBinding` to the
canonical gateway version, request digest, route, project, selected
derivation/output set, service endpoint and current attempt/fence; verify
the request against the same live target after authorization, before any
effect. Unknown abilities, caveats and profile IDs fail closed. For the
actual policy decision, use Basalt's generic
`authority_core::evaluate_authority` with a reviewed Mantle-specific
`Policy` and trusted policy reference, never Basalt's closed
`verify_ucan_enforcement_request` enum (which has no Mantle actions).
Do not construct `CredentialKind::VerifiedUcan` from a caller-provided
permission bitset or a Basalt receipt: the receiving shell must extract
verified grant facts and check this reviewed policy and local ownership.

The dependency candidate is the **immutable public UCAN Git revision**
`c483c7b58c42ec6636e9b8b8c0f73a2b609bf21e`, not a production
workspace-relative path into sibling Basalt and not crates.io `ucan ^0.1`
(Basalt explicitly documents that crates.io name as unrelated). The
upstream commit's `Cargo.toml` and `src/token/holder_invocation.rs` Git
object IDs match Basalt's reviewed snapshot byte-for-byte. That commit's
minimal verifier package closure includes path crates `ucan-core`
(verification-cache feature) and `verified-logic` plus immutable Radicle
revisions of `replay-ledger-core` and `revocation-view-core` at
`a0de79303f3f45c18a34fff535b690a85cb3cf59`, and the ordinary
locked Rust libraries from the UCAN manifest. `ucan-nostr` is a workspace
member but not required by the standalone holder verifier. Mantle, Basalt
and the `ucan` shell declare `AGPL-3.0-or-later`; `ucan-core` and
`verified-logic` declare `MPL-2.0`, and the pinned durable-authority-state
workspace declares `MIT`. Preserve their license files and
review the complete exact lock, source, and offline build closure before
any first consumer; licensing compatibility alone is not an excuse for
a casual vendor copy. This is a **private research proposal**, not
approval to add a moving Git ref, shared dependency or root manifest
entry before the real gateway receiver exists.

Use the pinned verifier's `ucan::DurableHolderReplayAdmission` with
`DurableReplayDurability::Synchronized` and its existing
`replay_ledger_core::ReplayLedgerPort` contract; do **not** reimplement
holder-verifier or replay-scope semantics. The pinned replay core's
`ReplayScope::slot_identity` binds tenant, actor, nonce, authority
generation and ledger namespace but intentionally excludes request digest:
changing the request cannot evade same-slot conflict. The shell chooses
nonzero, configured namespace/tenant/profile/attempt roles and a current
generation; its port must atomically compare-and-swap the exact expected
version and return `Synchronized` only after observed file **and parent
directory fsync**. A new authority generation is a separate replay
namespace and requires explicit revocation/rotation policy.

For the private replay ledger port, follow the repository's
`remote_credential_state` private owner/mode checks, cross-process lock,
bounded load, validation, atomic rename and corruption-fail-closed
pattern. Use a separate audience- and namespace-bound ledger with a
finite size/retention policy. Under the lock, load fresh persisted state
for each CAS, reject previously committed nonce slots regardless of a
changed request digest, and fail closed on load, lock, write, file-sync,
directory-sync or ambiguous commit. Never return `Synchronized` for a
mere in-memory mutation or unflushed rename. Do not reuse the ticket
state file, `NoRevocations`, or UCAN's `AllowAllReplayAdmission` as proof
of durable replay/revocation truth. Model a crash between replay commit
and build admission as a denied retry, with reconciliation by the
existing persisted attempt/idempotency key, not as an opportunity to
spend the same invocation again. The ticket path separately needs the
existing `commit_remote_ticket_admission_for_state` atomic redemption
and a service-observed endpoint; the challenge echo alone only rejects
an exact recorded frame and does not prevent a stolen bearer with a
fresh challenge.

The isolated `crunch-nix-gateway::replay::PrivateReplayLedger`
implements the pinned `ReplayLedgerPort` over owner-private files with a
bounded 5-second cross-process lock, canonical bounded state, separate
fsynced initialization marker, synced atomic state replacement and
denial of a missing/corrupt ledger after first reservation. Isolated
gateway-crate tests against the immutable `replay-ledger-core` revision
exercised reopen, changed-request nonce conflict, concurrent CAS with
exactly one applied, missing/corrupt state, lock contention and non-private
directory rejection (4 passed).
It is **not** connected to the UCAN holder verifier, current key/
revocation/time/profile/caveat inputs or any gateway effect; reaching its
finite capacity denies new admissions until a reviewed retention/rotation
policy exists. It does not defeat operator rollback of the entire state
directory and must not be represented as T5.2 completion.

### Async Build API mapping proposal (not yet an endpoint)

Expose versioned private routes `/v1/builds` (submit),
`/v1/builds/{attempt}` (status), `/logs` (bounded byte range),
`/events` (bounded page), `/cancel` (conditional mutation),
`/signed-result` (existing signed result only), and `/v1/usage`
(bounded summary). An unknown version/route/ability closes before a
state access. The pure core admits read-only event pages of at most 128
items and signed-result reads with distinct permissions; this does not
implement a server, authenticated cursors or sign any result. The shell
must authenticate first, then load the persisted
`RemoteCoordinatorState` and resolve the requested ID to its current
`RemoteCoordinatorJobSummary.current_attempt` and fence, refusing a
non-owner. Submit must use the existing `admit_coordinator_dispatch`
`AttachExisting`/`RedeliverResult` decisions and atomic idempotency
binding; **no second scheduler or fake accepted job**. A queued job is
observable as pending, never as successful BuildPaths.

For logs, call `replay_coordinator_attempt_log` with its persisted
`RemoteAttemptLogControlSummary`, cap returned bytes at 64 KiB and
records at the core page limit, and propagate its retained/truncated/head
cursor semantics rather than emitting fabricated empty records. Event
pages read a single durable per-attempt monotonic sequence with duplicate
delivery matched by exact `(attempt, fence, sequence, payload digest)`;
sign completion using the existing accepted action-result signer and
persist signed event identity before acknowledging it. An identical
redelivery returns the existing event; changed payload at the same
sequence, gaps, stale fence, and competing completion/cancel fail closed.
Cursor tokens of at most 256 bytes must carry version, owner/project,
attempt, fence, kind, offset and expiry under a separate keyed
authenticator; compare owner/fence to freshly loaded state on every
page. A public response contains no token or ticket material. Cancel
must conditionally mutate the current durable attempt fence; a terminal
completion wins only if already persisted, never based on an optimistic
in-memory reply. Result reads verify the existing signed result and output
set; usage comes from persisted transfer/resource lease/accounting
observations, not guessed CPU or byte counters. Explicit fixtures must
cover duplicate submit/delivery, missed pages, truncated logs, reconnect,
concurrent cancellation versus completion, revoked scope, stale fence and
post-restart nonce replay before any T4 acceptance claim.

The archived `2026-08-01-harden-remote-credential-boundary` validation
records completed focused credential tests and a tasks-gate receipt; its
historical broad-suite blockers and later archive status do not prove the
gateway's new ingress. Public exposure separately requires the T6.3
archive prerequisite, an OnixOS-owned T6.4 farm fixture and the full T7
rails. None is discharged by a private read-only test listener.

## Consequences and non-claims

The isolated pure admission, bounded parser and read-only store fixtures
establish their stated decisions over supplied authority facts and an
installed Nix 2.36 client's private-socket path-info query and denied
cached-derivation BuildPaths. They do not prove authenticity of Build
authority facts, production service integration, transport resumption,
successful Nix build or upload interoperability, actual execution,
sandboxing, arbitrary Nix compatibility, release eligibility or Valence
linkage. Keep the gateway disabled until the entire concrete shell,
authority, persistence and evidence path has direct end-to-end proof.
