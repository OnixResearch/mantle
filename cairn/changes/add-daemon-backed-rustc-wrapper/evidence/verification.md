# Verification: daemon-backed Rust compiler cache adapter

Date: 2026-08-03

## Result

The focused implementation and lifecycle checks passed. The adapter now has these surfaces:

- `mantle rust-cache serve`
- `mantle-rust-cache-daemon`
- `mantle-rustc-wrapper`
- `mantle-rustc-manifest`
- typed Nickel daemon policy and failure fixtures

The daemon uses `SO_PEERCRED`, bounded frames, fixed workers, explicit shutdown, BLAKE3-bound manifests, declared-input revalidation, and content-addressed receipts. Production compiler execution uses the exact configured Bubblewrap executable.

Bubblewrap mounts only verified declared inputs as read-only paths. It mounts only the fresh private output stage as writable. Policy roots are admission boundaries and are not mounted in full.

V1 admits one output file. Publication uses Linux `renameat2(RENAME_NOREPLACE)` for atomic no-clobber commit.

## Focused Rust tests

Command:

```text
nix develop path:/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/devshell-daemon-wrapper -c sh -c 'cargo fmt --manifest-path Cargo.toml --all -- --check; CARGO_TARGET_DIR=target/daemon-wrapper-final-v3 cargo test --manifest-path Cargo.toml -p crunch-rust-cache-core -p crunch-rust-cache -p crunch-rustc-wrapper -- --nocapture'
```

Result:

```text
crunch-rust-cache:      25 passed; 0 failed
crunch-rust-cache-core: 25 passed; 0 failed
architecture:            1 passed; 0 failed
crunch-rustc-wrapper:   20 passed; 0 failed
daemon binary:           2 passed; 0 failed
manifest binary:         2 passed; 0 failed
```

The focused tests include positive and negative cases for:

- canonical request, manifest, policy, response, and receipt identities;
- unsupported versions, cross-field mismatches, duplicate roles, and malformed limits;
- compiler queries, incremental builds, response files, missing manifests, and all named bypass classes;
- oversized and truncated frames before unbounded allocation;
- peer UID mismatch;
- stale sockets, active sockets, occupied socket paths, daemon shutdown, and worker drain;
- timeout kill and reap;
- declared file and directory identity, mutation, size bounds, and escaping symlinks;
- action-to-manifest compiler, source, sysroot, and environment binding;
- output rewriting, exact output sets, stale destinations, rollback, and concurrent commit races;
- Linux no-clobber publication with one winner for a shared destination;
- local compile and clean-target local cache restoration;
- signed clean-client shared result restoration;
- invalid signatures, incomplete trees, offline sources, conflicts, and publication failures in the common shared-result tests;
- real Bubblewrap execution with undeclared host-file denial, host-loopback denial, environment clearing, exact status, stdout, stderr, and admitted-stage writes.

Focused atomic tests also passed:

```text
concurrent output tests: 2 passed; 0 failed
stale output test:       1 passed; 0 failed
```

## Root CLI and proof-boundary tests

Commands used focused `cargo test -p mantle --bin mantle` filters.

Results:

```text
rust-cache serve CLI:             2 passed; 0 failed
witness environment scrub:       1 passed; 0 failed
self-build environment scrub:    1 passed; 0 failed
source-provider environment scrub: 1 passed; 0 failed
```

The strict lanes remove ambient `RUSTC_WRAPPER`, `RUSTC_WORKSPACE_WRAPPER`, `CARGO_BUILD_RUSTC_WRAPPER`, `MANTLE_RUST_CACHE_POLICY`, and all wrapper manifest variables before proof execution. The self-build lane then creates only its existing receipt-bound remap wrapper.

Root CLI checks passed:

```text
cargo check -p mantle --bin mantle: PASS
cargo clippy -p mantle --bin mantle --no-deps -- -D warnings: PASS
```

## Policy fixtures

Command:

```text
nickel export rust-cache/daemon/fixtures/policy.ncl
```

