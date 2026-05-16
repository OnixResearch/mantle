# Operator workflows

This page complements the top-level README.

Current operator-facing command families:

- `mantle doctor`
- `mantle build` / `mantle build --plan`
- `mantle shell` / `mantle develop`
- `mantle run`
- `mantle attest`
- `mantle release`

Use `mantle --help` for the full command list. Use this page for the common
operator loops.

## Validation tiers

Use the checked-in toolchain from `rust-toolchain.toml`.

### Ordinary first-party gate

Run the checked-in wrapper from the repo root:

```bash
./scripts/check-first-party-quality.sh
```

It runs three ordinary edit-time checks in order:

```bash
cargo fmt --check \
  -p mantle \
  -p crunch-attestation \
  -p crunch-build \
  -p crunch-delta \
  -p crunch-eval \
  -p crunch-glue \
  -p crunch-pipeline \
  -p crunch-project \
  -p crunch-shell \
  -p crunch-store
./scripts/check-first-party-clippy.sh
cargo test --workspace --lib --tests
```

Notes:

- The root `-p mantle` rustfmt leg covers the root package's `src/`,
  `examples/`, and `tests/`, including `tests/benchmark_harness.rs`.
- `./scripts/check-first-party-clippy.sh` excludes vendored workspace members
  `fuse-backend-rs`, `nix-compat`, `nix-compat-derive`, `snix-build`,
  `snix-castore`, `snix-store`, and `snix-tracing` so first-party warnings
  fail cleanly.
- The wrappers assume the documented build environment. If `clang`, `mold`,
  `pkg-config`, or the OpenSSL pkg-config path are missing, fix the shell env
  first instead of treating that as a code failure.

### Tigerstyle lane

Run the repo-pinned Tiger Style consumer check when you want the structural lint
pass:

```bash
./scripts/check-first-party-tigerstyle.sh
```

That wrapper delegates to this flake entry point:

```bash
nix run .#tigerstyle -- check
```

Notes:

- default package scope comes from `[workspace.metadata.tigerstyle]` in
  `Cargo.toml`
- workspace-specific lint rollout config lives in `dylint.toml`
- vendored workspace members stay out of the default Tiger Style scope

### Heavyweight rails

Keep these checks separate from the ordinary gate:

```bash
cargo test -p crunch-pipeline --test integration_build \
  pipeline_determinism_probe_ -- --ignored --nocapture
./scripts/prove-self-hosting.sh --check
./scripts/check-release-determinism-quality.sh
./scripts/check-release-nix-witness-quality.sh
nix build .#checks.x86_64-linux.release-determinism-quality --no-link -L
nix build .#checks.x86_64-linux.release-nix-witness-quality --no-link -L
```

- The release determinism quality rail runs the generated deterministic proof
  smoke and validates its receipt/log BLAKE3 in one checked-in entry point.
- The `release-determinism-quality` flake check is the Nix/CI-callable heavy
  gate for the same underlying generated proof regression.
- The `release-nix-witness-quality` package/check is the Nix/CI-callable heavy
  gate for the bounded `mantle release nix-witness` CLI path; keep it opt-in
  rather than part of ordinary developer builds.
- The determinism probe is an ignored integration rail, not part of every edit.
- `./scripts/prove-self-hosting.sh --check` is only self-hosting preflight.
- The full ignored proof remains a separate heavier run:

```bash
./scripts/prove-self-hosting.sh
```

### Vendored maintenance lane

Workspace-wide vendored upkeep stays outside the first-party gate. Do not treat
vendored clippy debt as an ordinary edit blocker for tracked mantle code.

If you bypass the checked-in wrappers and run direct compile-heavy `cargo`
commands, keep `TMPDIR` and `CARGO_TARGET_DIR` on disk-backed scratch and use
the PATH / `PKG_CONFIG_PATH` / `SNIX_BUILD_SANDBOX_SHELL` prerequisites called
out in the self-hosting workflow.

## Plan before building

Start with the no-mutate preflight:

```bash
# Default build host checks
mantle doctor

# Extra nightly-toolchain checks for self-build hosts
mantle doctor --profile self-build
```

Then preview what mantle would do per root:

```bash
mantle build --plan .#hello
mantle --json build --plan .#hello
```

Plan output uses four action labels:

