# Verification: Mantlepkgs source-update plans

## Result

The focused source-update implementation passes its required checks.

The accepted replay produced these identities:

- policy: `fe90ef525d34a644f5fe907f9abcf1672844d0903b9664b3d739601d0aedf5e7`;
- source observation: `b1353b4346a116ee5094dc7982875af0be786f1f112df3f93618bb56010ed50a`;
- OSV observation: `d0fbf70e0d1daed6d6d75c758bb820abb7d8b3535bf47c9dcac9b4db3fa3a650`;
- Repology observation: `079b2812fb6eb3e6e5278bf207482adeb27786f4d1c033f11c2b8262da8ba996`;
- update plan: `36241e442120c022470acadf4e0f7b7db6d2093a9bffcf184efb77d230984ae1`;
- offline publication receipt: `e7e1c46dc063251ed1199a95e768ea6a9ed3b21cc5597be8ad66b843fc43b78c`.

## Baseline

Before implementation, the existing checks passed:

- 54 `mantlepkgs-core` tests;
- 19 focused Mantlepkgs CLI tests;
- 6 `crunch-project` refresh tests.

## Rust tests

The final focused tests passed:

- 73 `mantlepkgs-core` tests;
- 33 `mantlepkgs_cmd::tests` tests;
- 6 `crunch-project` refresh tests;
- the exact stale-preimage denial test after the final denial-receipt change.

The tests cover positive and negative behavior. Negative tests include malformed and oversized responses, duplicate and ambiguous candidates, duplicate advisory findings, unavailable evidence, stale identities, stale preimages, unsafe paths, symlinks, digest changes, path overlap, partial-stage cleanup, and publication conflicts.

Evidence logs:

- `target/update-final2-core-tests.log`;
- `target/update-final2-cli-tests.log`;
- `target/update-release-project-refresh.log`;
- `target/update-release-stale-denial-test.log`.

## Quality checks

These checks passed:

- `cargo fmt --all -- --check`;
- strict no-dependency Clippy for `mantlepkgs-core` and the `mantle` binary;
- focused Tiger Style for the `mantlepkgs-core` library;
- `wasm32-wasip2` checking for `mantlepkgs-core`;
- `git diff --check`.

The broad binary Tiger Style check found only existing findings in `protected_exec.rs` and `protected_exec_seccomp.rs`. It found no issue in `src/mantlepkgs_cmd.rs`.

## Contracts and fixtures

The focused `mantlepkgs.update-plan` machine-contract check passed. It checked:

- JSON Schema validity;
- positive and negative fixtures;
- generated Nickel contract freshness;
- Rust owner shape;
- BLAKE3 freshness bindings.

Seven positive update Nickel fixtures exported successfully. Four negative policy fixtures failed as required.

The broad machine-contract check remains blocked by existing unclassified root JSON producers. The list includes existing `src/stagex_*.rs` files and other unrelated source files. See `target/update-release-machine-contracts-broad.log`.

## Offline replay and publication

Saved observation replay ran with `PATH=/no-nix`.

The run recorded source, OSV, and Repology observations. It then wrote the accepted plan and published one immutable output tree. It did not use Nix or the network.

The shell left the source lock unchanged. It wrote the changed lock and execution receipt only in the new output tree.

The configured live URL is an adapter endpoint. The endpoint must return the selected Mantle response envelope. Raw upstream responses with different schemas fail explicitly.

## Known external blockers

`nix flake check -L` reached flake package evaluation and then failed on the existing private Git input:

`https://git.onix.computer/z3tAR4For7qw8ZirkJzoDw1VNDDLM.git`

See `target/update-release-flake-check.log`.

After sync, traceability increased to 313 referenced requirements out of 745. Its receipt is `847cea94b923e7104a0afd7bfdc48bc9d44b71f8b8b37a8b441d6b0909ed74c9`. No `mantlepkgs_updates` requirement remains dangling.

The full workspace and broad repository rails retain these unrelated blockers:

- the existing `crunch-eval` standard-library inventory failure;
- 432 unrelated Tracey requirements without references;
- vendored dependency Clippy findings;
- unrelated first-party Tiger Style findings;
- existing machine-contract inventory gaps;
- host FUSE support failures.

No focused source-update check depends on these blockers.

## Claim boundary

This evidence proves only the recorded policy, observations, candidate decisions, structured effects, validation links, and local publication result.

It does not prove source trust, advisory completeness, absence of unknown vulnerabilities, package correctness, reproducibility, deployment safety, or release eligibility.
