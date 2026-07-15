## Context

The handoff validator is already shaped as a useful pure core, but a validator that is reachable only from unit tests does not protect release assembly or `mantle release verify`. Separately, a command path that always returns unavailable creates a misleading public capability. Practical hermetic mode may preserve useful diagnostics, but it cannot support the same claims as an enforced strict sandbox.

## Decisions

### Wire Cairn handoff validation into production

Release assembly and verification normalize loaded Cairn handoff rows and call the pure validator before those rows can enter a passing bundle or release receipt. The shell reads explicit files, computes BLAKE3 over artifact bytes, and supplies typed measured identities. The core compares artifact id, role, schema, measured digest, Cairn policy digest, readiness id, coverage ids, and typed non-claim boundaries.

No production path may mark a Cairn handoff present merely because metadata parsed or a digest-shaped string was supplied.

### Make bypass visible and fail closed

A release profile that requires Cairn evidence fails if the validator was not invoked, if validation evidence is missing, or if the validation receipt is not bound to the same release bundle. Optional generic profiles record absence without claiming validated handoff.

### Make source-root capability honest

The advertised source-root operation must either execute a bounded, receipt-producing implementation from explicit source-root inputs or be removed from executable command discovery and represented as an unsupported capability with a deterministic reason. An unconditional runtime error behind an apparently supported command is not accepted.

Source-root planning remains pure; filesystem discovery, capability probing, and execution remain in the shell.

### Require strict hermeticity for Onix release evidence

The Onix release profile selects strict hermetic mode and fails before pass evidence if sandbox, network, environment, path, clock, or declared-tool restrictions cannot be enforced. Practical mode is opt-in and may emit development or diagnostic receipts, but those receipts cannot satisfy strict release evidence fields.

### Dependency ordering

Local metadata blocks this package on `opaque-evidence-sidecar-binding`. Production consumption of authenticated Cairn handoffs requires the archived Cairn `authenticate-stack-provenance-inputs` receipt. The cross-repo prerequisite remains documented until Cairn cross-repository dependency metadata is available.

### CI scope

The checked-in workflow runs only `nix flake check`. Focused release, source-root, and hermeticity tests remain required implementation evidence outside the minimal CI command scope.

## Risks / Trade-offs

- Strict mode may be unavailable on some hosts; those hosts must report a blocker rather than emit release evidence.
- Removing an unavailable command can affect scripts, but an explicit capability query is more reliable than guaranteed runtime failure.
- Production wiring may expose stale existing fixtures; invalid handoff evidence should be repaired rather than bypassed.
