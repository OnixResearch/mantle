# 2026-11-08: authored source programs, transport, and the Cargo-free plan boundary

This records source-authoring and source-bundle repairs reached by the unchanged
authenticated preparation inputs. Every statement below comes from captured
output in this directory. Preparation still has not succeeded, no provider ran,
and no fixed point is accepted: I3–I5 and V1–V3 remain unchecked.

## Bounded command streams

Protected source leaves are admitted only when they declare the actual bounded
`stdout` and `stderr` products `validate_source_recipe` requires. Two authors
published files without them:

- `source_built_rust_programs_llvm::exec` now declares both streams centrally at
  a 64 MiB ceiling, so the original project extraction leaf and every other LLVM
  command carry the same explicit bound (`COMMAND_LOG_BYTES_MAX`).
- `minicargo::package_fields` now declares both streams at the same 64 MiB
  ceiling. This one author serves both the minicargo and successor package-field
  leaves.

`2026-11-06-llvm-stream-checks.log` and `2026-11-06-metadata-stream-checks.log`
retain the focused runs (27 recipe tests, leaf rustfmt, strict release Clippy and
the release CLI build). Their dependent preparations are
`2026-11-06-llvm-stream-preparation.log` and
`2026-11-06-metadata-stream-preparation.log`. Each passed the boundary fixed by
its own check and rejected the next undeclared stream:

```json
{"error":"source recipe: command llvm-1.90.0.original-project must publish actual bounded stdout","code":1,"kind":"build"}
{"error":"source recipe: command minicargo.package-metadata.0 must publish actual bounded stdout","code":1,"kind":"build"}
```

## Authored action obligations

After both stream repairs, preparation rejected the whole authored Rust-provider
program:

```json
{"error":"source recipe: unreachable or omitted command topology: <1042 authored steps>","code":1,"kind":"build"}
```

The 1042 omitted actions were the crate-directory selects, package worlds,
controller protocol leaves, generated selects and the per-stage LLVM tool trees.
They are authored execution obligations, so the root set must name every
produced output, exactly as the StageX transition author and the native-provider
author already do. `source_built_rust_programs` now derives `roots` from every
step output after the `exit-status` obligation is added; exports and semantic
roots are unchanged.

`2026-11-06-rust-roots-checks.log` retains the focused run: 27 recipe tests, 39
passing and 2 ignored Rust-program tests, leaf rustfmt, strict release Clippy and
the release CLI build. The dependent preparation passed the topology gate and
rejected the transport bound of the resulting member:
`authored source member is too large: source-built/rust/program.json`.

## Canonical chunk transport for authored members

A diagnostic run measured the authored member directly:

```text
mantle-diagnostic rust-recipe-bytes=259890409 steps=63882 roots=232254 exports=13
```

A 63,882-action program is larger than the bundle's 64 MiB single-file ceiling.
The bundle already carries large files as canonical chunk sequences, and
hydration concatenates them. `source_built_fixed_point_authoring` now emits that
same canonical sequence for an authored member above the ceiling: full-size
chunks, one final short chunk, sequential indices, one chunk count, per-chunk
digest. The single-file rejection is gone, while the admitted payload byte bound
and every per-chunk validation are unchanged.

`authored_members_use_canonical_chunk_sequences` pins the sequence with an
eight-byte ceiling: single-entry small members, `8,8,4` chunks for a twenty-byte
member, exact per-chunk digest and size, reconstruction to the original text,
and rejection of an empty program or a zero ceiling.

## Staged source completeness

The frozen profile's `mantle-source-tree` record held 1,806 files whose
top-level entries matched `STAGED_SOURCE_TOP_LEVEL_ENTRIES` minus `vendor-deps`,
and its `Cargo.toml` declares bins under `tools`, explicit examples and
auto-discovered integration tests. Preparation therefore rejected planning:

```json
{"error":"Cargo-free native target planning failed: target `generate-operator-command-contract` source .../inputs/mantle-source/tools/generate_operator_command_contract.rs is not readable","code":3,"kind":"internal"}
```

`examples`, `tests` and `tools` are now staged, and the staging fixture asserts
their target sources survive. The frozen revision is unchanged: the Mantle
source tree was reconstructed from the profile record itself (every file
re-checked against its recorded BLAKE3) and the three missing top-level entries
were restored from commit `5fc2ad4b3dfb1ae80ff5060cdd8fb6670230e316` (the
revision whose `Cargo.toml` matches the record, 504 files, every restored file
re-checked against its git blob).

`mantle source bundle refresh-mantle-source` replaced only that record:

