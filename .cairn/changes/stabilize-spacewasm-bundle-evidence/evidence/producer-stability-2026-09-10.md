# Evidence: producer repair and stability proof (2026-09-10)

Task-ID: mantle.spacewasm_stable_evidence
Covers: contract, boundary, denial, rebuild

## Producer repair

`nix/spacewasm-reference.nix` `upstreamUnitTests` and
`upstreamSpectestAddress` now:

- run the harness with `-- --format json -Z unstable-options` under
  `RUSTC_BOOTSTRAP=1` (pinned stable toolchain exception, ADR 0079);
- emit `stable-report.json` from the checked-in
  `nix/spacewasm/stable-report.jq` canonicalizer (closed grammar, canonical
  name order, duplicate/contradiction rejection);
- bind `receipt.json` to the stable identity only — no run-specific digests;
- keep raw `stdout.txt` / `stderr.txt` as run-evidence members.

Grammar version 1 admits libtest `test/ok|failed|ignored`,
`test/started` (carried and ignored), and `suite/started|ok|failed` lines
with bounded summary counts, bounded `exec_time`, and bounded per-test
`stdout` capture. The Rust core and the jq lane admit the same field sets.

## Stability proof (frozen acceptance run)

Repaired derivation:
`/nix/store/zwzw2s2fpghrk82vc2d2qx218q7hik9p-spacewasm-e24cf09355a90497148eb5029fdb8e3400bd63e3-unit-tests.drv`

Command: `nix-store --realise <drv> --check --keep-failed --option builders ''`

    CHECK_EXIT=1
    output 'out' (...) differs from "(...).check"
    STABLE_BYTES_IDENTICAL

The check-mode rebuild diverged on the output path (raw run evidence still
varies, exactly as the baseline recorded) while `stable-report.json` and
`receipt.json` were byte-identical across both runs. Receipt BLAKE3
`3491ddeda12ee9c6f7aceaea17da5803767a1a45676717bb6d37690414cbd96c`;
stable identity `671a3937ec597dcef3c73aab14c7f4a9684ac92e494e5aca8e3c95691b635250`
(231 admitted tests), reproduced again by a later independent build.

## Rails

- `nix develop -c cargo test -p crunch-spacewasm-core`:
  `test result: ok. 8 passed` (focused core) and
  `test result: ok. 12 passed` (stable-report fixtures, including the
  Rust↔jq canonical-serialization parity fixture).
- `nix develop -c cargo clippy -p crunch-spacewasm-core --all-targets -- -D warnings`: exit 0.
- `nix build .#checks.x86_64-linux.spacewasm-reference-{bundle,fixtures,negative,upstream-unit-tests}`
  with `--option builders ''`: all exit 0.

## Open items (change stays active)

T2.4 shell bounded-capture module, T3.1 repository repeatability gate,
T4.1 producer publication, T4.2 ChaosControl-owned consumer evidence,
T4.3 policy registry blocker.

## Non-claims

Proves stable member bytes for this repaired producer under the recorded
derivation and command identities. Does not prove evaluator correctness,
consumer admission, or release eligibility.
