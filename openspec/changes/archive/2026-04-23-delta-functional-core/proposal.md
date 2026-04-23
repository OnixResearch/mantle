# delta functional core

## Why

Crunch now has a proven first wave (`crunch-attestation-core`,
`crunch-project-core`) and second wave (`crunch-shell-core`,
`crunch-release-core`) for compiler-enforced functional-core / imperative-shell
splits, but delta transfer still keeps one valuable pure slice inside the mixed
`crunch-delta` crate.

Today `crates/crunch-delta/src/{model.rs,negotiation.rs,planner.rs}` hold pure
transfer-shape data, protocol negotiation, and reuse planning logic, yet the
crate also owns async store probing, HTTP/session framing, attestation
persistence, and other std-only shell work in `manifest.rs` and
`substitution.rs`.

That leaves three problems:

- the delta planner/protocol path still relies on social discipline instead of
  a compiler-enforced no-std boundary
- future edits can quietly pull store/network/runtime dependencies back into
  planning and negotiation logic
- the current pure slice still exposes std-shaped boundary types such as
  `HashSet` and `snix_castore::B3Digest`, so a straight file move would not be
  an honest no-std extraction

This change starts the third wave by making delta transfer planning a real
no-std core instead of a pure island inside a mixed crate.

## What Changes

- introduce `crunch-delta-core` as the third-wave dedicated no-std crate for
  delta transfer model, protocol negotiation, and transfer planning
- update the `functional-core`, `architecture`, and `portability` specs so the
  adopted no-std tier explicitly includes the delta domain
- require the delta core boundary to normalize digest/container types away from
  `snix_castore::B3Digest`, `HashSet`, async traits, `PathInfo`, and other
  std/runtime surfaces before the core call by introducing a core-local
  `DeltaDigest` newtype over 32 BLAKE3 bytes plus ordered `BTreeSet` / `Vec`
  traversal state
- keep `crunch-delta` as the std adaptor crate for fixture construction,
  manifest building, async substitution, HTTP framing, and store/attestation
  integration around the new core
- extend the no-std validation rail, adopted-core inventory, allowlist, and
  ownership review so the third wave is checked as strictly as the first two

## Capabilities

### New Capabilities

- `delta-core-tier`: delta transfer planning and protocol negotiation gain a
  compiler-enforced no-std home
- `delta-shell-boundary`: async substitution, manifest probing, and store/
  network integration become explicit std adaptor responsibilities around that
  core

### Modified Capabilities

- `compiler-enforced-functional-core`: adopted no-std coverage expands from the
  first two waves into delta transfer
- `workspace-architecture`: the workspace tiering now distinguishes
  `crunch-delta-core` from the std-facing `crunch-delta` shell
- `portability-boundary`: wasm/no-std compilation and dependency-boundary proof
  now include the delta core too

## Impact

- **Files**: new `crates/crunch-delta-core/`; updates to
  `crates/crunch-delta/src/{lib.rs,model.rs,negotiation.rs,planner.rs,manifest.rs,substitution.rs}`;
  validation updates under `scripts/` and
  `openspec/specs/functional-core/validation/`
- **APIs**: delta planning/protocol data moves behind new no-std-owned types;
  `crunch-delta` converts `snix_castore::B3Digest`, `HashSet`, `PathInfo`, and
  runtime transfer state to/from `DeltaDigest`, ordered `BTreeSet` membership,
  and typed core requests/results while remaining the std-facing
  facade/re-export surface
- **Dependencies**: the delta core must stay inside an approved no-std closure;
  std-only deps such as async/store/network crates remain in `crunch-delta`
- **Testing**: validation expands with host + wasm checks for
  `crunch-delta-core`, core positive/negative tests, and a std-shell boundary
  test for delta substitution

## Non-Goals

- moving `manifest.rs`, `substitution.rs`, or async remote-substitution
  orchestration into no-std
- rewriting all delta fixtures/bench helpers into the core in one pass
- changing delta wire semantics, chunk-profile negotiation, or reuse policy as
  the primary purpose of this change
- extracting store/build/runtime crates in the same wave
- implementing the change in this proposal step

## How to Validate

1. `openspec validate delta-functional-core` succeeds.
2. `openspec_gate stage=proposal change=delta-functional-core` is rerun and
   its current findings are attached before implementation starts.
3. The delta specs for `functional-core`, `architecture`, and `portability`
   all keep `MUST` language on each requirement’s opening line and attach
   dotted `ID:` lines plus scenarios.
4. The delta specs name `crunch-delta-core` as the third-wave core,
   `crunch-delta` as the std adaptor, and the normalization boundary for
   digests/collections/runtime types before the core call, including
   `DeltaDigest` as a 32-byte BLAKE3 newtype and ordered `BTreeSet` / `Vec`
   traversal inputs.
5. The proposal-stage validation transcript is saved under
   `openspec/changes/delta-functional-core/evidence/proposal-validation-2026-04-22.md`.
