# Final validation summary

## Accepted Radiance evidence

V16 ran from implementation commit
`5e35c8a518e871cbbf844598b274ddb7842c9316`.

- Disposition: `match`
- Fixed-point BLAKE3: `a06905539bd81c9581218ca648e98fb7a55c5a7e79847b6c840181a3e2605b07`
- Fixed-point bytes: `1285492`
- Receipt BLAKE3: `da80e6d4f2fbf4adb82aabfaeef37e4f61e8a628024d999672458f0589e571c9`
- Audit BLAKE3: `2da1ee4274c085598aef78258cbaec4fc84cc764d80db469356c198160dd004e`
- Execution events: 50 allowed, 0 denied
- Proof-time network requests: 0
- Replay: pass

The receipt binds eight source, executable, runtime-tree, and output roles.
V14 and V15 remain historical evidence. V98 remains unchanged.

## Passing focused checks

The following checks passed:

- Rust formatting for the new core and root package.
- 14 core tests.
- Core `wasm32-unknown-unknown` build.
- Strict core Clippy across all targets and features.
- 8 Radiance shell tests.
- Positive and negative explicit VCS checkout planner tests.
- Foreign-tree digest, mode, link, and tamper test.
- Strict root-package Clippy with `--no-deps`.
- Architecture checker self-test and repository scan: 0 findings, 10 negative fixtures.
- Machine contract check: 30 contracted surfaces and 63 classified surfaces.
- Three Nickel typechecks and generated-profile byte-equivalent JSON comparison.
- Focused Nix core, core-WASM, shell, and architecture checks.
- Cairn validation and proposal, design, and tasks gates.
- Tracey coverage: 157/157 requirement IDs referenced.
- Accepted-spec synchronization: eight Radiance requirements applied.
- Diagnostic Octet core check across all targets and features: clean, 0 findings.
- Diagnostic root Octet check: 42,188 inherited findings and 0 Radiance-path findings.
- Core Tiger Style check across all targets and features.
- Root Tiger Style scan: 2,326 inherited error diagnostics and 0 Radiance-path findings.

Command output is in this directory.

## Broad and external blockers

The broad root test command completed with 2,532 passed, 2 failed, and 72
ignored. Both failures are outside the Radiance paths:

- `oci_projection_shell::tests::frontend_cas_rejects_special_files_before_oci_projection`
  expected a different diagnostic string.
- `protected_exec_seccomp::linux::tests::seccomp_supervisor_reads_deep_descendant_exec_path`
  exceeded the existing 30,000 ms descendant-exit bound.

The pinned Octet command is blocked before Octet execution by the known
rust-src import mismatch:

- specified `sha256-q/gu/3mAuLgNfJlxV/Sw1jttbi4PIBjN+XH0bGmB5NQ=`
- got `sha256-WTRv7eyiu+VOfb8+90cALNJrUa3uLwRFIaeEr+tAIjQ=`

`nix flake check --no-build -L` is blocked while evaluating the existing
SpaceWasm bundler source derivation. The store reports its `.drv` path as not
valid.

The full `nix flake check -L` advanced further, including the Radiance checks,
but failed in the unrelated durable-file-publication-adoption check after
remote-builder process and cgroup cleanup failures.

These blockers do not weaken or replace the passing focused checks. They also
do not establish full-repository acceptance.

## Claim boundary

The result proves bounded source admission, protected execution, exact
lineage, route-local convergence, cross-route equality, and immutable
publication for the recorded inputs. It does not prove compiler correctness,
seed trust, semantic equivalence, or universal reproducibility.
