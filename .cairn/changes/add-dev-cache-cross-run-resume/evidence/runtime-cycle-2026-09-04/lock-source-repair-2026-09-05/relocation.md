# Captured Git sibling repair

## Observed blocker

The release-planner replay from `5833ede3` retained both Artifact Core revisions with separate source paths and content digests. The original five `missing-captured-git-source-fact` blockers disappeared.

The next package blocker was:

```text
missing-path-dependency-manifest: dependency `bounded-tree-core` manifest .../vendor-deps/bounded-tree-cap-0.1.0/../bounded-tree-core/Cargo.toml is not readable
```

The captured Git facts already contained `vendor-deps/bounded-tree-core-0.1.0/Cargo.toml`. Both packages had the same full Git source identity at revision `b0fd0103bc9eed2c1b6d852045959462d105d8f1`. The relative path no longer matched Cargo's versioned vendor layout.

## Repair

`src/rust_plan/git_paths.rs` resolves relative normal and build dependencies from captured package facts within the parent's exact Git source. The resolver keeps the full resolved source identity and checks the package name and version constraint. It rejects missing, ambiguous, cross-revision, absolute-path, and malformed-version candidates.

The resolver performs no I/O. Ordinary path-workspace dependencies keep their existing path behavior. The source pins and vendor files remain unchanged.

## Evidence

Both initial relocation regression tests failed before this repair. The positive test now selects the versioned sibling manifest. The negative tests reject another Git revision, an absolute path, a malformed version, an empty version, and an incompatible version.

The after-change runs reported:

```text
test result: ok. 238 passed; 0 failed; 0 ignored; 0 measured; 2422 filtered out; finished in 5.38s
test result: ok. 60 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 14.80s
machine schema contract check: PASS (34 contracted, 68 classified)
```

Clippy and formatting also passed. The final planner run includes the added ambiguity regression:

```text
test result: ok. 239 passed; 0 failed; 0 ignored; 0 measured; 2422 filtered out; finished in 6.12s
```

The updated Nix checks passed: `tigerstyle`, `rust-plan-source-admission`, and `rust-plan-child-action-preflight`. The Nix log records:

```text
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 57 filtered out; finished in 0.01s
test result: ok. 239 passed; 0 failed; 0 ignored; 0 measured; 2422 filtered out; finished in 0.78s
```

The final source-snapshot replay remains pending at this checkpoint.

## Preserved operator failures

The first debug replay reached its 300-second limit without a receipt. The first release transfer encountered a cross-filesystem reflink error. Its partial output lacked executable permission, so the first release launch exited 126 before the planner ran. A full byte copy and explicit permission preservation repaired the transfer.

An earlier validation linker disappeared from its Nix store path. The committed CLI rerun failed at the linker, not source admission. The full CLI suite then passed with the installed rustup toolchain's unwrapped LLD directory. No production code changed for that environment repair.

The old Pueue task records were absent during the later log-export attempt. The four empty wrapper logs are not execution evidence. Receipt status files and stderr captures remain available. `pueue-capture-status.txt` records this limit.

These observations do not establish a fixed point, provider correctness, or release eligibility. The failed bootstrap remains unchanged.
