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

## Follow-up: provider target propagation (2026-06-27)

A fresh provider-bound helper replay showed that the generated helper wrapper was invoking `self-build --cargo-free --fixed-point` without the provider proof target. The successful packaged provider proof used `targets: ["x86_64-unknown-linux-musl"]`; the failed replay receipt had `targets: []` and stopped at the stable `unsupported-compiler-guard` blocker for `aws-lc-sys@0.39.1`.

The helper now exports `CRUNCH_WITNESS_PROVIDER_TARGET`, defaults it to `x86_64-unknown-linux-musl`, rejects an explicitly empty value, and passes `--target "$CRUNCH_WITNESS_PROVIDER_TARGET"` to the generated provider-bound driver.

### Validation: helper syntax

Command:

```sh
bash -n scripts/rebuild-witness-request.sh
```

Output:

```text
Task 63: completed successfully
```

### Validation: focused binary witness-rebuild tests

Command:

```sh
PATH=/home/brittonr/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin:/home/brittonr/.cargo/bin:/nix/store/97vplpbajnr7x03fqh9biz5v6960sv22-clang-wrapper-21.1.8/bin:/nix/store/1sw8whfl5gfblp6r9qdkiw1b4j9fgwar-mold-2.40.4/bin:/nix/store/rvp7qlpf5jqvdckjy1afjb6aha6j8dxg-pkg-config-wrapper-0.29.2/bin:$PATH PKG_CONFIG_PATH=/nix/store/1l5jgzy26hkjz1y3apn1051asvn42sfn-openssl-3.6.1-dev/lib/pkgconfig SNIX_BUILD_SANDBOX_SHELL=/bin/sh cargo test -p mantle --bin mantle witness_rebuild -- --nocapture
```

Output excerpt:

```text
test witness_rebuild::tests::collect_rebuilt_output_paths_uses_provider_fixed_point_candidate_by_digest ... ok

test result: ok. 20 passed; 0 failed; 0 ignored; 0 measured; 859 filtered out; finished in 0.02s
```

### Validation: helper and witness-rebuild CLI tests

Command:

```sh
PATH=/home/brittonr/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin:/home/brittonr/.cargo/bin:/nix/store/97vplpbajnr7x03fqh9biz5v6960sv22-clang-wrapper-21.1.8/bin:/nix/store/1sw8whfl5gfblp6r9qdkiw1b4j9fgwar-mold-2.40.4/bin:/nix/store/rvp7qlpf5jqvdckjy1afjb6aha6j8dxg-pkg-config-wrapper-0.29.2/bin:$PATH PKG_CONFIG_PATH=/nix/store/1l5jgzy26hkjz1y3apn1051asvn42sfn-openssl-3.6.1-dev/lib/pkgconfig SNIX_BUILD_SANDBOX_SHELL=/bin/sh cargo test -p mantle --test release_cli witness_rebuild -- --nocapture
```

Output excerpt:

```text
test witness_rebuild_helper_rejects_empty_provider_target ... ok
test witness_rebuild_helper_provider_bound_driver_passes_default_target ... ok

test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 89 filtered out; finished in 0.36s
```

### Validation: provider fixed-point release fixtures

Command:

```sh
PATH=/home/brittonr/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin:/home/brittonr/.cargo/bin:/nix/store/97vplpbajnr7x03fqh9biz5v6960sv22-clang-wrapper-21.1.8/bin:/nix/store/1sw8whfl5gfblp6r9qdkiw1b4j9fgwar-mold-2.40.4/bin:/nix/store/rvp7qlpf5jqvdckjy1afjb6aha6j8dxg-pkg-config-wrapper-0.29.2/bin:$PATH PKG_CONFIG_PATH=/nix/store/1l5jgzy26hkjz1y3apn1051asvn42sfn-openssl-3.6.1-dev/lib/pkgconfig SNIX_BUILD_SANDBOX_SHELL=/bin/sh cargo test -p mantle --test release_cli provider_fixed_point -- --nocapture
```

Output excerpt:

```text
test release_create_can_package_provider_fixed_point_proof ... ok
test release_verify_provider_fixed_point_external_override_reports_source ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 98 filtered out; finished in 0.05s
```

### Validation: binary build

Command:

