# Verification: structured Mantlepkgs domains

## Result

The focused implementation checks pass. The change provides typed domain contracts, deterministic composition, explicit variants, separate validation roots, and durable external-corpus evidence.

## Baseline

Before the main implementation, the focused Mantlepkgs core and CLI tests passed. Cairn repository validation and the proposal, design, and tasks gates also passed.

The initial chained broad check stopped before its machine-contract step because `wasm32-unknown-unknown` was not installed. The final validation used the repository target, `wasm32-wasip2`, and ran the machine-contract checker separately.

## Focused Rust checks

The final focused core command was:

```console
cargo test -p mantlepkgs-core
```

Result: 33 passed, 0 failed.

The suite includes positive composition and validation cases. It also includes negative cases for metadata, limits, selector conflicts, stale identities, missing bases, cycles, blocked packages, missing validation packages, stale validation policy, malformed outcomes, output limits, timeouts, corpus provenance, unsupported package selections, and missing corpus sources.

The final focused CLI command was:

```console
cargo test --bin mantle mantlepkgs_cmd::tests
```

Result: 16 passed, 0 failed. The tests include typed Nickel fixtures, duplicate shard rejection, corpus tamper rejection, atomic failure behavior, and a child process with no Nix command in `PATH`.

## Quality checks

These checks pass:

- `cargo fmt --all -- --check`;
- focused `cargo clippy -p mantlepkgs-core --all-targets --no-deps -- -D warnings`;
- focused `cargo clippy --bin mantle --no-deps -- -D warnings`;
- `./scripts/check-first-party-tigerstyle.sh -p mantlepkgs-core`;
- `git diff --check`;
- `cargo check -p mantlepkgs-core --target wasm32-wasip2`.

The Tiger Style check used the repository-pinned Dylint package. Its cached upstream reference produced a non-fatal Git transport warning.

## No-Nix consumer checks

The current debug binary ran with an empty tool directory as `PATH`. It performed both commands without a Nix frontend:

```console
mantle --json mantlepkgs corpus-verify \
  --evidence mantlepkgs/corepkgs-corpus/evidence.json \
  --artifact-root mantlepkgs/corepkgs-corpus/evidence \
  --sealed-evidence-out target/corepkgs-corpus-sealed-final.json

mantle --json mantlepkgs domain-compose \
  --manifest mantlepkgs/corepkgs-corpus/domain.ncl \
  --sealed-manifest-out target/corepkgs-domain-sealed-final.json \
  --out target/corepkgs-domain-catalog-final.json
```

The regenerated files match the checked-in durable artifacts byte for byte.

The composed domain catalog identity is:

`1cd3e9c53f55ed5eafe98df9776b9128c0f99f0e29b0a5b6bb7c27d36a971822`

The validation-root identity is:

`d22a5037c074f4ae144f40be192ec954afed15e7f3380961b7785834593ca01c`

## External corpus

The producer evaluated Ekala `corepkgs` at revision:

`a9a1af8abbf08b972dbce7bb9c2643c7d76d140d`

The selected packages are `corepkgs-libpng` and `corepkgs-zlib`. Both package rows are buildable and have no blockers.

The producer catalog identity is:

`0e417992593aa5e5b9aa85bf87e57db88399770d2377bbc43a7c47c6be59849b`

The sealed corpus evidence identity is:

`74676b71adacc4a2a3150935199e27ae9e05ee20223fe9106950c108df9a6768`

The durable evidence includes these bound artifacts:

| Role | BLAKE3 | Bytes |
|---|---|---:|
| blockers | `c656af77ac86ca0d02b6d0daf96bc8041e10fab501dc7bf632bc6cd4b9207101` | 322 |
| catalog | `47f4d6fffbc4937e63b570f38cfd71e49b0bf2e2480d84b86a6b43d6dbf6effc` | 3437 |
| graph | `54ef6a4809fc4151fc000d5be77ab653958837332d507d635f873abd17a3d964` | 1028237 |
| producer | `da055133ae016423baca182d93da940bb03172df1f8388a8465199dedc02276d` | 2811 |
| sources | `dc0cc334cf763ab2262f93c813e032557bd039e9116aa588868340937f1fb28b` | 41766 |

## Machine-contract check

The Mantlepkgs JSON family is classified in `schemas/machine-contracts/inventory.ncl`. The checker reports no Mantlepkgs finding.

The repository-wide checker still fails on 40 existing root JSON producers that have no inventory family decision. These findings are outside this change. They include unrelated StageX, source-binding, and remote-credential sources.

## Broader check limits

The current flake cannot fetch the private input at:

`https://git.onix.computer/z3tAR4For7qw8ZirkJzoDw1VNDDLM.git`

A detached worktree at commit `401251b8` supplied the development shell. All Rust commands compiled the current change source, not the detached source.

Tracey reports 145 of 148 requirements referenced. The three missing IDs are unrelated release-provenance requirements:

- `mantle.release_provenance.content_bound_evidence_manifest`;
- `mantle.release_provenance.content_bound_requirement_coverage`;
- `mantle.release_provenance.legacy_coverage_boundary`.

The Tracey receipt is `068670c6966065d4ba6a0ba966a545bf655d02cacec4f90a630e809d3ea24c56`.

The repository also retains known broad baseline failures in the generated Nickel standard-library list, bootstrap blocker inventory, dependency-inclusive Clippy, and full first-party Tiger Style. This change does not modify those unrelated owners.

## Cairn lifecycle

Repository validation returned `valid: true` before and after archive.

The proposal, design, and completed tasks gates returned `PASS`.

Gate receipt hashes:

- proposal: `c7584a2402f0ddd83cfd2f23755eb7aeee94955028ffdebbc2fb3e57415ee314`;
- design: `f6cb3f6fdc1dc6e5fbedbc315dead1e33ebb911f7d75e1cd3bfdccfe1c3e675a`;
- completed tasks: `85f6130ba0e85960509316552f2b0b11f7502fcaf2bce350d6f72ff894d2e7a1`.

The accepted specification was created at `cairn/specs/mantlepkgs-catalog-structure/spec.md`.

The executed sync receipt is `20f1f896e5ac88cbc39edd9b11f8d9344da0d06ab7def784742c4cc0cdef8d3a`.

The executed archive receipt is `895c01dc2aa177f29c512878abee0bb9b3b80fe1e50aa5ec8c770d9270d0ee08`.

## Claim boundary

This evidence applies only to the recorded source revision, selected packages, policies, artifacts, systems, validation root, and receipts.

It does not prove package correctness, broad ecosystem coverage, evaluator parity, reproducibility, or release eligibility.
