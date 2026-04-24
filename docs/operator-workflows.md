# Operator workflows

This page complements the top-level README.

Current operator-facing command families:

- `crunch doctor`
- `crunch build` / `crunch build --plan`
- `crunch shell` / `crunch develop`
- `crunch run`
- `crunch attest`
- `crunch release`

Use `crunch --help` for the full command list. Use this page for the common
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
  -p crunch \
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

- The root `-p crunch` rustfmt leg covers the root package's `src/`,
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
```

- The determinism probe is an ignored integration rail, not part of every edit.
- `./scripts/prove-self-hosting.sh --check` is only self-hosting preflight.
- The full ignored proof remains a separate heavier run:

```bash
./scripts/prove-self-hosting.sh
```

### Vendored maintenance lane

Workspace-wide vendored upkeep stays outside the first-party gate. Do not treat
vendored clippy debt as an ordinary edit blocker for tracked crunch code.

If you bypass the checked-in wrappers and run direct compile-heavy `cargo`
commands, keep `TMPDIR` and `CARGO_TARGET_DIR` on disk-backed scratch and use
the PATH / `PKG_CONFIG_PATH` / `SNIX_BUILD_SANDBOX_SHELL` prerequisites called
out in the self-hosting workflow.

## Plan before building

Start with the no-mutate preflight:

```bash
# Default build host checks
crunch doctor

# Extra nightly-toolchain checks for self-build hosts
crunch doctor --profile self-build
```

Then preview what crunch would do per root:

```bash
crunch build --plan .#hello
crunch --json build --plan .#hello
```

Plan output uses four action labels:

- `cached` - all outputs are already accepted locally
- `substitute` - cache metadata says a remote hit is available
- `build` - no accepted hit exists, but local build preflight passed
- `preflight-error` - crunch cannot take the local build path yet

When you want degraded hermetic behavior to fail instead of warn, opt into
strict mode on the build-entry commands that expose it:

```bash
crunch build --strict-hermetic .#hello
crunch self-build --strict-hermetic --store /tmp/crunch-store -j 4 --no-substitute
```

Structured build reports surface the same operator facts in stable fields:

- `hermeticity_mode`
- `hermeticity_audit_events[]`
- `failed[]`
- `outcomes[].outputs[].artifact_attestation`

That last block gives both the logical store path and the persisted sidecar
path for the artifact attestation.

## Enter a dev shell or run a package

`crunch shell` resolves a `devShells` target from `crunch.ncl`, builds it, then
reads `$out/.crunch-shell.json` to construct the runtime environment.
`crunch develop` is the deprecated alias for the same implementation.

```bash
# Run one command inside the resolved shell
crunch shell --command env

# Run a script through $SHELL -c inside the shell environment
crunch shell --run 'echo "$CRUNCH_SHELL"'

# Add extra PATH entries from another output or directory
crunch shell --with /path/to/extra-tools --command my-tool

# Skip or enforce shell hooks
crunch shell --no-hook --command true
crunch shell --strict-hooks --command true

# Deprecated alias
crunch develop
```

Notes:

- The built shell output must contain `.crunch-shell.json`. If it does not,
  crunch fails clearly and points back at `mkShell`.
- `CRUNCH_SHELL` is always set to the built shell output path.
- `--command` and `--run` are mutually exclusive.

`crunch run` is the package-side entry point. It builds a package target from
`crunch.ncl` and executes the first program under `bin/`.

```bash
# Run the default package from crunch.ncl
crunch run

# Run a named package
crunch run .#hello

# Pass arguments through after --
crunch run .#hello -- --help
```

If the selected package output does not contain `bin/`, `crunch run` fails
instead of guessing.

## Inspect and verify attestations

Successful builds, accepted cache hits, and remote substitutions persist native
artifact attestations under the crunch state directory. Use `crunch attest` to
inspect those sidecars and the closure or project views derived from them.

```bash
# Show one artifact attestation
crunch attest show /nix/store/<hash>-hello

# Assemble one runtime closure attestation from rooted outputs
crunch attest closure /nix/store/<hash>-hello

# Verify persisted sidecars against canonical reconstruction
crunch attest verify artifact /nix/store/<hash>-hello
crunch attest verify closure /nix/store/<hash>-hello

# Compare two native attestation documents or artifact selectors
crunch attest diff /nix/store/<hash>-a /nix/store/<hash>-b

# Render or verify a project attestation
crunch attest project /nix/store/<hash>-root
crunch attest verify project --file saved-project-envelope.json /nix/store/<hash>-root
crunch attest verify project --digest <canonical-hex> /nix/store/<hash>-root
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

# Package release evidence from tracked worktree files plus verified vendored Cargo inputs
crunch release create \
  --release-id crunch-<version> \
  --binary /path/to/crunch \
  --proof-bundle target/self-hosting-proof/run-...