```text
input_manifest_blake3  29273e879992b713cec8414772afc4dc99d1454efef158cb86827083401d0820
output_manifest_blake3 e2f0191b78a6715e7c12394166aae1fc07b0fa6b79a9567cb2d4d54754ce5020
mantle files 2310 bytes 42280800
tops {'.cargo': 2, 'Cargo.lock': 1, 'Cargo.toml': 1, 'bootstrap': 305, 'builders': 2, 'config': 79, 'crates': 661, 'examples': 186, 'lib': 30, 'rust-toolchain.toml': 1, 'src': 393, 'tests': 309, 'tools': 9, 'vendor': 331}
```

`2026-11-08-profile-refresh.log` retains the refresh, the 172 passing `self_build`
tests, strict release Clippy and the refreshed CLI identity. The refreshed
profile is `/datapool/mantle-store/cold-inputs-20261030/profile/source-built-fixed-point-sources-refreshed-20261108.json`
(file BLAKE3 `4ebffad013247cf4f9ca582e92a562faa46847db998a7c50b9574cf19ccaf6c6`).
Adopting `e2f0191b…` as the reviewed source-profile pin is an operator decision;
the StageX lineage and native-provider pins were not changed.

## Cargo-free plan boundary

With that profile, preparation hydrated the complete Mantle source, authored the
StageX and Rust programs, and reached the Cargo-free plan for Mantle itself:

```json
{"error":"Cargo-free source topology is blocked: [\"native-host-unit-graph-planning-blocked\", \"native-package-target-planning-blocked\", \"native-unit-graph-planning-blocked\", \"unit-derivation-graph-blocked\"]","code":3,"kind":"internal"}
```

That message named only readiness classes. `cargo_free_blocker_detail` now
projects the first concrete blocker from each planned layer (bounded to eight per
layer and 4 KiB) into the same error. A local `rust-plan --no-cargo-oracle` run
over the same hydrated root reproduced the classes and reported the readings in
`2026-11-08-cargo-free-plan-probe.json` (readiness: source closure ready, the four
later layers blocked; the full 5,297,907-byte receipt log is retained outside the
repository at BLAKE3 `20879b266a1f0b1061c05d195fd56848ec4b0b4f48bc8d92aee93b0ff0196a7a`):

- `missing-registry-dependency-version`: vendored `redb` 3.1.3 pins its own
  **dev**-dependency `redb2_6 = "=2.6.0"`, which the vendor set does not carry
  (2.6.4 and 3.1.3 are present).
- `missing-captured-git-source-fact` for `turso` (a `turso_core`-style git
  dependency enabled through `casita`'s `experimental` feature).
- `missing-path-dependency-manifest` for `vendor-deps/basalt/vendor/ucan`, where
  the vendored copy carries `ucan.snapshot` and `ucan-revision.txt` rather than
  the extracted tree.

The dev-dependency strictness is repaired: a test-only requirement no longer
resolves into build-mode package facts, while the same unsatisfiable exact
requirement still rejects a real build edge.
`build_mode_dev_dependency_requirements_do_not_block_package_facts` pins both
sides. A git edge version is likewise a requirement, not an identity: only an
exact `=x` requirement narrows a captured fact, so `casita`'s
`turso = { version = "=0.8.0-pre.7", git = …, rev = … }` now binds the captured
`turso` fact instead of reporting a missing one.
`native_git_dependency_edges_treat_versions_as_requirements` pins that, and the
`2026-11-08-final-checks.log`/`2026-11-08-git-edge-checks.log` runs retain the
focused suites.

Repeating the probe after those repairs leaves exactly two package/target
blockers, both inside vendored git sources:

```text
missing-path-dependency-manifest  `turso_core` → vendor-deps/turso/../../core/Cargo.toml
missing-path-dependency-manifest  `ucan`      → vendor-deps/basalt/vendor/ucan/Cargo.toml
```

`2026-11-08-cargo-free-plan-probe.json` retains that reading (source closure
ready, the four later layers blocked, two package/target blockers). Both are
intra-repository path dependencies whose layout per-crate vendoring does not
preserve: the `turso` repository's `core/` sibling and `basalt`'s generated
`vendor/ucan` tree. The planner rejects rather than inventing either source. A
source-built proof therefore needs a captured git tree that keeps repository
layout, or a vendor generation that rewrites intra-repository path dependencies
the way `cargo vendor` does; that is an input-cohort decision, not a planner
repair, and it is left open here.

## Checks

`2026-11-08-final-checks.log` is the combined focused run over the final source
state:

```text
source_built_fixed_point_shell::authoring::tests   3 passed
source_built_recipe::tests                        27 passed
source_built_rust_programs (all)                  39 passed, 2 ignored
self_build::tests                                172 passed
dev-dependency regression                          1 passed
registry dependency source regressions             4 passed
```

Leaf `rustfmt --check`, strict release Clippy with `-D warnings` and the release
CLI build passed. The build identity is
`5dca49ec5852ec4fff85679163b0e066f93a3604eb0b324daa0611a0c66cf05d`. No
prepared profile, provider execution, stage equality, fixed point, determinism
receipt or release eligibility is claimed.
