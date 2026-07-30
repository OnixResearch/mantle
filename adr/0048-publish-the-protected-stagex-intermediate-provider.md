# ADR 0048: Publish the protected StageX intermediate provider

## Status

Accepted (2026-07-30)

## Context

The protected StageX transition produces a self-hosted TinyCC 0.9.27, native-musl headers and static libraries, and authenticated binutils 2.30 tools. These components satisfy the smallest useful normalized provider boundary. They do not supply a final native GCC provider.

The former CLI only validated the lineage manifest. It returned a materialization placeholder and kept a scaffold-only receipt. A passing transition report did not publish a provider.

## Decision Drivers

- Publish only observed and authenticated transition outputs.
- Keep final GCC admission and compiler correctness outside this claim.
- Prevent stale-root discovery and destination replacement.
- Validate relocated tools without ambient executable authority.
- Keep receipt identities deterministic across output locations.

## Decision

Mantle publishes a bounded intermediate provider from three explicit absolute inputs: the lineage manifest, one complete protected transition root, and an absent output path.

The provider contains:

- self-hosted target-prefixed TinyCC and its runtime;
- native-musl headers, CRT objects, and static libraries;
- 11 target-prefixed binutils tools, headers, and linker scripts;
- provider metadata, a protected relocation-validation report, and the complete lineage receipt.

Mantle creates a private staging directory. It rejects symlinks and unsupported file types. It verifies live component identities against the transition report and retained inventories. It runs static TinyCC and binutils smokes under a new seccomp policy that authorizes exact staged paths and BLAKE3 digests. Generated smoke executables require explicit promotion before execution.

The validation audit projects staging paths to provider-relative paths. Each complete stage report binds its declared authorization set only after the protected audit contains each exact executable path and digest identity from an intercepted `execve` or `execveat` decision. One event can satisfy equivalent authorization IDs, but an unused identity fails publication. This proves an execution decision, not successful process completion. The receipt binds the full transition plan, full transition report, protected audit, provider payload, four provider roles, full validation report, validation audit, stage reports, bounded claims, and non-claims. A domain-separated receipt-payload digest rejects stale or edited receipt fields. Exact allowlisted report-only observations use a separate domain-separated BLAKE3 projection over the artifact ID, plan digest, and complete report digest. Live provider components always use observed file or tree identities.

Mantle validates staging, publishes with Linux no-replace rename, and validates the emitted provider again. It never removes or overwrites an existing destination.

## Alternatives Considered

### Wait for final native GCC

Rejected because the current TinyCC/native-musl/binutils closure already satisfies an honest intermediate provider contract. Waiting would combine distinct claim boundaries.

### Infer the transition root

Rejected because checkout, environment, store, or output-path discovery can select stale evidence.

### Publish after a transition report passes

Rejected because transition success does not validate normalized layout, relocation, receipt linkage, or atomic publication.

### Copy over an existing destination

Rejected because replacement can hide stale or competing authority and can expose partial output.

## Consequences

- `seed-full.stagex-lineage` can become complete from a complete intermediate-provider receipt.
- Mantle self-build remains a separate blocker.
- Final native GCC admission remains separate evidence.
- Publication requires Linux seccomp user notification and no-replace rename support.
- The provider does not prove compiler correctness, complete musl or binutils behavior, kernel isolation, reproducibility, or release eligibility.
