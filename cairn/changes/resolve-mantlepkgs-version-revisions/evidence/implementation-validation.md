# Mantlepkgs version-resolution implementation validation

## Result

The implementation passed its focused core, shell, contract, Wasm, and clippy checks.

The real pilot resolved and rechecked two `hello` versions. It emitted two Mantlepkgs v1 manifests, generated both catalogs, and composed their domain shards without Nix.

## Focused checks

The exact transcript is in [`pilot/focused-validation.log`](pilot/focused-validation.log).

The successful commands were:

```text
nix develop -c rustfmt --edition 2024 --check crates/mantlepkgs-core/src/versions.rs src/mantlepkgs_version_cmd.rs src/foreign_graph_compiler.rs src/mantlepkgs_cmd.rs
nix develop -c env CARGO_TARGET_DIR=/tmp/mantlepkgs-version-baseline-target cargo test -p mantlepkgs-core
nix develop -c env CARGO_TARGET_DIR=/tmp/mantlepkgs-version-baseline-target cargo check -p mantlepkgs-core --target wasm32-unknown-unknown
nix develop -c env CARGO_TARGET_DIR=/tmp/mantlepkgs-version-baseline-target cargo test -p mantle --bin mantle mantlepkgs_version_cmd::tests
nix develop -c env CARGO_TARGET_DIR=/tmp/mantlepkgs-version-baseline-target cargo test -p mantle --bin mantle mantlepkgs_cmd::tests
nix develop -c env CARGO_TARGET_DIR=/tmp/mantlepkgs-version-baseline-target cargo test -p mantle --bin mantle foreign_graph_compiler::tests
nix develop -c env CARGO_TARGET_DIR=/tmp/mantlepkgs-version-baseline-target cargo clippy -p mantlepkgs-core -p mantle --bin mantle --no-deps -- -D warnings
PATH=/nonexistent mantle mantlepkgs domain-compose --manifest <pilot-domain-manifest> --sealed-manifest-out <sealed-output> --out <catalog-output>
git diff --check
```

The core suite passed `86` tests. The version command suite passed `15` tests.

The Mantlepkgs command suite passed `33` tests. The foreign graph compiler suite passed `22` tests. The no-std core also compiled for `wasm32-unknown-unknown`.

## Positive evidence

- Exact revision cohorts seal with deterministic BLAKE3 identities.
- Successful, unavailable, and failed observations remain distinct.
- The compact index keeps the newest sampled revision for each exact lookup key.
- Resolution receipts replay against the accepted policy and index.
- Revision groups preserve versioned selectors and one explicit default alias.
- Successful source and version rechecks emit existing Mantlepkgs manifests.
- Generated manifests include their checked policy files.
- Both historical catalogs publish and adapt into deterministic domain shards.
- Distinct versioned selectors compose without Nix in the consumer path.
- Ordinary Nix fixed-output derivations retain native builder and content-hash facts.
- Prefix-map assignments rewrite only exact mapped store paths.
- Symlink text is hashed without link traversal.

## Negative evidence

- Floating revisions, wrong systems, unsafe paths, invalid limits, and unknown methods fail closed.
- Missing versions remain blocked and do not select nearby revisions.
- Stale indexes, resolution sets, groups, plans, and receipts fail validation.
- Selector and alias collisions fail closed.
- Duplicate recheck receipts cannot hide an unchecked selector.
- Source metadata mismatches remain failed observations.
- Special files fail source-tree hashing.
- Failed rechecks do not emit partial manifests.
- Existing output roots reject replacement.
- Invalid cohorts and plans fail before the selected Nix executable runs.
- Oversized process capture fails before unbounded memory use.
- Foreign builtins without fixed-output metadata still fail shape validation.
- Store-path name extensions do not pass the exact prefix-map boundary rule.

## Pilot evidence

See [`pilot/README.md`](pilot/README.md) for the exact producer, identities, revision groups, rechecks, and claim boundary.

The generated manifests passed `mantlepkgs validate`. Both `mantlepkgs generate` runs published complete one-package catalogs.

The two catalogs adapted into separate domain shards. `mantlepkgs domain-compose` then published a five-record public catalog with `PATH=/nonexistent`.

## Cairn and Tracey

The locked Cairn CLI at revision `fb1a7403a7897f7fa161e0b3c5d86b4cf19e52f8` passed strict validation and the proposal, design, and tasks gates.

After spec sync, strict validation passed with `62` accepted specs. Tracey no longer listed any `mantlepkgs_versions.*` ID as missing or dangling.

Repository-wide Tracey coverage still exits nonzero for existing requirements outside this change. The exact global debt list is in `cairn-post-sync-validation.log`.

## Nix package check

`nixfmt --check flake.nix` passed.

`nix build .#checks.x86_64-linux.crunch --no-link -L` compiled the release binary after the Nix source filter admitted the embedded Mantlepkgs contracts and the existing Nario fixture.

The broad check then reached the library tests. It stopped with `173` passes and one unrelated failure in `protected_exec_seccomp::linux::tests::seccomp_listeners_require_distinct_fresh_worker_threads` inside the Nix sandbox.

The exact final output is in `nix-package-check.log`. This evidence does not claim that the broad package check passed.

## Reference audit

See [`nixpkgs-multiverse-reference-audit.md`](nixpkgs-multiverse-reference-audit.md).

The audit records the exact upstream revision, MIT license observation, system, BLAKE3 artifact identities, and retained authority boundary.

## Claim boundary

This evidence proves only the checked version-resolution workflow and the recorded pilot observations.

It does not prove package correctness, compatibility, cache retention, evaluator parity, reproducibility, deployment safety, or release eligibility.
