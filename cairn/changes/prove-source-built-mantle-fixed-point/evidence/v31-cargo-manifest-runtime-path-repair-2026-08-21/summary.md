# V31 Cargo manifest runtime-path repair

## Goal

Use the preserved V30 native and Rust providers to test the repaired source
closure before another complete proof.

## Diagnostic authority

- Source commit: `33d25bed`.
- Release orchestrator BLAKE3:
  `62f104fac95018a3e073954de2a2679baf5466fa3817a2e5f90819e3ab11edfd`.
- Preserved Rust provider metadata BLAKE3:
  `84e7d1812c55f8d1576760acd7818f932704b7b3ade1d10178590dccb696a2aa`.
- Preserved 17-member toolchain policy BLAKE3:
  `ac1fe0d21931a8b8d30f7f05a6bfef9b5d529b380dfba51d3d50fe81d6caa8f7`.
- Hermeticity: strict.
- Strict proof admission: true.

This was a bounded diagnostic. Reused providers cannot satisfy the promoted
source-built proof.

## Transfer diagnostic

Remote pueue task `264` first rejected missing `cc@1.2.59` vendor files. The
operator rsync used an unanchored `--exclude=target`. It removed Cargo package
paths such as `vendor-deps/cc/src/target/`.

The failed output was retained. A second transfer used only root-anchored
exclusions and exact checksum parity. It restored
`vendor-deps/cc/src/target/apple.rs` and every other nested `target` path.

## Corrected replay result

Remote pueue task `265` passed the complete vendor and Git source planning
boundary. It executed 683 stage1 units. It then failed at `crossterm@0.29.0`:

```text
error: Can't open Cargo.toml: Os { code: 2, kind: NotFound, message: "No such file or directory" }
   --> vendor-deps/crossterm/src/lib.rs:230:10
    |
230 | #![doc = document_features::document_features!()]
```

The planned environment set:

```text
CARGO_MANIFEST_DIR=/mantle/release/source/vendor-deps/crossterm
```

That value was a deterministic identity but not a readable host path. The
`document_features!()` proc macro reads `CARGO_MANIFEST_DIR/Cargo.toml` during
compilation.

## Repair

Deterministic non-custom Rust units now use:

```text
/proc/self/cwd/<package-relative-path>
```

Rustc already runs from the admitted physical source root. The procfs alias is
absolute, stable across proof roots, and readable during macro expansion. The
existing `/mantle/release/source` rustc remap still owns source diagnostics and
source-path identity. Custom-build behavior does not change.

A focused negative control compiled equal rlibs in two physical roots while
setting physical `CARGO_MANIFEST_DIR` values. Rustc's path remap did not rewrite
those `env!` strings. The two rlibs had different BLAKE3 values and retained
the physical roots. See `remap-env-experiment.txt`.

## Validation

Pueue task `6281` established the pre-change deterministic-path baseline. Pueue
task `6319` recorded `local-validation.log`. It passed:

- three positive and negative deterministic manifest-path tests;
- the deterministic release-path plan test;
- the unchanged custom-build manifest-path test;
- changed-file Rust formatting;
- focused strict Clippy with only the recorded baseline allowances;
- `git diff --check`.

The subprocess test runs from a temporary source root. It reads the selected
package manifest through `/proc/self/cwd` and rejects the nonexistent logical
release path. A corrected preserved-provider replay is still required.

## Non-claims

This diagnostic did not construct providers and cannot satisfy the promoted
proof. It did not reach stage2 or create a final receipt. The repair does not
authorize Cargo, network access, ambient source discovery, or unrelated procfs
reads.
