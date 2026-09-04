# Current vendor-source blocker and repair

Date: 2026-09-04

## Blocker

The first source-profile refresh failed because `vendor-deps/` was absent from
the transferred source tree. The second and third attempts used the V98 and
current local vendor trees. Both failed before profile output with:

```text
error: source-built profile vendor preflight failed: error: duplicate vendored package in Cargo.lock: artifact-auth-core 0.1.0
```

The current lock has two immutable sources for the same package name and
version:

- direct Mantle source: Artifact revision
  `c932138d880ddf4c2967f4c024b489b5c0022bf1`;
- Valence source: Artifact revision
  `e41340bec587b6d049b5cc518ec7db925dde84be`.

A direct `cargo vendor --locked --offline --versioned-dirs` probe also rejected
that pair. Its diagnostic is in `cargo-vendor-duplicate-source-probe.log`.

The copied vendor tree was stale for newer Valence, bounded-tree, transaction,
Nickel, and registry dependencies. Therefore, adding only the missing Artifact
package was not sufficient.

## Repair

ADR 0120 keeps both accepted source revisions. It does not patch, downgrade, or
replace either dependency.

The Cargo config now routes the Valence-selected Artifact revision and other
locked sources to `vendor-deps/`. It routes Mantle's direct Artifact revision
to `vendor-deps/.artifact-auth-c932/`.

The host-tool-free guard now parses that source map. It requires every directory
source to remain below `vendor-deps/`. It maps each locked source to exactly one
root and validates each root independently. The native Rust planner now reads
`.cargo/vendor-config.toml` as declared source material.

A disposable source copy produced a current main vendor root after replacing
the direct Artifact revision only in that generation copy. The final vendor
root removes the unneeded main-root Ed25519 package and adds the exact direct
`c932138d...` Core and Ed25519 packages under the nested source. The committed
Mantle manifests and lockfile were not changed.

## Evidence

- Empty-`CARGO_HOME`, locked, offline Cargo metadata passed against the composed
  tree: 3,664,088 output bytes.
- The composed vendor payload is 846,041,532 bytes.
- The current guard validated the complete tree and reached the intentionally
  missing source-profile read. It did not report a vendor preflight error.
- Guard tests: 12 passed, including separate-root acceptance and same-root
  duplicate rejection.
- Native vendor-root tests: 2 passed, including malformed-config rejection.
- Source-built fixed-point tests: 84 passed and 3 long tests remained ignored.
- Strict first-party Clippy and root-package formatting passed.
- The pinned Tiger Style Nix gate passed. Its first invocation built the check,
  then remained in post-build auto-GC after reporting 142.6 GiB of hard-link
  savings. That invocation was stopped and preserved. The cached rerun passed
  with automatic free-space work disabled.

The repair proves exact offline Cargo-source availability for this lockfile. It
does not change package semantics or make a promoted proof claim.
