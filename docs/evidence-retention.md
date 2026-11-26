# Checked-in Evidence Retention and Size Budget Policy

Mantle preserves immutable verification evidence under `.cairn/` while bounding
new artifact growth and keeping historical bulky payloads out of ordinary Nix
build and lint source closures.

## Measured Size Inventory

Measured against the working checkout using `scripts/check-evidence-size-budget.rs --inventory --json`
and `git count-objects -vH`:

| Cost surface | Scope | Count / Files | Bytes |
| --- | --- | --- | --- |
| Working tree (`.cairn/`) | All regular files | 8,461 files | 764,751,294 bytes |
| Working tree (`.cairn/archive/`) | Archived changes & transitions | 5,596 files | 621,763,311 bytes |
| Working tree (`.cairn/changes/`) | Active change records | 2,794 files | 141,646,754 bytes |
| Historical >10 MiB exceptions | 6 Stagex transition/audit JSONs (`2026-07-30-materialize-stagex-lineage-provider`) | 6 files | 405,817,960 bytes |
| Git pack history (`git count-objects -vH`) | Worktree packfile objects | 63,707 objects | 183.22 MiB packed (+6.36 MiB loose) |
| Nix `sourceFilter` `.cairn/` inputs (scoped) | Required test/fixture subtrees in `flake.nix` | 7 files | 607,136 bytes |

## Nix Source Filter Scoping

In `flake.nix`, `sourceFilter` admits only the `.cairn/` evidence directories
consumed directly by build/test targets:

- `.cairn/changes/separate-remote-build-hexagon/evidence/` (`tests/remote_stdio_cli.rs`, `tests/remote_hexagon_v2_parity.rs`)
- `.cairn/archive/2026-07-14-prove-hardware-simulation-build-flow/evidence/` (`src/remote_build/tests/external_batch_hardware_tests.rs`)
- `.cairn/archive/2026-10-04-add-store-backend-selection/evidence/` (`tests/store_archive_cli.rs`)

And `checks.tigerstyle` reuses `src` (with `dylint.toml` included in
`sourceFilter`) rather than copying the unfiltered repository root (`./.`).
All other historical archive payloads remain intact in Git history and on disk
without rewriting or deletion.

## Bounded Size Guard and Digest-Bound Exceptions

`config/evidence-size-budget.json` enforces:

1. **Per-file default budget (`10,485,760` bytes / 10 MiB)**: any newly checked-in
   file under `.cairn/` larger than 10 MiB is rejected by
   `scripts/check-evidence-size-budget.rs`.
2. **Nix-filtered `.cairn/` budget (`1,048,576` bytes / 1 MiB)**: ensures Nix
   source-filtered `.cairn/` test inputs remain compact (`607,136` bytes today).
3. **Explicit justified exceptions**: the 6 pre-budget 66–69 MB Stagex
   transition/audit JSON files are recorded with exact `size_bytes`, BLAKE3
   digests, and rationale. Any drift or missing excepted file fails the guard.
4. **Cold retrieval verification**: `scripts/check-evidence-size-budget.rs --self-test`
   exercises isolated oversized-file rejection, justified fixture exception
   admission, and cold BLAKE3 payload retrieval plus missing/corrupted failure
   handling.

Run the guard and inspect the reproducible inventory with:

```bash
nix develop -c cargo -Zscript scripts/check-evidence-size-budget.rs --self-test
nix develop -c cargo -Zscript scripts/check-evidence-size-budget.rs --inventory --json
```

## Non-Claims

- Evidence size budgets and digest checks verify storage bounds and file
  integrity only; they do not promote historical transcripts into current build
  or release proof.
