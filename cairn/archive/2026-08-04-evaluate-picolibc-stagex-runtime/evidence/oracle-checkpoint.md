# Oracle checkpoint: Picolibc for the StageX C runtime

Date: 2026-08-04. Owner: Mantle bootstrap maintainers (decision recorded by the agent session, subject to maintainer review).

## Question

Can the Picolibc 1.8.12 x86_64 Linux static profile reduce the early StageX
C runtime surface enough to justify a later protected-route proposal, and
does it satisfy the shared StageX libc behavior contract?

## Inspected evidence

- `evidence/pin-and-diagnostic-build.md`: pinned source (release 1.8.12,
  recursive sha256 `a2XLUN2U49Lpsyizzb2cQpMbww0cmOUUKdgIj6OHhpQ=`), signed
  nixpkgs toolchain closures, sandboxed build transcript, hello-world smoke.
- `evidence/comparison-report.json`: full typed comparison facts and the
  deterministic outcome.
- Diagnostic output manifest: 1,222 compiled units, tool roles (meson
  1.10.2, ninja 1.13.2, gcc 15.3.0, binutils 2.46), license classes from
  `COPYING.picolibc` (BSD-3-Clause dominant, BSD-2-Clause, FreeBSD,
  Other-permissive).
- Behavior matrix: positive 0/1 (invalid-signal case: `sigaction(0, ...)`
  does not return `EINVAL`; root cause `sig < 0 || sig >= _NSIG` in
  `libos/linux/sigaction.c`), negative 1/1 (malformed source rejected).
- Isolated-build identity: two fresh-state builds produced byte-identical
  trees, attestation BLAKE3
  `f0cc10766a6222c76753317d0c35fb3bcb973e9d44e8b999375bdf439fea4317`.
- Baseline: native-musl boundary at `origin/main` commit
  `5e56fbc10f06565226ab9dabb7238be0455d87d1`: 765 compiled units (746
  self-host + 19 predecessor), 106 recorded source-modification operations
  (9 removed directories, 23 removed files, 67 glob-matched files, 7 header
  declaration removals), canonical libc.a BLAKE3
  `87bea2db6aa8d4d34ec72427bbb9effad86ed6e0458bf4ce600b57d4966f92a7`.

## Decision

`rejected`. Picolibc fails the shared behavior contract and compiles more
translation units (1,222) than the native-musl baseline (765). ADR 0065
records the outcome. Provider selection, StageX lineage, bootstrap parity,
accepted provider digests, and release status are unchanged.

## Next action

None for this route. A future Picolibc release may re-enter only through a
new Cairn change with fresh construction, behavior, and admission evidence.
The comparison core and derivations stay in the tree as reusable diagnostic
machinery.

## Non-claims

This checkpoint does not claim Picolibc, musl, compiler, or kernel
correctness; does not admit Meson or Ninja into protected StageX execution;
and does not change release eligibility.