```sh
PATH=/home/brittonr/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin:/home/brittonr/.cargo/bin:/nix/store/97vplpbajnr7x03fqh9biz5v6960sv22-clang-wrapper-21.1.8/bin:/nix/store/1sw8whfl5gfblp6r9qdkiw1b4j9fgwar-mold-2.40.4/bin:/nix/store/rvp7qlpf5jqvdckjy1afjb6aha6j8dxg-pkg-config-wrapper-0.29.2/bin:$PATH PKG_CONFIG_PATH=/nix/store/1l5jgzy26hkjz1y3apn1051asvn42sfn-openssl-3.6.1-dev/lib/pkgconfig SNIX_BUILD_SANDBOX_SHELL=/bin/sh cargo build -p mantle --bin mantle
```

Output excerpt:

```text
Compiling mantle v0.1.0 (/home/brittonr/git/mantle)
Finished `dev` profile [unoptimized + debuginfo] target(s) in 35.83s
```

### Validation: whitespace

Command:

```sh
git diff --check
```

Output:

```text
completed successfully
```

### Fresh retained-request replay after target propagation

Command:

```sh
REQUEST=target/release-witness-requests/provider-bound-release-evidence-2026-06-26
SCRATCH=target/release-witness-requests/provider-bound-release-evidence-2026-06-26.provider-bound-target-rerun.work
CONFIG=target/release-signing/provider-bound-independent-witness-config
PROVIDER=/home/brittonr/.cargo-target/repo-targets/mantle/rust-source-provider-musl-host-route-patch-plan-rerun38-2026-06-25/provider-out
CLOSURE=/home/brittonr/git/mantle/target/source-built-rust-provider-fixed-point-2026-06-25/native-toolchain-closure.json
RUSTC=/home/brittonr/.cargo-target/repo-targets/mantle/rust-source-provider-musl-host-route-patch-plan-rerun38-2026-06-25/provider-out/bin/rustc
CLI=/home/brittonr/.cargo-target/debug/mantle
CRUNCH_CONFIG_DIR="$CONFIG" \
CRUNCH_WITNESS_REBUILD_CLI_BIN="$CLI" \
CRUNCH_WITNESS_RUST_SOURCE_PROVIDER="$PROVIDER" \
CRUNCH_WITNESS_TOOLCHAIN_CLOSURE="$CLOSURE" \
CRUNCH_WITNESS_RUSTC="$RUSTC" \
./scripts/rebuild-witness-request.sh \
  --scratch-dir "$SCRATCH" \
  "$REQUEST" \
  --identity provider-bound-independent \
  --system x86_64-linux \
  --toolchain source-root-rust-provider-musl \
  --host-class britton-desktop-local
```

Output/audit excerpts:

```text
provider-bound witness driver: /home/brittonr/git/mantle/target/release-witness-requests/provider-bound-release-evidence-2026-06-26.provider-bound-target-rerun.work/tmp/provider-bound-witness-driver.sh
provider-bound replay inputs: enabled
```

```text
provider-fixed-point-proof/stage1/status.txt: 0
provider-fixed-point-proof/stage2/status.txt: 0
provider-fixed-point-proof/meta.json: "status": "success"
provider-fixed-point-proof/meta.json: "target_triple": "x86_64-unknown-linux-musl"
provider-fixed-point-proof/meta.json: "binary_blake3": "465f8fdccaa7594c0f0578ef39b16cb58d21bafab92deb51cb2861aa612bd561"
```

The target propagation fix therefore gets the witness-produced provider proof past the prior `aws-lc-sys` compiler-guard blocker.

The full helper replay still did **not** produce publishable witness sidecars. It failed later in the self-hosting proof while fetching bootstrap inputs:

```text
store error: build: HTTP fetch failed for https://ftpmirror.gnu.org/binutils/binutils-2.42.tar.gz: io: Network is unreachable (os error 101)
FAILED root: bwrap
  message: dependency binutils.drv failed
```

This replay also exposed the next provider-candidate reproducibility issue once the network blocker is cleared: the witness-produced provider binary digest from the scratch-root source tree was `465f8fdccaa7594c0f0578ef39b16cb58d21bafab92deb51cb2861aa612bd561`, while the published provider artifact `binaries/01-mantle` in the release manifest is `b4fdeca80db000a4e003513417b665ae2a1014f6752ed9ff65bb3b6b57ce48f3`. Do not claim independent witness verification until a full helper replay both completes the self-hosting proof and produces provider/self-hosting proof artifacts whose digests match the signed release outputs.

