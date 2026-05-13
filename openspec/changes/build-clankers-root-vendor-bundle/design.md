## Context

The parent `build-clankers-with-crunch` change proved two small Clankers Rust crates with one generated Nickel input per registry crate. The root `clankers` binary is qualitatively different: an offline `cargo vendor --locked --offline --versioned-dirs` probe resolved 1,169 vendored package directories and 1,195,893,442 bytes of vendor data before compression. Committing that vendor tree, or generating ~1,000 Nickel fetch inputs by hand, would make the Crunch repo unwieldy and make future root-binary iterations expensive.

## Goals / Non-Goals

**Goals:**
- Keep the root proof fixed-output and reproducible.
- Keep Cargo offline at build time.
- Avoid committing the large source/vendor payload to git.
- Keep `packages/clankers/clankers.ncl` small enough to review and evaluate.
- Record enough metadata to recreate and verify the external payload.

**Non-Goals:**
- Do not make a public binary cache or upload artifact in this change.
- Do not claim root Clankers correctness beyond a build plus `--help`/`--version` smoke.
- Do not mutate the source Clankers checkout's pre-existing WIP files.

## Decisions

### 1. Use a single fixed external bundle for root source plus Cargo vendor data

**Choice:** Materialize `.crunch-drain/artifacts/clankers-root-bundle.tar.zst` locally and reference it from `packages/clankers/clankers.ncl` with `crunch.fetchTarball` and a recursive unpacked-tree SRI hash.

**Rationale:** This preserves Crunch's fixed-output derivation semantics while avoiding a monolithic committed artifact. It also keeps the derivation input graph small: the root build depends on the bundle plus the existing Rust/bootstrap inputs, rather than one input per registry/git crate.

**Alternative rejected:** Commit the vendor tree or a compressed vendor archive under `packages/clankers/`. That would add roughly a GiB of payload to git and make normal repository operations worse.

**Alternative rejected:** Keep per-crate `fetchTarball` entries for every dependency. That is review-hostile, would generate a huge Nickel file, and still needs special handling for git dependencies and generated `.cargo-checksum.json` metadata.

### 2. Bundle layout is Cargo-native

**Choice:** The bundle top-level directory contains the Clankers source checkout, a `vendor/` tree produced by `cargo vendor --locked --offline --versioned-dirs`, and `.cargo/config.toml` that replaces `crates-io` and git sources with the bundled vendor directory.

**Rationale:** Cargo already validates vendored `.cargo-checksum.json` files and knows how to build a workspace from that layout. Using Cargo-native vendor data avoids reproducing Cargo source replacement logic in Nickel or shell.

### 3. Metadata is committed, payload is not

**Choice:** Commit `packages/clankers/clankers-root-bundle.json` with source revision/status, bundle path, archive digest, recursive unpacked hash, vendor crate count, byte size, and the exact generation command. Keep `.crunch-drain/artifacts/*` out of git.

**Rationale:** The metadata makes the local proof auditable and repeatable without bloating the repo. A future distribution change can replace the local file URL with a content-addressed artifact mirror while keeping the same NCL/build contract.

## Risks / Trade-offs

**Large local artifact** → Store under `.crunch-drain/artifacts/`, clean temporary staging dirs after each build, and record sizes in metadata/evidence.

**Host-local file URL** → This is acceptable for the local proof. The committed metadata makes the portability gap explicit; a future mirror/cache can swap only the URL while preserving the fixed hash.

**Native build-script tools** → Start with the same Rust/bootstrap toolchain. If the root build fails due to a concrete native tool (`pkg-config`, `cmake`, C compiler path, wasm tool, etc.), add the smallest required Crunch input and record the error-to-tool evidence.

## Validation Plan

1. Generate and hash the local root bundle.
2. Evaluate `packages/clankers/clankers.ncl` and shell-check the builder script.
3. Build with `nix-shell -p bubblewrap --run './target/debug/crunch build packages/clankers/clankers.ncl --store "$PWD/.crunch-drain/store" -j 4 --trust-unsigned'`.
4. Verify `$out/bin/clankers` exists and run a network-free `--version` or `--help` smoke.
5. Record build and smoke evidence in `bootstrap/evidence/clankers-root-build.json`.
