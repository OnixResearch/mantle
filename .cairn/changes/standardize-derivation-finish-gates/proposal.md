# Proposal: Standardize derivation finish gates

## Why

Every Mantle bootstrap recipe carries its own hand-written smoke checks: a
`--version` run, a copied-tree relocation run, a malformed-input rejection. The
checks differ per recipe, they are optional, and the repository record shows the
exact failure class they miss: published GCC wrappers that retained
`/tmp/...binutils/bin/ld`, a TinyCC compiler that embedded logical store paths,
and prefix-sensitive smokes that looked in the wrong store path under
content-addressed provisionals. Each of these was found manually, late, and per
recipe.

The reviewed external reference (`mic92/repkgs`, commit `1cd7b8b`, see
`evidence/repkgs-review.md`) shows the alternative: one shared finish phase that
ends every build the same way. Its version check, absolute-reference leak gate,
relocation rerun, and dlopen audit catch broken installs mechanically instead of
per recipe.

## What Changes

- Define a versioned finish-gate contract in typed Nickel policy with a
  deterministic export consumed by the Rust shell. r[mantle.derivation_finish_gates.shared_contract]
- Add a version gate: a declared command must run in an empty environment and
  print the pinned version. r[mantle.derivation_finish_gates.version_check]
- Add a reference leak gate: absolute logical store-prefix references in
  outputs are reported per policy, and cross builds must fail on any
  build-platform reference. r[mantle.derivation_finish_gates.reference_leak_gate]
- Add an opt-in relocation gate that reruns the version check from a copy of
  the output at another path. r[mantle.derivation_finish_gates.relocated_rerun]
- Add an opt-in dlopen audit gate that fails the build when a runtime dlopen
  finds no provider, unless the soname is declared optional.
  r[mantle.derivation_finish_gates.dlopen_audit]

## Impact

- **Immediate consumer**: Mantle bootstrap recipes under `bootstrap/` and the
  `builders/` layer; long-running source-built fixed-point proofs that today
  depend on ad-hoc smokes.
- **Immediate outcome**: a new derivation defaults to version, leak, and
  output-shape checks without writing per-recipe shell.
- **Durable capability**: one checked definition of "this output is installed
  correctly" shared by every derivation, with explicit opt-outs instead of
  silent omission.
- **Maintenance owner**: Mantle builder-layer owner, covering
  `builders/mk_derivation.ncl`, `lib/derivation.ncl`, and the finish shell in
  the root package.
- **Repeatability evidence**: positive fixtures (passing gates on a known-good
  output) and negative fixtures (wrong version output, leaked prefix,
  unresolved dlopen, non-relocatable output) under the existing test suites.
- **Compatibility**: existing recipes keep working unchanged; gates are
  additive and defaulted on only for new derivations or explicit adoption.

## Scope

The change covers the gate contract, the shared finish shell, typed Nickel
policy, per-gate opt-outs, build-report surfacing, and adoption of the gates in
the bootstrap recipe family. It covers positive and negative fixtures for every
gate.

## Out of Scope

- Relocating output references themselves (owned by
  `relocate-dynamic-output-references`).
- Compiler correctness, runtime behavior beyond the declared checks, or
  release eligibility.
- Nushell or any new build language; gates run in the existing shell path.
- Changing derivation hashing or the closed core derivation contract beyond
  the new optional fields.

## Success Criteria

- A derivation that installs a binary printing the wrong version fails the
  finish gate with the exact command and expected string in the error.
- An output containing an absolute reference to a denied store path fails or is
  reported per policy, cross builds always failing.
- A passing output passes all enabled gates; each gate has a negative fixture
  that fails closed.
- Existing checked-in recipes evaluate and build unchanged before adoption.