- `cached` - all outputs are already accepted locally
- `substitute` - cache metadata says a remote hit is available
- `build` - no accepted hit exists, but local build preflight passed
- `preflight-error` - mantle cannot take the local build path yet

When you want degraded hermetic behavior to fail instead of warn, opt into
strict mode on the build-entry commands that expose it:

```bash
mantle build --strict-hermetic .#hello
mantle self-build --strict-hermetic --store /tmp/mantle-store -j 4 --no-substitute
```

Structured build reports surface the same operator facts in stable fields:

- `hermeticity_mode`
- `hermeticity_audit_events[]`
- `effect_policy_version` (`mantle-build-effects-v1` for deterministic proof receipts)
- `declared_effects[]` and `observed_effects[]`; deterministic release verification
  fails closed when observed effects are missing or exceed the declared set
- `failed[]`
- `outcomes[].outputs[].artifact_attestation`

That last block gives both the logical store path and the persisted sidecar
path for the artifact attestation.

## Enter a dev shell or run a package

`mantle shell` resolves a `devShells` target from the compatibility-named
`crunch.ncl` package root, builds it, then reads `$out/.crunch-shell.json` to
construct the runtime environment.
`mantle develop` is the deprecated alias for the same implementation.

```bash
# Run one command inside the resolved shell
mantle shell --command env

# Run a script through $SHELL -c inside the shell environment
mantle shell --run 'echo "$CRUNCH_SHELL"'

# Add extra PATH entries from another output or directory
mantle shell --with /path/to/extra-tools --command my-tool

# Skip or enforce shell hooks
mantle shell --no-hook --command true
mantle shell --strict-hooks --command true

# Deprecated alias
mantle develop
```

Notes:

- The built shell output must contain `.crunch-shell.json`. If it does not,
  mantle fails clearly and points back at `mkShell`.
- `CRUNCH_SHELL` is always set to the built shell output path.
- `--command` and `--run` are mutually exclusive.

`mantle run` is the package-side entry point. It builds a package target from
the compatibility-named `crunch.ncl` package root and executes the first program
under `bin/`.

```bash
# Run the default package from the compatibility-named crunch.ncl package root
mantle run

# Run a named package
mantle run .#hello

# Pass arguments through after --
mantle run .#hello -- --help
```

If the selected package output does not contain `bin/`, `mantle run` fails
instead of guessing.

## Inspect and verify attestations

Successful builds, accepted cache hits, and remote substitutions persist native
artifact attestations under the mantle state directory. Use `mantle attest` to
inspect those sidecars and the closure or project views derived from them.

```bash
# Show one artifact attestation
mantle attest show /nix/store/<hash>-hello

# Assemble one runtime closure attestation from rooted outputs
mantle attest closure /nix/store/<hash>-hello

# Verify persisted sidecars against canonical reconstruction
mantle attest verify artifact /nix/store/<hash>-hello
mantle attest verify closure /nix/store/<hash>-hello

# Compare two native attestation documents or artifact selectors
mantle attest diff /nix/store/<hash>-a /nix/store/<hash>-b

# Render or verify a project attestation
mantle attest project /nix/store/<hash>-root
mantle attest verify project --file saved-project-envelope.json /nix/store/<hash>-root
mantle attest verify project --digest <canonical-hex> /nix/store/<hash>-root
```

Selector rules:

- `show` and `verify artifact` accept a logical store path, an exported store
  path, or a raw store-path string.
- `closure` and `verify closure` accept one or more rooted outputs.
- `project` and `verify project` need at least one selected built root.
- `diff` accepts either saved attestation JSON or store-path selectors.

## Package and verify release evidence

Release evidence starts from a full proof run, not from
`./scripts/prove-self-hosting.sh --check`.

