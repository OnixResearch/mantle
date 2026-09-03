# Final source-observation validation

Validated implementation commit: `2596144f0ff3fcb670f73dac7d29164883b6d199`

## Focused checks

`focused.log` records all commands and output.
Every command completed with status `0`.

- `crunch-source-core`: 17 passed.
- Source bundle and monotonic ingest: 104 passed.
- `crunch-release-core`: 269 passed, plus one compile-fail doctest.
- Release-source shell: 7 passed.
- Witness source-acquisition shell: 6 passed.
- Release CLI source filter: 26 passed and one fixture writer stayed ignored.
- Project adapters: 7 passed.
- Checked fetch and Git nominal values: 2 passed.
- Source-core WASM: passed.
- Strict Clippy for the source core, release core, Mantle binary, and release CLI: passed.
- Source-core Tiger Style: passed.
- Source-observation architecture: zero findings and 16 negative fixtures.
- Machine contracts: 27 contracted and 60 classified surfaces.
- Durable-publication Nickel positive and negative tests: passed.
- Focused formatting and `git diff --check`: passed.
- Strict Cairn validation: passed.
- Pre-sync Tracey coverage: 155 of 155 accepted requirements referenced.
- Post-sync default Tracey coverage: 157 of 157 requirements referenced.
- Focused source-observations Tracey coverage: 177 of 177 requirements referenced.
- Cairn proposal, design, and tasks gates: passed; all 21 tasks are complete.

## Nix checks

`nix.log` records the final Nix checks.
The command completed with status `0`.

- `nix flake check --no-build -L`: passed.
- Source-core host and WASM checks: passed.
- Source-observation architecture check: passed.
- Durable-file-publication adoption check: passed.
- Repository formatting check: passed.
- Strict first-party Clippy check: passed.
- Full pinned Tiger Style check: passed.

The refreshed durable-publication adoption receipt has BLAKE3:

`63befee881c12c4edda0b8c523e81e325b632d8b32dc6d454559866d37b092f5`

## Broader test boundary

The earlier committed-source `check-first-party-quality.sh` run passed its bridge, formatting, and strict Clippy stages.
Its serialized test stage reported 2,519 passed, 3 failed, and 72 ignored.

The three failures are existing host-sensitive boundaries outside this change:

- `oci_projection_shell::tests::frontend_cas_rejects_special_files_before_oci_projection`;
- `protected_exec_ptrace::linux::tests::ptrace_supervisor_denies_digest_mismatch_before_exec`;
- `protected_exec_seccomp::linux::tests::seccomp_supervisor_reads_deep_descendant_exec_path`.

`first-party-quality.log` preserves the exact diagnostics.
Focused source, release, Clippy, Tiger Style, and Nix checks remain green.

## Compatibility

The selected source-bundle v1 fixture remains byte-for-byte unchanged.

- File BLAKE3: `b8abb366a886bb79314411a3d479a97fe8481173334c238333b9118fe7ab8030`.
- Embedded manifest BLAKE3: `8ed1103b5de3054ee13ea391af805e276e3cc3b6ceaa50b2149193adb1ff1777`.

Legacy `SourceAcquisition` bytes remain unchanged when the observation subject is absent.
The new golden release source binding has BLAKE3 `3a10bb482e9fec2acd73a4cd7ac97e7e449da5be8c54f63911f6487d1765360b`.

## Claim boundary

These checks prove the implemented source-observation, ingest, compatibility, and release-linkage boundaries.
They do not prove source ownership, upstream intent, review quality, license compliance, build correctness, or release eligibility.
