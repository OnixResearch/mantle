# 2026-11-06: source-qualified consuming-lock identities

## Outcome and owner

The successor resolver now retains source qualification and the exact selected
consuming-lock owner through transitive edges and feature revisits. Its current
consumer is the Cargo source topology in Rust 1.93.1/1.94.1, where local and
registry `home` packages share version 0.5.12. Mantle's Rust source-authoring
owners maintain this mechanism; [ADR 0145](../../../../adr/0145-retain-source-qualified-consuming-lock-identities.md)
records the decision. Exported manifest/host keys are unchanged, and lockfiles
are borrowed rather than cloned into every package or key.

This work is over base `7cd4e9cbb289eaf4277d223a67b5d40ea0e55c0a`, not a frozen
provider/proof cohort. Final source and CLI identities are retained in
`2026-11-06-lock-source-identities.txt`; the release executable's BLAKE3 is
`f7256b337ddf68b1f2db80c65190bc7450766d88edafcc5a3c199c543caea165`.

## Original-source diagnosis

The earlier native continuation preparation passed the pre-LLVM capacity
boundary and stopped at `ambiguous original locked identity home`. Inspection
of all admitted successor archives found that the earlier cohorts have different
local/registry home versions, while the later Cargo locks contain two home
records at 0.5.12. Their selected Cargo edge is explicitly
`home 0.5.12 (registry+https://github.com/rust-lang/crates.io-index)`.
The old resolver discarded that source token.

Original archive BLAKE3 identities and exact package/owner edge observations are
in `2026-11-06-lock-source-original-facts.json`. The original Cargo lock decoder,
not its general PackageIdSpec matcher, supplies the lookup rule: admit version
first; omitted source prefers a unique path identity, otherwise requires a
single remaining identity. `2026-11-06-lock-source-original-grammar.json` retains
actual source excerpts from Cargo's `TomlLockfilePackageId::from_str` and
`lookup_id`, with archive/member/line provenance.

A source filter alone would not fix the complete transition. The old resolver
also selected an owner's children from the first matching name/version record.
A regression with distinct local/registry child versions observed the wrong
child even when the local path was already known. Exact borrowed owner state
now accompanies the worklist and feature requests. Conflicting authorities
behind one manifest/host key reject before feature deduplication.

Complete source tokens are consumed and validated, then matched without source
fallback. Version ambiguity, selected duplicate identities, source mismatch,
malformed qualifiers and trailing token data remain failures. The existing
consuming workspace lock/patch boundary, fixed target policy, source/archive
pins, feature semantics and resource ceilings are unchanged. No dependency-local
lock fallback or version-order selection was added.

## Independent host diagnostic

An isolated offline Cargo 1.97.1 metadata probe used authentic memchr 2.7.6
vendor contents/checksum from the admitted Rust 1.94.1 archive, alongside a
same-name/version local package with distinguishable children. Cargo emitted a
source-qualified registry edge and a local name/version edge; directory source
replacement preserved registry identity. A unique-registry control admitted a
bare name, confirming that absent source is not universally a local marker.
Lock-update refusals are recorded separately from parse errors.

`2026-11-06-lock-source-cargo-oracle.json` retains exact tool/command/environment,
archive provenance, identities, mutation outcomes and the observed summary.
The original fixture/vendor trees and raw results remain under
`/home/brittonr/mantle-lock-source-20261106/`; temporary Python launch scripts
were removed after capture. This is a host semantic diagnostic, not protected
Cargo-free execution, provider construction or bootstrap evidence.

## Exercised regressions and checks

| Run | Passed | Failed | Observed scope |
|---|---:|---:|---|
| Pueue 36, `lock-source-before` | 17 | 3 | Source qualification rejected; local owner chose the registry child; malformed source text was ignored |
| Pueue 37, `lock-source-after` | 22 | 0 | Correct registry/local transitive owners, qualifier denials, exact-identity duplicates, source-omission precedence, conflicting-owner denial and existing lock/cfg regressions |

The baseline ran 19:39:13–19:43:50 -0500, exit 101; its full transcript is
`2026-11-06-lock-source-before.log`. Two additional source-omission/authority
boundary tests were added after that baseline. The repaired run was
19:48:13–19:59:16 -0500, exit 0, and also passed focused release Clippy
(`-p mantle --bin mantle --no-deps -- -D warnings`) and built the release CLI.
Its full transcript is `2026-11-06-lock-source-after.log`.

Changed Rust leaves passed leaf-only rustfmt checking. The operator-guide
checker passed; its command/output are in `2026-11-06-lock-source-doc-check.log`.
The ordinary workspace quality rail remains unaccepted due to its previously
captured unrelated formatting failure; that known failure was not rerun.

No prepared profile, provider execution, stage1/stage2 equality, full-source
fixed point, deterministic proof receipt, full parity or release eligibility is
accepted by these regressions or host diagnostics. I3–I5 and V1–V3 remain unchecked.

## Real preparation

Pueue 38 (`lock-source-real-preparation`, 19:59:16–20:08:38 -0500), exit 1,
ran the unchanged authenticated `/home/brittonr/mantle-cold-20261105/prep.sh`
after the successful release build. Source/profile/lineage/native-provider pins,
source reservations and resource limits were unchanged. The binary and launcher
BLAKE3 identities are retained in `2026-11-06-lock-source-real-preparation.log`.
Preparation passed the prior `home` lock ambiguity and reached:

```json
{"error":"source recipe: non-normalized relative path \"x86_64-linux-musl/lib/\"","code":1,"kind":"build"}
```

This is a separate typed-operand boundary. The directory suffix must be retained
using the existing typed separator contract, not admitted by weakening normalized
relative-path validation or silently rewriting the original environment value.
No prepared profile or provider/fixed-point acceptance is implied.

## Lifecycle check

Pinned-shell Cairn validation returned exit 0, `valid: true`, with no issues or
findings; the full report is `2026-11-06-lock-source-cairn-validate.json`.
Structural validation is not implementation/proof acceptance or archive authority.
Temporary parent validation/preparation launch scripts were removed after capture.

