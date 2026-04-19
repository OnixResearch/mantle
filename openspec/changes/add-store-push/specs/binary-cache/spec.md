## ADDED Requirements

### Requirement: Store push to directory

The store layer MUST provide a library function that exports selected signed
PathInfo entries to a flat Nix binary cache directory layout.

For each selected PathInfo the function MUST:
1. Render the NAR archive from the castore node using `write_nar`
2. Write the NAR to `<dest>/nar/<nar-sha256-nixbase32>.nar`
3. Construct a `NarInfo` with correct `URL`, `FileHash`, and `FileSize` fields
4. Write the narinfo to `<dest>/<store-path-hash>.narinfo`
5. Write `<dest>/nix-cache-info` if it does not already exist

The function MUST return a structured `PushReport` with counts of pushed paths,
skipped-unsigned paths, skipped-already-present paths, and total bytes written.

#### Scenario: Push a single signed path

- GIVEN a local store with one signed PathInfo for path `hello`
- AND a writable empty target directory
- WHEN `export_paths_to_cache_dir([hello], dest)` is called
- THEN `<dest>/<digest>.narinfo` exists and is parseable as a valid narinfo
- AND `<dest>/nar/<nar-hash>.nar` exists and its sha256 matches the narinfo `NarHash`
- AND `<dest>/nix-cache-info` exists with the correct `StoreDir`
- AND the report shows `pushed_count = 1`

#### Scenario: Push skips unsigned PathInfo by default

- GIVEN a local store with one unsigned PathInfo for path `unsigned-pkg`
- AND `trust_unsigned` is false
- WHEN `export_paths_to_cache_dir([unsigned-pkg], dest)` is called
- THEN no narinfo or NAR file is written for `unsigned-pkg`
- AND the report shows `skipped_unsigned_count = 1`

#### Scenario: Push includes unsigned PathInfo when trust_unsigned is set

- GIVEN a local store with one unsigned PathInfo for path `unsigned-pkg`
- AND `trust_unsigned` is true
- WHEN `export_paths_to_cache_dir([unsigned-pkg], dest)` is called
- THEN `<dest>/<digest>.narinfo` is written for `unsigned-pkg`
- AND the report shows `pushed_count = 1`

#### Scenario: Idempotent push skips already-present paths

- GIVEN a target directory that already contains `<digest>.narinfo` for path `hello`
- WHEN `export_paths_to_cache_dir([hello], dest)` is called again
- THEN the existing narinfo is not overwritten
- AND no NAR is re-rendered for `hello`
- AND the report shows `skipped_already_present_count = 1`

#### Scenario: Push multiple paths

- GIVEN a local store with three signed PathInfo entries
- AND a writable empty target directory
- WHEN `export_paths_to_cache_dir([a, b, c], dest)` is called
- THEN three narinfo files and up to three NAR files exist in the target
- AND the report shows `pushed_count = 3`

#### Scenario: Narinfo references match stored PathInfo

- GIVEN a signed PathInfo with two runtime references
- WHEN it is pushed to a directory
- THEN the narinfo `References` field lists both referenced store path names
- AND the narinfo `NarHash` and `NarSize` match the PathInfo values

### Requirement: NAR rendering as a public StoreHandle method

`StoreHandle` MUST expose a public method for rendering a NAR archive from a
castore node to an arbitrary `AsyncWrite` sink.

This method MUST stream the NAR without buffering the full archive in memory.

#### Scenario: Render NAR to a file

- GIVEN a castore node representing a directory tree
- AND a writable file handle
- WHEN `store.render_nar(node, file)` is called
- THEN the file contains a valid NAR archive of that tree
- AND the NAR sha256 matches what `PathInfo.nar_sha256` records

#### Scenario: Render NAR to a hasher

- GIVEN a castore node
- WHEN `store.render_nar(node, hasher_writer)` is called
- THEN the hasher receives the full NAR byte stream without intermediate files

### Requirement: CLI store push subcommand

The CLI MUST provide `crunch store push` for exporting build results to a
binary cache directory.

Required arguments:
- `--to <path>` — target directory (required)
- `<path>...` — store paths to push (optional if `--all` is given)
- `--all` — push all signed paths in the local store
- `--signing-key <path>` — signing key file (for narinfo signatures)
- `--trust-unsigned` — include unsigned PathInfo entries

The command MUST use the store mutation lock to prevent concurrent push and
GC operations.

#### Scenario: Push named paths to directory

- GIVEN a successful build of `hello` and `world`
- WHEN `crunch store push --to /srv/cache hello world` runs
- THEN both paths are exported to `/srv/cache/`
- AND the command prints a summary of pushed paths and bytes

#### Scenario: Push all paths

- GIVEN a local store with five signed paths
- WHEN `crunch store push --all --to /srv/cache` runs
- THEN all five paths are exported

#### Scenario: Push with no signed paths warns

- GIVEN a local store with only unsigned PathInfo entries
- AND `--trust-unsigned` is not passed
- WHEN `crunch store push --all --to /srv/cache` runs
- THEN the command prints a warning that no paths were pushed
- AND it exits with code 0

#### Scenario: Push to nonexistent directory

- GIVEN `--to /nonexistent/dir`
- WHEN `crunch store push --all --to /nonexistent/dir` runs
- THEN the command creates the directory and its `nar/` subdirectory
- AND proceeds with the push

### Requirement: nix-cache-info file

The push operation MUST write a `nix-cache-info` file in the target directory
if one does not already exist. The file MUST contain at least `StoreDir` set
to the store prefix used by this crunch instance.

#### Scenario: nix-cache-info reflects store prefix

- GIVEN crunch running with default store prefix `/crunch/store`
- WHEN paths are pushed to an empty directory
- THEN `nix-cache-info` contains `StoreDir: /crunch/store`

#### Scenario: Existing nix-cache-info is preserved

- GIVEN a target directory with an existing `nix-cache-info`
- WHEN paths are pushed
- THEN the existing `nix-cache-info` is not modified
