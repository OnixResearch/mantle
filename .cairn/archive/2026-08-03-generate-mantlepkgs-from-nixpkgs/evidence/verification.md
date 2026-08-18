# Mantlepkgs verification

## Live acceptance rail

All consumer commands used these controls:

- `PATH=/usr/bin:/bin`
- `CRUNCH_NO_FUSE=1`
- explicit bubblewrap executable
- `--offline`
- no substitution option
- one absolute physical store root
- logical store prefix `/mantle/store`

The accepted catalog was `fe5ef37d81a8492ddad4232cfc942e80b70fcfa75a2b45816e2eb96f4855d1d8`.

The following live rebuilds passed:

| Package | Result | Receipt BLAKE3 |
|---|---|---|
| `hello` | complete, realized, 951 units | `3bdfd6c06d511f6b695b7f0ba1a86c3e73526c8862d9c33942c566cd3c6f35a7` |
| `zlib` | complete, realized, 760 units | `f7bf3c68238b34ecb76c1bf50b4a6ec9f0678d3ac2186ca22b001b79d7c45fb1` |
| `libpng` | complete, realized, 951 units | `4bf949f61e39fafd5d75d82acc158a59809da873f469ebca09c02f706bb8d27a` |

The verification command also accepted the catalog with Nix absent from `PATH`. It reported three packages and six bound artifacts.

Planning passed for all selected roots with Nix absent from `PATH`.

The live manifest passed validation. The floating-source fixture failed as required and reported `floating-source`.

## Focused checks

These commands passed after the final code changes:

```text
cargo fmt --all -- --check
cargo test -p mantlepkgs-core
cargo test -p snix-build
cargo test -p crunch-build
cargo test -p mantle --bin mantle mantlepkgs_cmd::tests
cargo test -p mantle --bin mantle foreign_import_cmd::tests
cargo test -p mantle --bin mantle foreign_graph_compiler::tests
cargo clippy -p crunch-build --all-targets --no-deps -- -D warnings
cargo clippy -p mantle --bin mantle --no-deps -- -D warnings
./scripts/check-first-party-tigerstyle.sh -p mantlepkgs-core -- --all-targets
git diff --check
```

The final focused counts include:

- foreign graph compiler: 18 passed;
- foreign derivation importer: 4 passed;
- structured attributes: 4 passed;
- `crunch-build`: 690 passed;
- all selected focused commands: zero failures.

Positive and negative tests cover catalog logic, source admission, publication, protocol payloads, sandbox capabilities, and Nix-free subprocess behavior.

## Broader baseline checks

`cargo test --workspace` reached an unrelated existing failure:

```text
stdlib::tests::embedded_stdlib_matches_repo_lib_directory
79 passed; 1 failed
```

`lib/artifact-auth-cutover-receipt.ncl` exists in the repository. It is absent from `crates/crunch-eval/src/stdlib.rs::STDLIB_FILES`.

`nix flake check -L` reached the existing `bootstrap-blocker-inventory` failure:

```text
115 findings across 3 classes
355 evidence-backed suppressions
0 promotion claims
```

The check requires zero findings and zero promotion claims.

Dependency-inclusive Clippy remains blocked by existing vendored `fuse-backend-rs` findings. The no-dependency checks for the changed first-party surfaces passed.

The full first-party Tiger Style rail has existing findings outside this change. The focused `mantlepkgs-core` rail passed.

## Host constraints

Host FUSE returns a descriptor I/O error. The acceptance rail used the supported no-FUSE path and mounted complete PathInfo input closures.

The shared Cargo target was active in another Mantle run. Final builds and tests used a private target at `target/mantlepkgs-cargo` to avoid binary replacement races.

The first `/tmp` retry reached its dataset quota. Final realization used one root-filesystem scratch and store location with sufficient space.
