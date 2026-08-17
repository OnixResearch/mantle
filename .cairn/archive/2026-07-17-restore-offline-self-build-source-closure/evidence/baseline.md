# Baseline evidence

## Goal

Restore the checkout-local locked Cargo directory source and obtain current fixed-point self-build evidence without treating metadata, preflight, bootstrap-tool, or stage1 progress as completion.

## Observable completion

- Locked offline metadata resolves from `vendor-deps/` with a fresh empty `CARGO_HOME`.
- Mantle's self-build validator accepts every locked package and every Cargo file/package checksum.
- Fixed staging includes all current compile-time roots while preserving negative exclusions.
- The canonical proof bundle admits stage1/stage2 and bootstrap-tool equality under its selected strict proof policy.

## False-completion cases

A copied single crate, ordinary online Cargo build, proof preflight, bootstrap-tool success, stage1 compilation, or process exit without inspecting the proof bundle is not completion.

## Baseline chronology

### Missing locked package

Pueue task `67` ran:

```text
SNIX_BUILD_SANDBOX_SHELL=/bin/sh nix develop -c cargo metadata --offline --locked --format-version 1 --config .cargo/vendor-config.toml
```

It failed at the known boundary:

```text
error: no matching package named `cap-fs-ext` found
location searched: directory source `vendor-deps`
required by package `mantle v0.1.0`
```

### Pinned proof compiler incompatibility

After Cargo-authored full vendor regeneration, pueue task `81` launched the canonical proof and failed before stage0 because the pinned nightly rejected unstable `char::MAX_LEN_UTF8` in `crates/crunch-build/src/distributed/remote_failure_debug.rs`.

### Host FUSE capability frontier

After the stable expression repair, pueue task `92` staged and validated source/vendor input, fetched bootstrap sources, then failed at the host FUSE descriptor path while starting `musl-seed-toolchain.drv`. This matches the repository's documented host-specific route requiring `CRUNCH_NO_FUSE=1`; it is not a Cargo/source-closure failure.

### Missing tracked compile-time policy root

Pueue task `97` used `CRUNCH_NO_FUSE=1`, built both bootstrap tool roots successfully, and reached sandboxed Mantle compilation. It then failed because staged source lacked:

```text
config/action-result-policy/generated/action-result-policy.json
```

The exact compiler diagnostic was:

```text
error: couldn't read `crates/crunch-store/src/../../../config/action-result-policy/generated/action-result-policy.json`: No such file or directory
```

This proves the fixed top-level staging allowlist, not the ordinary checkout build or vendor payload, owns that blocker.

### Bootstrap musl libc binding frontier

After adding `config/`, pueue task `24` again built both bootstrap tools and compiled the staged workspace through the root package. The bootstrap musl target then rejected all four direct `libc::renameat2` calls because that target's Rust `libc` bindings expose the Linux syscall number and `RENAME_NOREPLACE` flag but not the function symbol. Ordinary glibc tests had hidden this target-specific API-shape gap.

The repair must preserve the kernel's one-step no-clobber operation. An ordinary rename or check-then-rename fallback would compile but would violate the existing publication race contract.

### Transient staged-source path retention

After the syscall-backed no-clobber repair, pueue task `30` used `CRUNCH_NO_FUSE=1`, built the bootstrap tools, compiled the complete staged workspace, and produced the final optimized binary. The unchanged path-leak scan then rejected runtime data containing:

```text
/tmp/build/crunch
```

Inspection of the failed binary tied the value to the production remote-farm Nickel loader's `env!("CARGO_MANIFEST_DIR")` lookup. This was not debug metadata: the compile-time checkout root was embedded as runtime import-path data, so intermediate compilation success remained insufficient.

## Portfolio registry

| Family | Result | State |
|---|---|---|
| Copy only `cap-fs-ext` | Could clear one error but not prove closure completeness | rejected |
| Reuse Nix crate source | Does not establish Cargo's complete directory source/checksum set | rejected |
| Full `cargo vendor --locked` refresh | Empty-home locked offline metadata passes | active |
| Stage unrestricted checkout | Closes omissions by weakening source authority | rejected |
| Add fixed `config/` root | Preserves the allowlist while supplying observed compile input | active |
| Ordinary rename fallback | Compiles but weakens no-clobber publication | rejected |
| Shared `SYS_renameat2` shell | Preserves Linux no-clobber semantics across libc binding shapes | active |
| Keep production `CARGO_MANIFEST_DIR` lookup | Retains a transient build path as runtime data | rejected |
| Source-or-embedded stdlib resolver | Removes checkout identity while preserving installed Nickel imports | active |

## Claim boundary

The repaired vendor payload is ignored checkout-local state. Even after a successful proof, this does not make a fresh Git clone self-contained or prove compiler correctness, seed trust removal, release reproducibility, deployment success, or full Cargo compatibility.
