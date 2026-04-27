Task-ID: V3
Covers: bootstrap.gcc40.transition

Status: blocked.

## Blocker

Depends on V2 build transcript and `./scripts/check-bootstrap-transcript.rs`
(deferred to parent live-bootstrap-source-chain task I10).

## Required when unblocked

Command: `./scripts/check-bootstrap-transcript.rs --reject-host-tools evidence/V2-build.md`
Verify: no host compiler, host libc, host shell, Nix command, or legacy
provider path leakage in the gcc-4.0.4 build transcript.

Verified: 2026-04-27 (blocker recorded)