```bash
# Produce a full proof bundle first
./scripts/prove-self-hosting.sh

# Package release evidence from tracked worktree files, witness workflow driver,
# stage0 inventory, self-hosting test files, and verified vendored Cargo inputs
mantle release create \
  --release-id mantle-<version> \
  --binary target/self-hosting-proof/run-.../binaries/stage2-mantle \
  --proof-bundle target/self-hosting-proof/run-...

# Rebuild published artifacts into an isolated output area and compare bytes
mantle release reproduce target/release-evidence/<release-id> \
  --rebuild-output-dir target/release-rebuild/<release-id> \
  --rebuild-command ./scripts/rebuild-release-artifacts.sh

# Smoke the generated deterministic proof path and save a rail receipt/log
cargo -Zscript scripts/release-determinism-smoke.rs
cargo -Zscript scripts/check-release-determinism-smoke-receipt.rs \
  target/release-determinism-smoke/latest/receipt.json

# Run the real release-specific proof rail: full proof bundle -> release bundle
# -> two clean deterministic proof rebuilds -> verified deterministic receipt
./scripts/prove-real-release-determinism.sh

# Reuse an existing full self-hosting proof bundle instead of rerunning it
./scripts/prove-real-release-determinism.sh \
  --proof-bundle target/self-hosting-proof/run-...

# Re-check a saved real deterministic proof rail output
cargo -Zscript scripts/check-real-release-determinism-receipt.rs \
  target/release-evidence/<release-id>

# Write portable JSON/Markdown summary artifacts for archival/review
cargo -Zscript scripts/summarize-real-release-determinism.rs \
  target/release-evidence/<release-id>

# Inspect bootstrap parity's checked self-build proof descriptor consumption
mantle --json bootstrap parity-report

# Re-check a saved bundle using only bundle-local contents
mantle release verify target/release-evidence/<release-id>

# Require a verified bit-for-bit reproducibility report
mantle release verify target/release-evidence/<release-id> --require-reproducible
```

Repeat `--binary` when one release bundle should carry multiple executables.
The checked-in proof bundle keeps durable copies of stage1 and stage2 under
`binaries/`, so the packaged release binary can be the proven stage2 output
rather than a scratch-store path that disappears when the proof exits.

`mantle release verify` proves bundle-local integrity and proof-context by
itself. It checks that required bundled artifacts exist, that the manifest stays
canonical, that recorded digests still match, and that the nested proof bundle
is a full proof artifact. Reproducibility is reported separately as `absent`,
`matched`, or `mismatched`. The bit-for-bit reproducible release label requires
a verified canonical reproducibility report whose artifact set matches the
published release artifact set; ordinary bundle-local integrity never implies
that label. This still does not prove a full-source bootstrap root or
independent rebuild agreement. `bootstrap parity-report` also consumes the
checked-in compact descriptor at
`bootstrap/evidence/real-self-build-proof-parity.json` for the
`crunch.self-build` row; that descriptor surfaces the bounded proof digest and
provider kind, but the row remains partial and still blocks Guix/StageX parity
until the separate source-root/lineage blockers are closed.

## Sign and verify decentralized release material

Once a release bundle verifies locally, sign it into a verification directory:

```bash
mantle release attest target/release-evidence/<release-id>
```

That writes `target/release-verification/<release-id>/release-attestation.json`
and a matching `.sig` sidecar by default.

Before witness publication, export the trusted public key, scaffold the
verifier-local social policy files, and export a portable witness-request
directory:

```bash
RELEASE_TRUSTED_KEY=$(mantle attest key-show --signing-key /path/to/release.key)
RELEASE_SIGNER_NAME="${RELEASE_TRUSTED_KEY%%:*}"

mantle attest policy-init target/release-verification/<release-id> \
  --profile single-witness \
  --trusted-release-signer "$RELEASE_SIGNER_NAME" \
  --trusted-witness-identity <witness-identity>

mantle release witness-export target/release-evidence/<release-id> \
  --verification-dir target/release-verification/<release-id> \
  --request-dir target/release-witness-requests/<release-id>
```

Use `--profile self-proof-only` when the verification directory should stay at a
self-proof-only policy with `min_matching_witnesses = 0`. Use
`--profile single-witness` when one matching witness should be enough to satisfy
policy. `mantle attest key-show` prints the exact `name:base64` verifier token
accepted by `--trusted-public-key`; omit `--signing-key` to read the default
configured signing key instead. The signer name for `--trusted-release-signer`
is the token prefix before the colon, so a shell workflow can derive it with
`RELEASE_SIGNER_NAME="${RELEASE_TRUSTED_KEY%%:*}"` when the release key was
auto-generated in a per-operator `CRUNCH_CONFIG_DIR`. `mantle attest
policy-init` writes `policy.json` plus an explicit empty `revocations.json`,
and it refuses to overwrite either file unless `--force` is present. `mantle
release witness-export` copies only public verification material into the
request directory: the verified release-evidence bundle, the signed release
attestation, and request metadata. It does not copy signing keys or
verifier-local policy files.

