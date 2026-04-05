## Context

v0 uses input-addressed derivations exclusively. Output paths are
known before the build starts — computed from the ATerm hash of the
derivation. The pipeline assumes this: `convert()` fills output paths
into the `Derivation` struct, `Builder` checks those paths on disk
for cache hits, and the sandbox receives the final path in `$out`.

CA derivations break this assumption. The output path depends on what
the build produces, so it can't be known pre-build. The pipeline
needs a two-phase approach: provisional paths during the build,
final paths after.

Nix's CA derivation implementation (experimental) uses a similar
two-phase scheme. The provisional "output hash" is computed from
the derivation inputs (like input-addressed), and the final path
is computed from the NAR hash of the output. References to the
provisional path in the output are rewritten.

## Goals / Non-Goals

**Goals:**
- CA derivations as the default for new derivations
- Input-addressed mode retained for seed/bootstrap compatibility
- Identical outputs share store paths
- Self-reference and cross-reference rewriting
- Works with the existing `BuildService` trait (no changes to
  snix-build's interface)

**Non-Goals:**
- Floating CA derivations (early-cutoff optimization where
  downstream builds are skipped if the CA path didn't change).
  This is a scheduler optimization, not a correctness requirement.
  Can be added later.
- Intensional store / content-addressable deduplication at the
  file level (dedup within outputs). Store-level dedup is a
  separate concern.
- Changing FOD behavior.

## Decisions

### 1. Provisional paths use hash_placeholder

**Choice:** Before a CA build, output paths are set to
`hash_placeholder(output_name)` — the same mechanism Nix uses for
`builtins.placeholder`. The placeholder is a deterministic string
derived from the output name.

**Rationale:** `hash_placeholder` already exists in nix-compat and
produces fixed-length strings matching store path length. Build
scripts that use `$out` get a writable path. The placeholder is
recognizable for rewriting.

**Alternative:** Use the input-addressed path as provisional.
Rejected — this conflates two addressing modes and makes it
unclear whether a path is provisional or final.

**Implementation:**
```rust
// In convert(), for CA derivations:
let placeholder = hash_placeholder(output_name);
// Set in environment but NOT in Derivation.outputs[].path
environment.insert(output_name.clone(), placeholder.into());
// outputs[].path = None (resolved post-build)
```

### 2. Post-build resolution in Builder

**Choice:** After `BuildService::do_build` returns, the Builder:
1. Computes NAR hash of each output (already done for PathInfo)
2. Computes the CA store path via `build_ca_path`
3. Scans output for provisional path references
4. Rewrites provisional → final paths in the output bytes
5. Re-computes NAR hash after rewriting (content changed)
6. Moves/copies output to final path in the store
7. Updates KnownPaths with the resolved path

**Rationale:** Steps 1-2 and 4-5 are the core CA logic. Step 3
reuses the existing refscan infrastructure. Step 6 is a filesystem
operation. Step 7 enables downstream builds.

**Alternative:** Rewrite before NAR hashing (compute hash of
rewritten content). This is actually what we must do — the final
CA path is the hash of the *rewritten* content, not the raw
build output. Otherwise the path would be wrong if the output
contains self-references.

**Corrected order:**
1. Scan output for provisional path references
2. Rewrite provisional → temporary marker (since final path
   is not yet known, use a fixed-width zeroed placeholder)
3. Compute NAR hash of the rewritten output
4. Compute CA store path from that hash
5. Rewrite the temporary marker → final CA path
6. Persist to store

Wait — this is the chicken-and-egg problem. The final path
depends on the content hash, but the content contains the path.

**Resolution (Nix's approach):** Nix solves this by:
1. Replacing all self-references with a fixed "self-reference
   marker" (all zeros of the same length)
2. Computing the NAR hash of the marker-replaced content
3. Computing the CA path from that hash
4. Replacing the markers with the actual CA path
5. NOT re-hashing (the CA path is derived from the
   marker-replaced content, which is the canonical form)

The store records that this path has a self-reference, so
verification can account for it.

**Choice (final):** Follow Nix's approach. Self-references are
replaced with a zero marker before hashing. The CA path is
derived from the zero-replaced content. The zero markers are
then replaced with the final path in the on-disk output.

### 3. addressing_mode enum in Nickel

**Choice:** Add `addressing_mode` field defaulting to
`'content-addressed`. Input-addressed remains available for
seed-dependent builds where path stability matters.

**Rationale:** New derivations should get CA by default. But
seed toolchain wrapping and bootstrap builds need deterministic
paths tied to inputs, not content.

**Implementation:** In `lib/derivation.ncl`:
```nickel
addressing_mode | [| 'input-addressed, 'content-addressed |]
               | doc "How output paths are computed"
               | default = 'content-addressed,
```

`crunch-glue` reads this from `CrunchDerivation` and branches
in `convert()`.

### 4. KnownPaths deferred resolution

**Choice:** `KnownPaths` stores CA derivations with
`output_paths: HashMap<String, Option<StorePath>>` — `None`
before build, `Some` after. `get_output_path()` returns the
resolved path or an error indicating the build hasn't happened.

**Rationale:** The Builder already processes derivations in
dependency order. When it reaches a derivation whose CA input
hasn't been resolved, that's a bug (the input should have been
built first). So `None` is an assertion failure, not a wait
condition.

### 5. PathInfoService required for CA cache

**Choice:** CA cache checks require `PathInfoService`. The
system looks up whether a derivation was previously built and
what CA path it resolved to. Filesystem existence alone is
insufficient — we need the mapping from derivation identity
to CA output path.

**Rationale:** Input-addressed derivations can check cache by
path existence (the path is deterministic from inputs). CA
derivations can't — the path depends on content, so we need
a record of what a derivation resolved to last time.

**Dependency:** This creates a hard dependency on the persistent
PathInfoService change. CA cache checks don't work with the
current in-memory HashMap because it's lost on restart.

Without persistent PathInfo, CA derivations still work but
always rebuild (no cache). This is acceptable as an intermediate
state.

## Risks / Trade-offs

**[Rebuild-everything transition]** Switching the default to
`'content-addressed` means existing v0 builds get different
output paths. There's no migration path — you rebuild. This is
fine for a project with no users yet.

**[Self-reference complexity]** The zero-marker rewriting is
subtle. Off-by-one in marker length or incorrect scan boundaries
corrupt outputs silently. Needs thorough testing with binaries
that contain embedded paths.

**[Performance]** Post-build rewriting adds a full scan of every
output. For large outputs (compilers, debug symbols), this is
measurable. Acceptable for correctness; optimize later if needed.

**[PathInfoService dependency]** CA cache requires persistent
PathInfo. If that change isn't done first, CA derivations work
but always rebuild. Sequence: wire-store-dir → persistent-pathinfo
→ ca-derivations. Or do ca-derivations without caching and add
caching when PathInfo lands.
