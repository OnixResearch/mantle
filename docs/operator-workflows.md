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

# Package release evidence from the current tracked worktree
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

For the current trust boundary behind those claims, see
[`docs/bootstrap-stage0-inventory.md`](bootstrap-stage0-inventory.md).
For the checked-in benchmark workflow, see
[`docs/benchmark-suite.md`](benchmark-suite.md).