Independent rebuilders can then replay the checked-in witness rebuild rail in a
second environment:

```bash
./scripts/rebuild-witness-request.sh \
  target/release-witness-requests/<release-id> \
  --identity <witness-identity> \
  --system x86_64-linux \
  --toolchain rust-1.91.1 \
  --host-class nixos-25.05 \
  --signing-key /path/to/witness.key
WITNESS_TRUSTED_KEY=$(mantle attest key-show --signing-key /path/to/witness.key)
```

The wrapper is the operator-facing shell. It resolves `bwrap`, picks an
absolute static `SNIX_BUILD_SANDBOX_SHELL`, derives a controlled scratch root
(`target/release-witness-requests/<release-id>.work/` by default or
`$CRUNCH_WITNESS_SCRATCH_DIR`), rewrites `TMPDIR` and `CARGO_TARGET_DIR` under
that root, and then calls the machine-readable core command `mantle release
witness-rebuild <request-dir> ...`. The replay preserves the release bundle's
recorded proof mode, including `--non-nix-host` for non-Nix-host proofs. Use
`--check` when you want prerequisite and request-validation preflight only;
successful `--check` output is not rebuild proof. The request directory stays
immutable after validation. Witness sidecars
and the rebuild audit directory land under the scratch verification output,
not back inside the exported request tree.

Inspect the discovered material with:

```bash
mantle attest release-show target/release-verification/<release-id>
mantle attest witness-show target/release-verification/<release-id>
```

Then import the returned witness sidecars and verify technical status plus
social policy against trusted key material:

```bash
mantle attest witness-import target/release-verification/<release-id> \
  target/release-witness-requests/<release-id>.work/release-verification/<release-id>
mantle attest release-verify target/release-verification/<release-id> \
  --trusted-public-key "$RELEASE_TRUSTED_KEY" \
  --trusted-public-key "$WITNESS_TRUSTED_KEY"
```

The scratch verification directory also carries `witness-rebuild-audit/meta.json`
plus captured workflow stdout/stderr so the witness can hand back both the
signed sidecars and a replay transcript. `mantle attest witness-import` fails
closed on missing signatures, release-digest mismatches, and conflicting
existing witness identities before copying anything into the publisher
verification directory.

`mantle attest release-verify --json` reports bundle-local technical validity,
social policy sufficiency, and independent rebuild agreement as separate
fields. The independent agreement fields are `independent_agreement_status`,
`independent_agreement_class`, `independent_agreement_report_digest`,
`independent_agreement_counted_witness_count`,
`independent_agreement_skipped_witness_count`,
`independent_agreement_failed_witness_count`, and per-witness classification
reasons under `independent_agreement_witnesses`. Agreement is derived from the
accepted witness files, the verifier-local `policy.json`, `revocations.json`,
and trusted keys passed on the command line. It is not a hand-authored claim.
Unknown-key or bad-signature witnesses remain visible in
`discovered_witness_count` and in the independent-agreement witness list, but
are skipped before quorum evaluation. Signature-valid witnesses with wrong
release references or rebuilt binary digests are failed evidence, not counted
agreement. If a canonical agreement report is present at
`target/release-verification/<release-id>/agreement-report.json`, verification
checks it byte-for-byte against the derived report; duplicate
`agreement-report.json` attachments, including nested
`independent-agreement/agreement-report.json`, are rejected as ambiguous.
Release-evidence bundles may carry the same optional report artifact at
`independent-agreement/agreement-report.json`, where normal manifest digest
verification applies.

A successful single-witness run proves external witness agreement only under
the configured policy; a satisfied independent-agreement status gives the
stronger verifier-local `independent-rebuild-agreement` class for the accepted
witness set. It still does not prove a full-source bootstrap root or globally
reproducible release outputs.

For the current trust boundary behind those claims, see
[`docs/bootstrap-stage0-inventory.md`](bootstrap-stage0-inventory.md).
For the checked-in benchmark workflow, see
[`docs/benchmark-suite.md`](benchmark-suite.md).
