# Build with Crunch — Tasks

## Phase 1: Tier 1 — Smoke (no seed, no network)

- [ ] Run `crunch build examples/hello.ncl --store /tmp/crunch-store` and verify output contains "Hello, crunch!"
- [ ] Run the same build a second time and confirm it skips (caching — output path printed without sandbox invocation)
- [ ] Write a failing-build example (`exit 1`) and confirm crunch exits non-zero with build log on stderr
- [ ] Write a multi-step script example (create directory tree with multiple files) and verify output structure

## Phase 2: Tier 2 — Seed toolchain (bash + coreutils + gcc)

- [ ] Run `crunch bootstrap -o examples/seed.ncl` and verify seed.ncl is generated with valid store paths
- [ ] Build `examples/hello-world.ncl` — compile C hello world with gcc from seed
- [ ] Verify the built binary exists and runs correctly (`$out/bin/hello` prints "Hello from crunch!")
- [ ] Build `examples/mk-hello.ncl` — mkDerivation with buildPhase/installPhase
- [ ] Verify mk-hello output binary runs

## Phase 3: Tier 3 — Fetchers (network required)

- [ ] Build `examples/fetch-file.ncl` — fetchurl of a single file with known hash
- [ ] Build `examples/fetch-tarball.ncl` — fetchTarball with extraction
- [ ] Build `examples/fetch-git.ncl` — fetchGit at a pinned revision
- [ ] Test hash mismatch: modify the hash in fetch-file.ncl and confirm build fails with clear error

## Phase 4: Tier 4 — Composition (multi-output, deps, phases)

- [ ] Build `examples/multi-output.ncl` — derivation with $out and $dev outputs
- [ ] Verify both output paths exist with correct contents (binary in $out, header in $dev)
- [ ] Build `examples/package-set.ncl` — libfoo + app built in dependency order
- [ ] Eval `examples/override.ncl` — overrideAttrs produces modified derivation JSON

## Phase 5: Tier 5 — Real package from source

- [ ] Write a Nickel file that fetchTarball's a small C project (jq or similar), builds it with configure/make/install
- [ ] Build it with crunch and verify the installed binary works
- [ ] Document any failures or missing capabilities discovered during the attempt
