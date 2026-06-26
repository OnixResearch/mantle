# Provider-bound witness replay validation

Date: 2026-06-26
Change: `witness-provider-bound-replay`
Task-ID: V1
Covers: r[verification_evidence.release_witness_rebuild_multi_output]

## Summary

Implemented a witness-owned provider fixed-point proof candidate path for `mantle release witness-rebuild`. The workflow driver now receives the extracted source tree path and a provider proof output directory. If the workflow writes a provider fixed-point proof there, `witness_rebuild` validates it with the existing provider proof verifier, requires stage binary paths to canonicalize under the witness-owned provider proof directory, and only selects those binaries by matching BLAKE3 digest against signed release outputs.

This proves the collection/signing boundary and helper preflight support. It does **not** prove a fresh independent rebuild of `provider-bound-release-evidence-2026-06-26`; the retained full-helper audit still shows an upstream/network fetch blocker before witness sidecar creation.

## Validation commands

### Helper syntax

Command:

```sh
bash -n scripts/rebuild-witness-request.sh
```

Output:

```text
Task 65: completed successfully
```

### Focused unit tests

Command:

```sh
PATH=/home/brittonr/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin:/nix/store/97vplpbajnr7x03fqh9biz5v6960sv22-clang-wrapper-21.1.8/bin:/nix/store/1sw8whfl5gfblp6r9qdkiw1b4j9fgwar-mold-2.40.4/bin:/nix/store/rvp7qlpf5jqvdckjy1afjb6aha6j8dxg-pkg-config-wrapper-0.29.2/bin:$PATH PKG_CONFIG_PATH=/nix/store/1l5jgzy26hkjz1y3apn1051asvn42sfn-openssl-3.6.1-dev/lib/pkgconfig SNIX_BUILD_SANDBOX_SHELL=/bin/sh cargo test -p mantle --bin mantle witness_rebuild -- --nocapture
```

Output excerpt:

```text
test witness_rebuild::tests::collect_rebuilt_output_paths_rejects_invalid_provider_fixed_point_proof ... ok
test witness_rebuild::tests::collect_rebuilt_output_paths_rejects_provider_fixed_point_stage_escape ... ok
test witness_rebuild::tests::collect_rebuilt_output_paths_uses_provider_fixed_point_candidate_by_digest ... ok

test result: ok. 20 passed; 0 failed; 0 ignored; 0 measured; 859 filtered out; finished in 0.01s
```

### Release CLI/helper tests

Command:

```sh
PATH=/home/brittonr/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin:/nix/store/97vplpbajnr7x03fqh9biz5v6960sv22-clang-wrapper-21.1.8/bin:/nix/store/1sw8whfl5gfblp6r9qdkiw1b4j9fgwar-mold-2.40.4/bin:/nix/store/rvp7qlpf5jqvdckjy1afjb6aha6j8dxg-pkg-config-wrapper-0.29.2/bin:$PATH PKG_CONFIG_PATH=/nix/store/1l5jgzy26hkjz1y3apn1051asvn42sfn-openssl-3.6.1-dev/lib/pkgconfig SNIX_BUILD_SANDBOX_SHELL=/bin/sh cargo test -p mantle --test release_cli witness_rebuild -- --nocapture
```

Output excerpt:

```text
test witness_rebuild_helper_rejects_incomplete_provider_bound_inputs ... ok
test witness_rebuild_helper_check_accepts_provider_bound_inputs ... ok
test witness_rebuild_helper_check_is_preflight_only ... ok
test witness_rebuild_helper_happy_path_writes_sidecars_and_audit ... ok

test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 89 filtered out; finished in 0.39s
```

### Binary build

Command:

```sh
PATH=/home/brittonr/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin:/nix/store/97vplpbajnr7x03fqh9biz5v6960sv22-clang-wrapper-21.1.8/bin:/nix/store/1sw8whfl5gfblp6r9qdkiw1b4j9fgwar-mold-2.40.4/bin:/nix/store/rvp7qlpf5jqvdckjy1afjb6aha6j8dxg-pkg-config-wrapper-0.29.2/bin:$PATH PKG_CONFIG_PATH=/nix/store/1l5jgzy26hkjz1y3apn1051asvn42sfn-openssl-3.6.1-dev/lib/pkgconfig SNIX_BUILD_SANDBOX_SHELL=/bin/sh cargo build -p mantle --bin mantle
```

Output excerpt:

```text
Compiling mantle v0.1.0 (/home/brittonr/git/mantle)
Finished `dev` profile [unoptimized + debuginfo] target(s) in 17.85s
```

### Retained request preflight with provider inputs

Command:

```sh
CRUNCH_WITNESS_REBUILD_CLI_BIN=/home/brittonr/.cargo-target/debug/mantle CRUNCH_WITNESS_RUST_SOURCE_PROVIDER=/home/brittonr/.cargo-target/repo-targets/mantle/rust-source-provider-musl-host-route-patch-plan-rerun38-2026-06-25/provider-out CRUNCH_WITNESS_TOOLCHAIN_CLOSURE=/home/brittonr/git/mantle/target/source-built-rust-provider-fixed-point-2026-06-25/native-toolchain-closure.json SNIX_BUILD_SANDBOX_SHELL=/bin/sh ./scripts/rebuild-witness-request.sh --check target/release-witness-requests/provider-bound-release-evidence-2026-06-26 --json --scratch-dir target/release-witness-requests/provider-bound-release-evidence-2026-06-26.provider-bound-check.work
```

Output excerpt:

```text
provider-bound inputs supplied; check-only mode will not run provider fixed-point proof
witness request: /home/brittonr/git/mantle/target/release-witness-requests/provider-bound-release-evidence-2026-06-26
provider-bound replay inputs: enabled
check-only mode: this validates prerequisites and request parsing only; it does not produce publishable witness sidecars
{"check_only":true,"kind":"mantle-witness-rebuild","release_id":"provider-bound-release-evidence-2026-06-26",...}
```

### Whitespace and Cairn validation

Commands:

```sh
git diff --check
nix run path:/home/brittonr/git/cairn#cairn -- validate --root .
```

Output:

```text
git diff --check: completed successfully
```

```json
{
  "change_issues": [],
  "changes": 1,
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "spec_issues": [],
  "specs_validated": 7,
  "valid": true
}
```

## Remaining independent rebuild blocker

Current inspected audit:

`target/release-witness-requests/provider-bound-release-evidence-2026-06-26.two-output-fix.work/witness-rebuild-audit/meta.json`

Relevant extracted failure text:

```text
HTTP fetch failed for https://ftpmirror.gnu.org/gmp/gmp-6.2.1.tar.bz2: io: Network is unreachable (os error 101)
```

Until a fresh full helper run completes and produces witness sidecars for both release artifacts, this change proves only the provider-proof candidate mechanism and preflight plumbing, not independent external witness verification for the release.

## Archive notes

`cairn archive witness-provider-bound-replay --execute` completed, but hit the known Cairn date fallback and created `cairn/archive/1970-01-01-witness-provider-bound-replay`. The archive directory was manually renamed to `cairn/archive/2026-06-26-witness-provider-bound-replay`.

After archiving, the accepted `MODIFIED` requirement text was manually synced into `cairn/specs/verification-evidence/spec.md` so the canonical spec includes the provider-proof scenario and the updated incomplete/path-escape scenarios.

Post-archive validation command:

```sh
nix run path:/home/brittonr/git/cairn#cairn -- validate --root .
```

Output:

```json
{
  "change_issues": [],
  "changes": 0,
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "spec_issues": [],
  "specs_validated": 6,
  "valid": true
}
```
