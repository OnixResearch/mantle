## Why

crunch can build derivations but cannot download anything. There's no
`fetchurl`, `fetchTarball`, or `fetchGit`. Every input must already exist
on disk as a seed path. This makes crunch unusable for real-world builds
where source tarballs, patches, and git repos need to be fetched.

The fetcher infrastructure already exists in two places we control:
snix-redox's `fetchers.rs` (sync, ureq-based, battle-tested against 70+
Redox packages) and upstream snix-glue's `Fetch` enum + `fetchurl.rs`
derivation parser. crunch needs to wire these together with its own
Nickel-native interface.

## What Changes

- **Nickel fetch helpers**: `crunch.fetchurl`, `crunch.fetchTarball`,
  `crunch.fetchGit` functions in the stdlib that produce FOD records
  with `builder = "builtin:fetchurl"` and the right environment
  variables. Users write Nickel, not raw derivation plumbing.

- **Builtin fetcher execution**: When the build orchestrator encounters
  `builder = "builtin:fetchurl"`, it bypasses the bwrap sandbox and
  executes the fetch directly (download + hash verify). Adapted from
  snix-redox's `fetchers.rs`.

- **Hash verification + auto-fix**: On hash mismatch, report both
  expected and actual hashes with the `.ncl` file location to update.
  `--fix` rewrites the hash in-place (from the defaults spec).

- **Nickel `fetch` contract**: A contract for fetch derivations that
  validates URL format, hash format, and required fields at eval time.

## Capabilities

### New Capabilities

- `fetch-url`: Download a file by URL, verify content hash, store as FOD
- `fetch-tarball`: Download + decompress + unpack a tarball, verify NAR hash
- `fetch-git`: Clone a git repo at a specific rev, verify NAR hash
- `fetch-auto-fix`: On hash mismatch, suggest or apply the correct hash

### Modified Capabilities

- `build-pipeline`: Orchestrator gains a builtin fetcher path that
  skips the sandbox for `builder = "builtin:fetchurl"` derivations
- `nickel-stdlib`: New fetch helpers added to `lib.ncl`

## Impact

- **Files**: `lib/fetch.ncl` (new), `crates/crunch-build/src/fetcher.rs`
  (new), `crates/crunch-build/src/orchestrate.rs` (modified),
  `src/main.rs` (modified for `--fix`), `lib/lib.ncl` (re-export)
- **Dependencies**: `ureq`, `flate2`, `tar`, `lzma-rs`, `bzip2-rs`,
  `ruzstd` added to crunch-build
- **Testing**: Unit tests for each fetch variant, integration test that
  fetches a real URL and verifies the hash