### Follow-up: source material mismatch

After the target fix was pushed, the retained request was inspected to explain the provider digest mismatch. The copied release source archive in the retained request does not contain the provider-candidate witness rebuild implementation, while the current pushed tree does.

Command:

```sh
SRC=target/release-witness-requests/provider-bound-release-evidence-2026-06-26.provider-bound-target-rerun.work/source-tree
for file in src/main.rs src/witness_rebuild.rs scripts/rebuild-witness-request.sh; do
  echo "== $file"
  if cmp -s "$SRC/$file" "$file"; then echo identical; else echo different; fi
  wc -c "$SRC/$file" "$file"
done
```

Output excerpt:

```text
== src/main.rs
identical
146408 .../source-tree/src/main.rs
146408 src/main.rs
== src/witness_rebuild.rs
different
 40481 .../source-tree/src/witness_rebuild.rs
 73630 src/witness_rebuild.rs
== scripts/rebuild-witness-request.sh
different
wc: .../source-tree/scripts/rebuild-witness-request.sh: No such file or directory
14063 scripts/rebuild-witness-request.sh
```

Additional grep evidence:

```text
.../source-tree/src/witness_rebuild.rs:674: "rebuilt output count mismatch: supported workflow produces {SUPPORTED_WORKFLOW_OUTPUT_COUNT} output, request expects {expected_count_u32}"
src/witness_rebuild.rs:700: collect_provider_fixed_point_candidates(&plan.scratch_layout.provider_fixed_point_proof_dir, &mut candidates)?;
src/witness_rebuild.rs:730: fn collect_provider_fixed_point_candidates(
```

Conclusion: the retained `provider-bound-release-evidence-2026-06-26` request is no longer a viable independent-witness target for current-code provider-bound verification. A valid independent witness attempt needs a freshly packaged release bundle whose source archive, provider fixed-point proof, provider binary, self-hosting proof, and signed release attestation are all produced from the same pushed source snapshot.

### Follow-up: current pushed-tree provider fixed-point proof

After committing the retained-request source-mismatch evidence at `d40d134b`, a fresh provider fixed-point proof was run from the current pushed tree. This is **not** release verification and does **not** prove independent witness verification; it only establishes a current-source provider candidate for a future freshly packaged release bundle.

Command:

```sh
OUT=/tmp/mantle-provider-fixed-point-current-d40d134b
PROVIDER=/home/brittonr/.cargo-target/repo-targets/mantle/rust-source-provider-musl-host-route-patch-plan-rerun38-2026-06-25/provider-out
CLOSURE=/home/brittonr/git/mantle/target/source-built-rust-provider-fixed-point-2026-06-25/native-toolchain-closure.json
RUSTC=/home/brittonr/.cargo-target/repo-targets/mantle/rust-source-provider-musl-host-route-patch-plan-rerun38-2026-06-25/provider-out/bin/rustc
rm -rf "$OUT"
/home/brittonr/.cargo-target/debug/mantle self-build \
  --cargo-free \
  --fixed-point \
  --out "$OUT" \
  --rust-source-provider "$PROVIDER" \
  --toolchain-closure "$CLOSURE" \
  --rustc "$RUSTC" \
  --target x86_64-unknown-linux-musl
```

Output:

```text
Cargo-free fixed-point: success
bundle: /tmp/mantle-provider-fixed-point-current-d40d134b
stage1_binary_blake3: 115a3de848759f6ef8f4bcbd157af65e7f9af77298c26366d35107c7a55c244f
stage2_binary_blake3: 115a3de848759f6ef8f4bcbd157af65e7f9af77298c26366d35107c7a55c244f
```

Metadata/status excerpts:

```text
meta.json: "status": "success"
meta.json: "fixed_point": true
meta.json: "root": "/home/brittonr/git/mantle"
meta.json: "target_triple": "x86_64-unknown-linux-musl"
stage1/status.txt: 0
stage2/status.txt: 0
```

The current-source provider digest is therefore `115a3de848759f6ef8f4bcbd157af65e7f9af77298c26366d35107c7a55c244f`. It intentionally does not repair the retained release request, whose signed provider artifact is still `b4fdeca80db000a4e003513417b665ae2a1014f6752ed9ff65bb3b6b57ce48f3`; a new release evidence bundle must be created from this current source lineage before another independent witness attempt can succeed.

