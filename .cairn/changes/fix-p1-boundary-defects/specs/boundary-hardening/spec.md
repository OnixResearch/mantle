# Boundary Hardening Delta

These requirements record the P1 boundary behavior fixed by this change. They
are target requirements; the focused regressions under `evidence/` are the
current verification surface.

## ADDED Requirements

### Requirement: Private key material never crosses a diagnostic boundary

r[boundary_hardening.private_key_diagnostics] Signing-key loading MUST admit
only bounded regular files without group or other permission bits, MUST NOT
follow symlinks, MUST revalidate file identity before reading, and MUST read
through zeroizing buffers. A malformed encoded keypair MUST produce a stable
redacted error, and the declared public half MUST match the secret half.

#### Scenario: A malformed keypair is rejected

GIVEN an encoded keypair of the expected length with an invalid base64 byte
WHEN the keypair is parsed
THEN the error MUST name only the failure class
AND it MUST NOT contain the encoded input, the secret half, or the file text.

#### Scenario: A loose or substituted key file is rejected

GIVEN a signing-key path that is a symlink, a non-regular file, larger than the
bound, or carries group or other permission bits
WHEN the loader is invoked
THEN it MUST fail before reading the contents
AND it MUST NOT admit a partially published or substituted key.

### Requirement: GC preserves an explicit unsigned admission

r[boundary_hardening.unsigned_gc] GC MUST preserve retained PathInfo trust
status verbatim. An unsigned PathInfo admitted through explicit policy MUST
remain stored and reachable after plan-bound GC; GC MUST NOT sign it, drop it,
or panic. The build-output persistence path MAY continue to refuse unsigned
results.

#### Scenario: An unsigned root undergoes GC

GIVEN a Snix state directory with an unsigned PathInfo registered as a retained
root and an unrelated reclaimable path
WHEN a GC plan is executed
THEN the unsigned root MUST remain present with an empty signature list
AND the unrelated path MUST be reclaimed.

### Requirement: Worker liveness survives service panic and unobserved teardown

r[boundary_hardening.worker_liveness] A cancellable build-service panic or an
unobserved fetch/sandbox teardown MUST complete the worker task with a failed
outcome. The service MAY retain its quarantine and lease until teardown is
observed; the worker MUST NOT replace the task with an unobservable permanent
pending future.

#### Scenario: A cancellable service panics before spawn

GIVEN a BuildService whose cancellable method panics
WHEN the worker runs the goal
THEN the goal MUST be classified as failed within a bounded deadline
AND watch shutdown MUST drain without publishing an outcome or PathInfo.

#### Scenario: Environment bytes are not UTF-8

GIVEN a sandbox environment value with non-UTF-8 Unix bytes
WHEN the sandbox is initialized
THEN the bytes MUST be preserved on Unix
AND NUL or an oversized value MUST be rejected with a typed error before spawn.

### Requirement: Sandbox diagnostics and acquisition bytes are bounded

r[boundary_hardening.bounded_diagnostics] Sandbox stdout and stderr MUST be
drained under an explicit per-stream bound. An over-limit stream MUST retain a
bounded prefix, continue draining to avoid pipe deadlock, and append an
explicit truncation marker.

r[boundary_hardening.bounded_acquisition] Tar and flat acquisition MUST enforce
separate compressed, decompressed, per-entry, and cumulative byte budgets while
consuming bytes. An over-limit input MUST return a distinct limit error, clean
partial public output, and MUST NOT admit a result. An exact-limit input MUST
succeed.

#### Scenario: A one-entry archive exceeds the entry budget

GIVEN a valid archive with one regular entry larger than the declared per-entry
budget
WHEN extraction runs
THEN it MUST fail with a limit error before creating the oversized entry
AND no admitted output may be published.

#### Scenario: A flat response exceeds the download budget

GIVEN a flat response one byte larger than the declared download budget
WHEN acquisition runs
THEN it MUST fail explicitly rather than treating the truncation as EOF
AND an exact-limit response MUST succeed.

### Requirement: Clean-runner flake evaluation is credential-free

r[boundary_hardening.clean_runner_evaluation] Flake evaluation MUST NOT require
an SSH agent, SSH key, or Git URL rewrite for a pinned public dependency. A
transitive manifest that still names SSH MUST be redirected to the identical
HTTPS revision at the Cargo source boundary.

#### Scenario: A transitive source still names SSH

GIVEN a pinned git dependency whose own manifest names a transitive artifact
source over SSH
WHEN a clean runner evaluates the flake and resolves the workspace
THEN the effective lock MUST contain only the HTTPS source for that revision
AND no SSH transport or rewrite MUST be needed.

### Requirement: Boundary changes retain focused evidence

r[boundary_hardening.change_evidence] Each boundary change MUST retain focused
positive and negative regression output and a scoped quality-gate result. A
skipped or unavailable check MUST be recorded as a blocker, not as a passing
claim.

#### Scenario: A focused regression is recorded

GIVEN the changed boundary
WHEN its focused tests and scoped gate run
THEN the change evidence MUST quote the exact commands and results
AND it MUST NOT promote those results into arbitrary correctness claims.
