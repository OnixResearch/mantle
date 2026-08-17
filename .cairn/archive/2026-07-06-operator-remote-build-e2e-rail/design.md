## Design

### Goal

Prove the operator remote-build e2e rail against the already-accepted
`[depends:remote_builds.operator_e2e_rail]` requirement without changing its spec text.
The rail is an implementation/evidence artifact, not a new capability.

### Functional core / imperative shell

- **Core reuse, not new core.** The rail MUST drive the same core validation
  seams already used by supported remote-build operation: route planning,
  framed handshake, missing-input sync, signed output admission, and bounded
  observability. No rail-specific trust, transfer, or admission logic is added
  to `crunch-build`/`crunch-store` core; the rail is a shell orchestrator that
  wires existing seams together with bounded local multi-process fixtures.
- **Shell orchestration.** A rail driver (preferred: a `.rs` script under
  `scripts/` or a `#[ignore]`'d integration test) spawns a bounded set of local
  processes (a coordinator, one or more workers over stdio/loopback, and a
  client), owns readiness via owned listeners or staged files, and collects
  machine-readable evidence. No ambient network services, no hidden global state.
- **Evidence is a pure render.** The rail produces a versioned JSON evidence
  object from the recorded seam outputs. Evidence rendering MUST be deterministic
  pure logic over the recorded facts; the shell only feeds it observed bytes and
  digest/identity facts.

### Evidence shape (machine-readable, redacted)

The emitted JSON evidence MUST include:

- `rail_version`, `fixture_id`, and a stable `mode` label.
- `composition`: ordered phase records for route, handshake, input-sync,
  execution, transfer/admission, and observability, each with a phase status and
  the core seam used.
- `upload_summary`: bounded object counts and byte counts by class.
- `trust_basis`: signer/trust-basis identity (no private key paths).
- `artifact_attestation_ref`: artifact-attestation path/digest reference.
- `log_status_bounds`: enforced byte/chunk/cursor limits.
- `redaction`: assertion that bearer tickets, private key paths, raw
  environment values, and uploaded content were omitted.
- `non_claims`: explicit statement that the rail proves fixture composition only
  — not production P2P deployment, release reproducibility, or package-manager
  compatibility.

Evidence MUST NOT carry bearer tickets, private key paths, raw environment
values, uploaded content, or unbounded argv/path/log lists.

### Negative cross-seam harness

A negative fixture selector runs each of: missing output trust, unframed
stdout, stale source state, upload-quota/privacy overflow, and fallback
requested without explicit policy. Each negative case MUST reject before output
admission or local-success reporting and MUST record a phase + stable reason
code. The harness asserts no secret leakage in negative diagnostics.

### Determinism

Repeated rail runs on the same fixture MUST produce byte-stable evidence, with
any inherently non-deterministic fields (timestamps, temp paths) documented and,
where possible, scrubbed or omitted. The rail MUST NOT depend on ambient network
or hidden global state.

### Risks

- Existing partial rail code may already satisfy some clauses; the audit task
  (I1) prevents duplicating work and keeps the change focused on real gaps.
- Multi-process fixtures can be flaky on a loaded host; readiness MUST be owned
  listeners/staged files, not sleep loops, and the rail MUST bound process
  lifetimes and reap children on timeout.
