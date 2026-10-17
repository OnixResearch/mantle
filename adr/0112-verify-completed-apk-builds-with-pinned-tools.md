# ADR 0112: Verify completed APK builds with pinned tools

- **Status:** Proposed
- **Date:** 2026-10-11

## Context

The official APK proof copies the complete reviewed source to a read-only
snapshot before any SDK build. Its first execution completed three independent
Bubblewrap builds with real Google and Temurin tools. The post-build verifier
then called `nix develop` again from that snapshot; this reevaluated a large
flake and tried to realize offline-unavailable inputs before it could run the
already extracted reviewed `apksigner`. During the long run, host Nix GC also
removed the unrooted first Mantle executable, despite its SHA-256 pin.
Neither condition invalidates the observed builds, but neither permits a
success receipt without a new independent verification of their artifacts.

## Decision drivers

- A verified build result must not become a success receipt solely because a
  test log says it passed. Real reviewed `apksigner verify`, structural checks,
  matched clean digests, changed-Java digest, and a signed-region tamper
  failure remain mandatory.
- A fresh Cargo target and complete immutable execution source must be bound
  before dispatch; a verifier must not synthesize missing builds.
- Reentering a development shell from a copied source changes its Nix input
  graph and may depend on uncached unrelated packages.
- A long-running pin to a Nix store path is not a GC root.

## Decision

The fresh builder enters the locked development shell once before launching
the read-only snapshot runner. The runner invokes commands from that
pre-entered shell and verifies a separately supplied, signed Bubblewrap
output for preflight, test-only keytool and post-build `apksigner`. The
operator GC-roots the independently pinned first Mantle executable and
signed verifier tools in private scratch for the duration of the proof.

If post-build verification is interrupted, a separate checked-in verifier
runs from a second, independently pinned, read-only full-source snapshot.
It rechecks the original execution snapshot, the original-HTTPS archives and
bundle, the first executable, signed verifier tools and runtime, the offline
fresh-source replay, the exact passing ignored test log and all three
distinct private store/state observations. It then invokes the extracted
reviewed `apksigner` under signed Bubblewrap, compares clean and perturbed
APK digests, and proves tamper rejection before writing the same bounded
success-receipt schema with both source snapshot identities. It never
rebuilds, invents missing observations, or promotes a test log to trust.

## Consequences

The verifier can complete an interrupted real run without a second SDK build,
but only while all source snapshots, the original execution artifacts and
signed tools remain available and pass fresh checks. Missing original
observations or a verifier error produce a blocker and no success receipt.
The private GC roots and retained scratch run must outlive verification;
neither is a release claim or an authorization to use production keys.