# Re-check a saved bundle using only bundle-local contents
crunch release verify target/release-evidence/<release-id>
```

Repeat `--binary` when one release bundle should carry multiple executables.

`crunch release verify` proves bundle-local integrity and proof-context only.
It checks that required bundled artifacts exist, that the manifest stays
canonical, that recorded digests still match, and that the nested proof bundle
is a full proof artifact. It does not prove a full-source bootstrap root,
independent rebuild agreement, or globally reproducible release outputs.

## Sign and verify decentralized release material

Once a release bundle verifies locally, sign it into a verification directory:

```bash
crunch release attest target/release-evidence/<release-id>
```

That writes `target/release-verification/<release-id>/release-attestation.json`
and a matching `.sig` sidecar by default.

Before witness publication, export the trusted public key, scaffold the
verifier-local social policy files, and export a portable witness-request
directory:

```bash
RELEASE_TRUSTED_KEY=$(crunch attest key-show --signing-key /path/to/release.key)
RELEASE_SIGNER_NAME="${RELEASE_TRUSTED_KEY%%:*}"

crunch attest policy-init target/release-verification/<release-id> \
  --profile single-witness \
  --trusted-release-signer "$RELEASE_SIGNER_NAME" \
  --trusted-witness-identity <witness-identity>

crunch release witness-export target/release-evidence/<release-id> \
  --verification-dir target/release-verification/<release-id> \
  --request-dir target/release-witness-requests/<release-id>
```

Use `--profile self-proof-only` when the verification directory should stay at a
self-proof-only policy with `min_matching_witnesses = 0`. Use
`--profile single-witness` when one matching witness should be enough to satisfy
policy. `crunch attest key-show` prints the exact `name:base64` verifier token
accepted by `--trusted-public-key`; omit `--signing-key` to read the default
configured signing key instead. The signer name for `--trusted-release-signer`
is the token prefix before the colon, so a shell workflow can derive it with
`RELEASE_SIGNER_NAME="${RELEASE_TRUSTED_KEY%%:*}"` when the release key was
auto-generated in a per-operator `CRUNCH_CONFIG_DIR`. `crunch attest
policy-init` writes `policy.json` plus an explicit empty `revocations.json`,
and it refuses to overwrite either file unless `--force` is present. `crunch
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
WITNESS_TRUSTED_KEY=$(crunch attest key-show --signing-key /path/to/witness.key)
```

The wrapper is the operator-facing shell. It resolves `bwrap`, picks an
absolute static `SNIX_BUILD_SANDBOX_SHELL`, derives a controlled scratch root
(`target/release-witness-requests/<release-id>.work/` by default or
`$CRUNCH_WITNESS_SCRATCH_DIR`), rewrites `TMPDIR` and `CARGO_TARGET_DIR` under
that root, and then calls the machine-readable core command `crunch release
witness-rebuild <request-dir> ...`. Use `--check` when you want prerequisite and
request-validation preflight only; successful `--check` output is not rebuild
proof. The request directory stays immutable after validation. Witness sidecars
and the rebuild audit directory land under the scratch verification output,
not back inside the exported request tree.

Inspect the discovered material with:

```bash
crunch attest release-show target/release-verification/<release-id>
crunch attest witness-show target/release-verification/<release-id>
```

Then import the returned witness sidecars and verify technical status plus
social policy against trusted key material:

```bash
crunch attest witness-import target/release-verification/<release-id> \
  target/release-witness-requests/<release-id>.work/release-verification/<release-id>
crunch attest release-verify target/release-verification/<release-id> \
  --trusted-public-key "$RELEASE_TRUSTED_KEY" \
  --trusted-public-key "$WITNESS_TRUSTED_KEY"
```

The scratch verification directory also carries `witness-rebuild-audit/meta.json`
plus captured workflow stdout/stderr so the witness can hand back both the
signed sidecars and a replay transcript. `crunch attest witness-import` fails
closed on missing signatures, release-digest mismatches, and conflicting
existing witness identities before copying anything into the publisher
verification directory. `crunch attest release-verify` reports the technical
class, policy status, and final class separately. Unknown-key or bad-signature
witnesses remain visible in `discovered_witness_count` but are excluded before
quorum evaluation, so mixed witness sets do not abort verification. A
successful single-witness run proves external witness agreement under the
configured policy. It still does not prove a full-source bootstrap root or
globally reproducible release outputs.

For the current trust boundary behind those claims, see
[`docs/bootstrap-stage0-inventory.md`](bootstrap-stage0-inventory.md).
For the checked-in benchmark workflow, see
[`docs/benchmark-suite.md`](benchmark-suite.md).
