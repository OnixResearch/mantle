# Verification: Mantlepkgs package-impact reports

## Result

The focused implementation checks pass. The change provides deterministic `mantle-package-impact-v1` reports over explicit base and head snapshots.

The report separates catalog changes, build observations, and closure facts. It does not infer a build result from a derivation path, CAS presence, or an absent observation.

## Baseline

Before the core implementation, 33 `mantlepkgs-core` tests passed. Eight focused `crunch-build` action-result tests also passed.

The host has no `cargo` executable. The current flake cannot fetch its private Git input. A detached worktree at commit `401251b8` supplied the development shell. Each Cargo command compiled the current worktree source.

## Focused Rust checks

The final Mantlepkgs core command was:

```console
cargo test -p mantlepkgs-core
```

Result: 54 passed, 0 failed. Of these tests, 21 are package-impact tests.

The impact suite includes positive cases for catalog classification, admitted build transitions, complete closure deltas, retained dependencies, canonical ordering, and stable identities. It includes negative cases for stale bindings, invented success, incompatible snapshots, missing PathInfo facts, missing logical sizes, unresolved references, duplicate members, arithmetic overflow, edge limits, report-byte limits, and semantic mutations.

The focused CLI command was:

```console
cargo test --bin mantle mantlepkgs_cmd::tests::impact
```

Result: 3 passed, 0 failed. The tests cover atomic local publication, rejection before publication, and current action-result admission.

The final action-result regression command was:

```console
cargo test -p crunch-build action_result
```

Result: 8 passed, 0 failed. These tests cover request, policy, platform, signature, output-set, PathInfo, CAS, producer, and publication admission boundaries.

## Quality checks

These focused checks pass:

- `cargo fmt --all`;
- `cargo clippy -p mantlepkgs-core --all-targets --no-deps -- -D warnings`;
- `cargo clippy --bin mantle --no-deps -- -D warnings`;
- repository-pinned Tiger Style for `mantlepkgs-core`;
- `cargo check -p mantlepkgs-core --target wasm32-wasip2`;
- `git diff --check`.

## Contract and fixture checks

Pueue task `7658` exported all positive policy, snapshot, incompatibility, and external-adapter Nickel fixtures.

Pueue task `7660` confirmed that invalid identity-domain, BLAKE3 digest, and limit fixtures fail their contracts.

The handwritten contracts allow zero byte deltas and nullable variant, observation, PathInfo, logical-size, and non-comparable closure fields. The external adapter exports deterministic forge-neutral status and summary data. It requests no credentials or network effect.

## Local command behavior

Pueue task `7653` ran the current debug binary with `PATH=/no-nix`. The command wrote one atomic report without a Nix frontend.

The report identity is:

`ad30c0b4c7681f56ca911726f45d9757342414125ee6c2bc315591a61765650d`

The policy identity is:

`c331dbb596956fdece535a04d0bf3cd8ec8a6c466b8f1058ac3d6edc2fa41af1`

The base snapshot identity is:

`eb015f09299e9537017b831ae63649b7c1cb5b0a054a9ee730192deaa95ea7a7`

The head snapshot identity is:

`63c726738f8eaec706d2359395e4293853dfb8c5e6e4dfeb4807dd73dbd402fe`

Pueue task `7661` rejected incompatible system, store-prefix, and conversion-policy snapshots. None of those cases created a report file.

Action-result observations require one or more explicit `--action-result-report` inputs. The shell computes each report identity and requires the current discovery schema, reused disposition, selected action and result, an admitted candidate, no conflict, and a non-empty trust basis.

## Closure evidence boundary

A comparable closure requires equal system, store prefix, closure semantics, and byte semantics. Both sides must be complete and must include valid PathInfo identities, logical sizes, in-prefix store paths, unique member identities, and resolved references.

A non-comparable closure has stable reason codes. It has no member list, member-count delta, retained-dependency list, or byte delta.

## Machine-contract check

Pueue task `7665` regenerated the Nickel contract and freshness marker. It then validated:

- the JSON Schema;
- positive and negative machine fixtures;
- the generated Nickel contract;
- invariant pointer coverage;
- Rust owner shape;
- freshness against the final producer source.

The repository-wide checker in pueue task `7666` reports no package-impact finding. It still fails on existing unclassified root JSON producers, including unrelated StageX sources.

## Traceability

Before specification sync, the broad Cairn profile found 298 referenced requirements out of 730 accepted requirements.

After sync, it found 305 referenced requirements out of 737. Both counts increased by seven. This matches the seven requirements in this change. None of the `mantlepkgs_impact` requirement IDs appear in the dangling set.

The post-sync Tracey receipt is:

`e05fe0b6ff3399b04b2fb32db7908d76cd93e8ff23598be02380484e15171922`

The broad profile still fails because 432 unrelated accepted requirements lack references. This is a repository baseline outside this change.

## Broader check limits

The current flake fails while fetching:

`https://git.onix.computer/z3tAR4For7qw8ZirkJzoDw1VNDDLM.git`

The repository also retains known broad baseline failures in the generated Nickel standard-library list, bootstrap blocker inventory, dependency-inclusive Clippy, full first-party Tiger Style, and host FUSE. This change does not modify those owners.

## Claim boundary

This evidence applies only to the recorded snapshots, policies, local admission reports, closure facts, source revision, and generated receipts.

It does not prove package correctness, unchanged runtime behavior, reproducibility, deployment safety, release eligibility, or forge authority.
