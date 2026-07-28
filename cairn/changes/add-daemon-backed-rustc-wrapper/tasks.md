# Tasks

## Phase 1: Protocol and policy

- [ ] [depends:persist-rust-unit-castore-results] I1 Extend the Rust cache core with bounded wrapper request, response, invocation-manifest, output-contract, bypass, and daemon-disposition schemas. r[rustc_cache_adapter.daemon_boundary]
- [ ] [serial] I2 Add a typed Nickel daemon policy contract with deterministic runtime export for modes, roots, peers, effects, limits, result sources, publication, and redaction. r[rustc_cache_adapter.daemon_boundary.policy]
- [ ] [parallel] I3 Add positive and negative pure tests for canonical protocol identity, malformed messages, oversized fields, unsupported versions, invalid output contracts, and every bypass class. r[rustc_cache_adapter.daemon_boundary.protocol]

## Phase 2: Daemon and wrapper shells

- [ ] [serial] I4 Add `mantle rust-cache serve` with one store-owning daemon, restrictive Unix-socket creation, peer-user verification, bounded concurrency, explicit shutdown, and no ambient credential forwarding. r[rustc_cache_adapter.daemon_boundary]
- [ ] [serial] I5 Add `mantle-rustc-wrapper` as a thin Cargo `RUSTC_WRAPPER` client that preserves the real compiler path, argument order, exit status, stdout, and stderr. r[rustc_cache_adapter.wrapper_parity]
- [ ] [serial] I6 Add fail-open pass-through and fail-closed modes for daemon loss, unsupported invocations, missing manifests, policy rejection, and protocol failure. r[rustc_cache_adapter.wrapper_parity.bypass]
- [ ] [parallel] I7 Add subprocess tests for unauthorized peers, stale sockets, daemon termination, truncated frames, timeout, cancellation, compiler failure, and stream parity. r[rustc_cache_adapter.daemon_boundary.protocol]

## Phase 3: Declared-input eligibility and execution

- [ ] [serial] I8 Add Mantle-generated invocation manifests that bind source, compiler, sysroot, platform, arguments, admitted environment, dependencies, proc macros, build-script outputs, native-link inputs, output contracts, and effect policy. r[rustc_cache_adapter.strong_eligibility]
- [ ] [serial] I9 Verify the manifest BLAKE3 and every current declared input before strong lookup or publication. Bypass or block every missing, changed, out-of-root, or unclassified input. r[rustc_cache_adapter.strong_eligibility]
- [ ] [serial] I10 Execute eligible misses with explicit argv, environment, working directory, readable roots, writable staging, resource limits, and effect policy. r[rustc_cache_adapter.strong_eligibility.enforcement]
- [ ] [parallel] I11 Add negative tests for undeclared include files, changed generated files, proc-macro reads outside admitted roots, native-link drift, sysroot drift, secret environment, network attempts, and unsupported effects. r[rustc_cache_adapter.strong_eligibility.enforcement]

## Phase 4: Result reuse and Cargo output commit

- [ ] [serial] I12 Query the common local Rust unit cache, admit one complete result, restore artifacts into private staging, verify them, and commit Cargo output paths before wrapper success. r[rustc_cache_adapter.atomic_output_commit]
- [ ] [serial] I13 On eligible compiler success, ingest declared artifacts and publish the common Rust unit result only after output and receipt verification. r[rustc_cache_adapter.atomic_output_commit]
- [ ] [serial] I14 Bypass compiler queries, incremental compilation, missing manifests, unsupported response forms, unsupported output shapes, and unsupported effect policy without publishing cache results. r[rustc_cache_adapter.wrapper_parity.bypass]
- [ ] [parallel] I15 Add positive clean-target Cargo tests for local hits and negative tests for partial restore, stale output, artifact conflict, compiler failure, and concurrent distinct outputs. r[rustc_cache_adapter.atomic_output_commit]

## Phase 5: Optional shared cache integration

- [ ] [depends:share-rust-unit-action-results] I16 Add explicit signed remote lookup and publication policy to the daemon without exposing remote credentials to wrapper or compiler processes. r[rustc_cache_adapter.shared_results]
- [ ] [parallel] I17 Add clean-client remote-hit, invalid-signature, incomplete-tree, offline, conflict, and publication-failure tests. r[rustc_cache_adapter.shared_results]

## Phase 6: Boundaries, evidence, and lifecycle

- [ ] [serial] I18 Keep strict self-build, witness, and release proof lanes scrubbed of ambient `RUSTC_WRAPPER` and daemon variables unless a later proof change admits them. r[rustc_cache_adapter.proof_boundary]
- [ ] [parallel] I19 Document explicit setup, declared-input requirements, pass-through classes, security boundaries, local and remote trust, Cargo-owned behavior, no-FUSE materialization, and non-claims. r[rustc_cache_adapter.proof_boundary]
- [ ] [serial] V1 Run focused core, daemon, wrapper, protocol, policy, pass-through, local-hit, and output-commit tests. Record exact output in `cairn/changes/add-daemon-backed-rustc-wrapper/evidence/verification.md`. r[rustc_cache_adapter.wrapper_parity]
- [ ] [serial] V2 Benchmark daemon round-trip, cold miss overhead, local hit restoration, remote hit restoration, and pass-through overhead with named sample and size limits. r[rustc_cache_adapter.performance]
- [ ] [serial] V3 Run `nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- validate --root .`, all three gates for this change, and `tracey coverage`. Record exact output before sync and archive. r[rustc_cache_adapter.proof_boundary]