### Follow-up: current pushed-tree self-hosting proof blocker

The current-tree self-hosting proof preflight passed, but full helper replay failed in stage0 while fetching GCC prerequisites from `ftpmirror.gnu.org`.

Preflight command:

```sh
CRUNCH_PROOF_SCRATCH_DIR=/tmp/mantle-self-hosting-current-58ed7d78-work \
  ./scripts/prove-self-hosting.sh --check \
  --bundle-dir /tmp/mantle-self-hosting-current-58ed7d78
```

Preflight output excerpt:

```text
pkg-config: /nix/store/0yz5jd6ywpq414g4k9g3ysbima3g3fa7-pkg-config-wrapper-0.29.2/bin/pkg-config
bwrap: /nix/store/gr9l6ql3wg70idpqlqhnfdx81hak22c8-bubblewrap-0.11.0/bin/bwrap
SNIX_BUILD_SANDBOX_SHELL: /nix/store/4mdqc2snfiihr6r61ln1rqs4fis6br9b-busybox-static-x86_64-unknown-linux-musl-1.36.1/bin/busybox
proof mode: fixed-point
selected provider kind: legacy-fetch
proof scratch free: 47488 MiB
proof command: /home/brittonr/.rustup/toolchains/nightly-2025-11-01-x86_64-unknown-linux-gnu/bin/cargo test -p mantle --test self_hosting -- --ignored --nocapture
```

Full helper attempts:

```text
CRUNCH_PROOF_SCRATCH_DIR=/tmp/mantle-self-hosting-current-58ed7d78-rerun1-work ./scripts/prove-self-hosting.sh --bundle-dir /tmp/mantle-self-hosting-current-58ed7d78-rerun1
# FAILED after 65.32s: message: dependency gcc.drv failed

CRUNCH_PROOF_SCRATCH_DIR=/tmp/mantle-self-hosting-current-58ed7d78-rerun2-work ./scripts/prove-self-hosting.sh --bundle-dir /tmp/mantle-self-hosting-current-58ed7d78-rerun2
# FAILED after 49.08s: message: dependency gcc.drv failed
```

A durable direct stage0 repro with the same helper-built `crunch` binary and a normal PATH succeeded, showing the self-build logic is not deterministically broken:

```text
[4/4] Verifying output...
verifying /tmp/mantle-stage0-repro-helper-crunch-58ed7d78/store/07w0gjlnswwfx09s68x1c9ljxf61wvfr-mantle/bin/mantle...
  binary OK
self-build-proof: hermeticity-mode=practical
self-build-proof: staged-source=/tmp/mantle-stage0-repro-helper-crunch-58ed7d78/store/5ymisagfa62zzwlxc3v0flhmrx93bhn2-mantle-src
self-build-proof: bwrap-source=mantle-built:/tmp/mantle-stage0-repro-helper-crunch-58ed7d78/store/pys6ig7mb98pf53s7cq35iafz2wll35f-bwrap/bin
self-build-proof: output-binary=/tmp/mantle-stage0-repro-helper-crunch-58ed7d78/store/07w0gjlnswwfx09s68x1c9ljxf61wvfr-mantle/bin/mantle
```

A durable repro using a helper-style symlink PATH failed in the same way as the full helper. Failure excerpts from `/tmp/mantle-stage0-repro-helper-path-58ed7d78/stage0.stderr.txt`:

```text
transient fetch failed; retrying attempt=1 max_attempts=5 error=HTTP fetch failed for https://ftpmirror.gnu.org/gmp/gmp-6.2.1.tar.bz2: http status: 502
transient fetch failed; retrying attempt=1 max_attempts=5 error=HTTP fetch failed for https://ftpmirror.gnu.org/mpc/mpc-1.2.1.tar.gz: http status: 502
transient fetch failed; retrying attempt=1 max_attempts=5 error=HTTP fetch failed for https://ftpmirror.gnu.org/gcc/gcc-13.3.0/gcc-13.3.0.tar.gz: http status: 502
transient fetch failed; retrying attempt=1 max_attempts=5 error=HTTP fetch failed for https://ftpmirror.gnu.org/mpfr/mpfr-4.1.0.tar.bz2: http status: 502
...
sandbox build failed drv=/nix/store/phlhbyjgiwda18006mqyz8dgl2wil2mh-gmp-src.drv err=store error: build: HTTP fetch failed for https://ftpmirror.gnu.org/gmp/gmp-6.2.1.tar.bz2: http status: 502
FAILED root: bwrap
  phase: build
  error_class: builder
  saved_log_path: /tmp/mantle-stage0-repro-helper-path-58ed7d78/state/logs/lr1nd84b23a9kk0qs97m1m7vglinfmc2-bwrap.drv.log
  message: dependency gcc.drv failed
```

