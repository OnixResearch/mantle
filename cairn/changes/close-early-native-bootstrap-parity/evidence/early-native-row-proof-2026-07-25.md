# Early-native row proof — 2026-07-25

## Scope

This evidence closes only the bounded `binutils.tcc` and `gcc.4.0` parity rows.
It does not claim general compiler or binutils correctness, provider or seed
admission, or whole-bootstrap correctness.

## Accepted artifacts

| Row | Logical output | Canonical artifact-attestation BLAKE3 | NAR SHA-256 |
|---|---|---|---|
| `binutils.tcc` | `/mantle/store/rm39hk4lfnaqzhcpazva7n17f58l5syl-binutils-2.30-tcc-source-v1` | `e0a40c67426126069e36508046c41cc6230afb93e90cc1e07f2255f9d62cb5a1` | `4062d1da2398a0a5103cc214b3cf0a6c8bfaf3418717d516e6c2a0e1773084e4` |
| `gcc.4.0` | `/mantle/store/7rfhqyf3dmv0ziwh8j6jl8jr9labmn9z-gcc-4.0.4-musl-cxx-v7` | `d48ce69b37a151cccaf5ee756362806b517c4427481581063a2ddc7bf8f7d0ac` | `a63f40ae6b88edb4b219b1423027d341685bf296384f4619885101631954b96b` |

The durable output envelopes are
`bootstrap/evidence/early-native-binutils-artifact.json` and
`bootstrap/evidence/early-native-gcc40-artifact.json`. The validator
recanonicalizes each attestation and compares its logical path, canonical
BLAKE3, and NAR digest to the row receipt.

## Independent closure and acceptance observations

The row receipts are:

- `bootstrap/evidence/early-native-binutils-row-v2.json`
- `bootstrap/evidence/early-native-gcc40-row-v2.json`

Validation does not accept predecessor digest strings or receipt booleans by
themselves. It independently:

1. hashes every contracted current Nickel source with BLAKE3;
2. loads every envelope under
   `bootstrap/evidence/early-native-predecessors/`, canonicalizes the artifact
   attestation, and compares role, logical path, and digest;
3. loads the row-specific acceptance evidence, compares the positive and
   rejection matrices, and requires the runtime fingerprint to record
   `trust_unsigned=false`, `signing_key_selected=true`, and substitutions
   disabled;
4. scans the final row source for required matrix/bound markers and forbidden
   ambient-host, TinyCC-delegation, fabricated/non-admission markers; and
5. canonicalizes the final output artifact envelope.

The final GCC build removed inherited ambient `PATH`; its declared PATH now
contains only the bounded BusyBox applet directory and declared derivation
outputs. Task `1617` rebuilt the signed, no-substitution GCC root after that
change. Task `1634` verified the resulting artifact attestation and printed:

```text
OK artifact digest=d48ce69b37a151cccaf5ee756362806b517c4427481581063a2ddc7bf8f7d0ac
```

Both strict runtime fingerprints record `substitution_mode="disabled"`,
`signing_key_selected=true`, and `trust_unsigned=false`. The accepted GCC
recipe log is
`sgssv3i19y3f4gkiziaq9amm3n62jwvb-gcc-4.0.4-musl-cxx-v7.drv.log`; it ends
with `GCC 4.0.4 regenerated C/C++ early-native row behavior matrix passed`.
The accepted binutils recipe log is
`66rwlnpidlrgnqwlpxs4jh3wj7jq1jdw-binutils-2.30-tcc-source-v1.drv.log`; it
records `source-built TCC-era binutils matrix passed`.

The embedded Bison patch intentionally preserves upstream tabs and blank
context lines. `.gitattributes` scopes Git whitespace handling for that file.
A diagnostic rewrite attempt (task `1681`) failed and was not used as evidence;
the exact previously receipted source bytes were restored. Task `1688` then
accepted the current Bison root from the signed no-substitution state at
`/mantle/store/4ag6mwv2pmsa0b8wii2zwc4jws5iajbk-bison-2.3-musl`.

## Verification completed

- Task `1647`: `nix develop -c cargo test -p mantle --bin mantle bootstrap_parity -- --nocapture`
  — `88 passed; 0 failed`.
- Task `1656`: `nix develop -c cargo test -p mantle --test bootstrap_parity_cli -- --nocapture`
  — `16 passed; 0 failed`.
- The CLI suite includes positive production-row coverage and negative cases
  for malformed receipts, stale source digests, stale generated-artifact
  status, cross-row substitution, corrupt predecessor attestations, missing
  output evidence, forbidden host discovery, wrapper delegation, and an
  untrusted runtime fingerprint.
- Task `1694`: `nix develop -c ./scripts/check-gcc40-configure-bridge.rs --self-test`
  — `PASS (classes=3, source_spellings=2, source_bytes_max=65536, invocation_count_max=4096)`.
- Task `1697`: direct nightly Cargo-script execution of
  `scripts/check-bootstrap-source-pins.rs` exited successfully.
- Task `1687`: `git diff --check` exited successfully.
- Task `1720`: `nix develop -c cargo test -p mantle --test bootstrap_eval -- --nocapture`
  completed all 30 tests successfully before the chained formatting leg reported
  one stale formatting diff; task `1729` applied formatting and the subsequent
  `cargo fmt --check -p mantle -v` leg passed.
- Task `1745`: focused first-party
  `cargo clippy -p mantle --bin mantle --test bootstrap_parity_cli --no-deps -- -D warnings`
  passed. The visible `snix-castore` dead-code warning is dependency output and
  was outside the `--no-deps` first-party lint surface.
- Task `1742`: canonical Cairn validation recorded `valid: true`; proposal,
  design, and tasks gates each recorded `valid: true` and `verdict: "PASS"`.

## Remaining non-claims and downstream blockers

The configure preprocessing bridge remains confined to the audited
`conftest.c` authority and is not a provider preprocessor. Intermediate
same-version bootstrap artifacts remain explicit predecessors rather than
being retroactively promoted. GCC 4.7, GCC 10, full-musl binutils, full-source
Rust qualification, StageX lineage, deterministic fixed-point proof, and
whole-bootstrap promotion remain separate downstream work.
