# Behavior, identity, and decision evidence (I4, I5, V1, V2)

Date: 2026-08-04. Worktree `.pi/worktrees/evaluate-picolibc-stagex-runtime`, branch `cairn/evaluate-picolibc-stagex-runtime`.

## Behavior matrix (I4)

`bootstrap/picolibc-1.8.12-behavior.ncl` compiles the exact StageX
native-musl smoke program plus the shared malformed source against the
diagnostic output and runs the binary inside the Mantle sandbox.

Result (`behavior.json`, output
`0xg7f0dgmj2v6pd1sdz5vcw9d5f6ixim-picolibc-1.8.12-x86_64-linux-behavior`):

```json
{
  "positive_passed": 0,
  "positive_total": 1,
  "negative_passed": 1,
  "negative_total": 1,
  "failures": ["runtime mismatch: status=16 output="]
}
```

Status 16 is the contract's `failure_invalid_signal` case: `sigaction(0,
...)` must return -1 with `EINVAL`. Picolibc's userspace range check is
`sig < 0 || sig >= _NSIG` (`libos/linux/sigaction.c`), which accepts signal
0; musl rejects it. The malformed-source rejection case passed.

Additional recorded finding: linking against `crt0-linux.o` emits
`requires executable stack` (its `.note.GNU-stack` section is executable).

## Isolated-build identity (I4)

Two builds of `bootstrap/picolibc-1.8.12-diagnostic.ncl` with fresh state,
output, and scratch roots. The diagnostic serializes ninja (`-j1`) after the
first run showed compile-order noise in `ninja-build.log` only; all runtime
artifacts were already byte-identical before serialization.

Final run: `diff -r` of the two output trees is empty, and both attestation
digests are BLAKE3 `f0cc10766a6222c76753317d0c35fb3bcb973e9d44e8b999375bdf439fea4317`.

Output paths (same logical path in both stores, as expected for an
input-addressed derivation):
`6ynchn4qdvwy5fsn7pw59yiya2kxdd8m-picolibc-1.8.12-x86_64-linux-diagnostic`
under `~/mantle-fix-producer/picolibc-nix-store` and
`~/mantle-fix-producer/picolibc-iso-store`.

## Baseline facts (I5)

Native-musl boundary at `origin/main` `5e56fbc10f06565226ab9dabb7238be0455d87d1`:

- 765 compiled units (746 self-host + 19 predecessor, per
  `src/stagex_musl_native.rs` `SELFHOST_COMPILE_COUNT` /
  `PREDECESSOR_COMPILE_COUNT`)
- 106 recorded source-modification operations: 9 removed directories, 23
  removed files, 67 glob-matched files (counted against the musl-1.1.24
  release tree), 7 header declaration removals
- Canonical libc.a BLAKE3
  `87bea2db6aa8d4d34ec72427bbb9effad86ed6e0458bf4ce600b57d4966f92a7`

## Decision (I5)

```text
outcome=rejected
reason=behavior matrix failed: 0/1 positive, 1/1 negative
reason=no surface reduction: 1222 diagnostic units vs 765 baseline units
```

Full report: `evidence/comparison-report.json`. ADR: `adr/0065`. Oracle
checkpoint: `evidence/oracle-checkpoint.md`. No provider, parity, lineage,
or release state was modified.

## Self-tests (V1)

- Core: `cargo test -p mantle --bin mantle picolibc_comparison` → 17 passed
  (all three outcomes, malformed, oversized, contradictory, nondeterministic
  inputs).
- Shell: `picolibc_compare --self-test` → `self-test-ok` (manifest and
  behavior parsing, candidate/blocked/rejected paths).

## Transcripts (V2)

- meson-setup.log, ninja-build.log, ninja-install.log and the fact manifests
  are inside the diagnostic output under
  `share/picolibc-diagnostic/`.
- The behavior output preserves smoke.c, malformed.c, both compiler stderr
  captures, the smoke binary, and behavior.json under
  `share/picolibc-behavior/`.