This leaves current-code release repackaging blocked on completing a full self-hosting proof. The next attempt should either run when `ftpmirror.gnu.org` is healthy or add a deliberate cached/bootstrap-source path for the GCC prerequisite tarballs before packaging a fresh current-source release bundle.

### Follow-up: switch legacy self-build GNU source URLs to kernel.org mirror

To remove the observed `ftpmirror.gnu.org` dependency from the legacy self-build bootstrap slice, the first-wave GNU source URLs in `bootstrap/gcc.ncl`, `bootstrap/binutils.ncl`, `bootstrap/dash.ncl`, and `bootstrap/make.ncl` were switched to `https://mirrors.kernel.org/gnu/...` while preserving the existing fixed-output hashes. This mirrors prior checked-in precedent in `bootstrap/make-tcc.ncl` for avoiding `ftpmirror.gnu.org` 502s.

Validation: helper-style symlink-PATH stage0 self-build succeeded after the URL changes.

Command shape:

```sh
ROOT=/tmp/mantle-stage0-repro-kernel-mirror-58ed7d78
PATHFARM=/tmp/mantle-proof-path-repro-kernel-mirror-58ed7d78
# populate PATHFARM with symlinks for the proof helper's tool PATH
PATH="$PATHFARM" \
SNIX_BUILD_SANDBOX_SHELL=/nix/store/4mdqc2snfiihr6r61ln1rqs4fis6br9b-busybox-static-x86_64-unknown-linux-musl-1.36.1/bin/busybox \
/tmp/mantle-self-hosting-current-58ed7d78-rerun1-work/cargo-target/debug/crunch \
  --verbose --log-level info \
  --store "$ROOT/store" \
  --state-dir "$ROOT/state" \
  --nix-compat \
  self-build --no-substitute -j 4
```

Fetch/proof output excerpts:

```text
fetching tarball url=https://mirrors.kernel.org/gnu/binutils/binutils-2.42.tar.gz
fetching tarball url=https://mirrors.kernel.org/gnu/mpfr/mpfr-4.1.0.tar.bz2
fetching tarball url=https://mirrors.kernel.org/gnu/mpc/mpc-1.2.1.tar.gz
fetching tarball url=https://mirrors.kernel.org/gnu/gmp/gmp-6.2.1.tar.bz2
fetching tarball url=https://mirrors.kernel.org/gnu/gcc/gcc-13.3.0/gcc-13.3.0.tar.gz
[4/4] Verifying output...
  binary OK
self-build-proof: hermeticity-mode=practical
self-build-proof: staged-source=/tmp/mantle-stage0-repro-kernel-mirror-58ed7d78/store/zvj2lhkcs2bkhiybkqzzlrrgwkhfyqb3-mantle-src
self-build-proof: bwrap-source=mantle-built:/tmp/mantle-stage0-repro-kernel-mirror-58ed7d78/store/pys6ig7mb98pf53s7cq35iafz2wll35f-bwrap/bin
self-build-proof: output-binary=/tmp/mantle-stage0-repro-kernel-mirror-58ed7d78/store/07w0gjlnswwfx09s68x1c9ljxf61wvfr-mantle/bin/mantle
```

Validation: focused `bootstrap/make.ncl` build exercised the patched `bootstrap/make.ncl` URL.

```text
fetching tarball url=https://mirrors.kernel.org/gnu/make/make-4.4.1.tar.gz
build succeeded drv=make-src.drv outputs=["/tmp/mantle-build-make-kernel-mirror/store/sadgqs1ykvs2m6zs35qn8q4qg5s38rr3-make-src"]
build succeeded drv=gnumake.drv outputs=["/tmp/mantle-build-make-kernel-mirror/store/096b231pbs9cqw67p32sdk011lpm60xl-gnumake"]
```

This validation only proves stage0 under the helper-style PATH. A full current-source self-hosting proof still needs to be rerun from the pushed URL-change commit before packaging a fresh release bundle or making any independent witness claim.