Result: PASS.

These fixtures failed as required:

```text
invalid-limit.ncl
invalid-unsafe-path.ncl
invalid-ambient-authority.ncl
```

Summary:

```text
policy-positive-and-negatives-passed
```

## Performance rail

Workload: copy the checked-in `crunch-rust-cache-core/Cargo.toml` fixture through the daemon protocol.

Limits:

- artifact bytes: 389
- concurrency: 1
- timeout: 5,000 ms
- cold-miss samples: 1
- all other samples: 5

Observed debug-build latency:

| Workload | Median | p95 |
|---|---:|---:|
| Cold miss | 39,580 us | 39,580 us |
| Local hit restoration | 26,251 us | 26,658 us |
| Signed shared hit restoration | 31,423 us | 32,016 us |
| Daemon round-trip bypass | 23,326 us | 23,357 us |
| Direct pass-through | 462 us | 716 us |
| Cold-miss overhead over pass-through | 39,118 us | 38,864 us |

These values include daemon startup because the bounded test uses `--once`. They are regression observations, not production throughput claims.

## Quality checks

Commands and results:

```text
cargo fmt --all -- --check: PASS
cargo clippy -p crunch-rust-cache-core -p crunch-rustc-wrapper --all-targets --no-deps -- -D warnings: PASS
cargo check -p crunch-rust-cache-core --target wasm32-wasip2: PASS
cargo build -p crunch-rustc-wrapper --bins: PASS
nixfmt --check flake.nix: PASS
git diff --check: PASS
```

Tiger Style:

```text
crunch-rust-cache-core --lib: PASS
crunch-rustc-wrapper --lib: PASS
```

Dependency compilation still prints the existing vendored `snix-castore` dead-code warning. Focused no-dependency Clippy has no first-party warnings.

## Cairn lifecycle

Commands:

```text
/home/brittonr/git/OnixResearch/cairn/target/debug/cairn validate --root .
/home/brittonr/git/OnixResearch/cairn/target/debug/cairn gate proposal add-daemon-backed-rustc-wrapper --root .
/home/brittonr/git/OnixResearch/cairn/target/debug/cairn gate design add-daemon-backed-rustc-wrapper --root .
/home/brittonr/git/OnixResearch/cairn/target/debug/cairn gate tasks add-daemon-backed-rustc-wrapper --root .
```

Results:

```text
validate: PASS, no findings
proposal: PASS, receipt 66dd518f62518913b91430a93bfbb6681a952206a5574626df290fb62e3fba85
design:   PASS, receipt 027a8b9dda7ace03d366c7447794ff0af27da51ff4435871a50a6dd1a205ac8a
tasks:    PASS, 22 of 22 complete, receipt d653dc3a99f472a90dbd23c7d9998a28dffbf6c07b5daa9fc165d7d076cabb73
```

`tracey coverage --root . --json` ran. It reported 145 of 148 accepted release-provenance requirements referenced. The three missing references are existing unrelated release-provenance requirements:

```text
mantle.release_provenance.content_bound_evidence_manifest
mantle.release_provenance.content_bound_requirement_coverage
mantle.release_provenance.legacy_coverage_boundary
```

The traceability receipt is:

```text
068670c6966065d4ba6a0ba966a545bf655d02cacec4f90a630e809d3ea24c56
```

## Nix blocker

`nix build .#rustc-wrapper -L` reached evaluation of `mantle-rustc-wrapper-0.1.0`, then failed while fetching the existing private dependency:

```text
error: Failed to fetch git repository 'https://git.onix.computer/z3tAR4For7qw8ZirkJzoDw1VNDDLM.git'
```

This is the existing private-input blocker. Cargo binary builds passed in the detached repository dev shell.

## Claim boundary

Wrapper receipts prove bounded observations for one declared invocation and selected policy. They do not prove compiler correctness, source trust, Bubblewrap correctness, full Cargo compatibility, universal reproducibility, release eligibility, deployment safety, or global cache availability.
