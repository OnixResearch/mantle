# Design: Complete live-bootstrap source chain

## Context

The current full-source chain is scaffolded but incomplete. Several bootstrap
files deliberately emit placeholder errors. The parent repair change treats that
archive history as partial scaffolding only; this change owns the implementation
and proof work.

## Goals / Non-Goals

**Goals:**

- Build the live-bootstrap ladder without the musl.cc binary provider.
- Preserve the normalized `bootstrap/seed.ncl` provider contract consumed by
  later bootstrap stages.
- Record proof metadata that prevents a legacy-provider run from satisfying the
  full-source claim.

**Non-Goals:**

- Remove the legacy seed before this chain passes full validation.
- Count prerequisite-only, placeholder, deferred, or archived partial-scaffolding
  evidence as full-source proof.

## Decisions

### 1. Stage order stays explicit

**Choice:** Implement and validate stages in this order:

1. `bootstrap/stage0-posix.ncl`
2. `bootstrap/mes.ncl`
3. `bootstrap/tinycc.ncl`
4. `bootstrap/binutils-tcc.ncl`
5. `bootstrap/gcc-4.0.ncl`
6. `bootstrap/gcc-4.7.ncl`
7. `bootstrap/gcc-10.ncl`
8. `bootstrap/musl-full.ncl`
9. `bootstrap/binutils-full.ncl`
10. `bootstrap/seed-full.ncl`

The GCC ladder is mandatory: tinycc → gcc-4.0.4 → gcc-4.7.4 → gcc-10.x
(or newer). This design does not permit replacing gcc-4.0.4 with an unspecified
alternative unless a separate spec delta changes the ladder requirement.

**Rationale:** The current placeholders and archived work already name these
transition points. Keeping the order explicit makes validation resumable and
prevents one final green command from hiding skipped stages.

### 2. Later stages consume only the normalized provider

**Choice:** `bootstrap/seed-full.ncl` must expose the same public contract as
`seed-legacy.ncl`: `name`, `target`, `dynamic_linker`, `toolchain`, provider
metadata, target-prefixed tool paths, headers, libraries, retained-tool
metadata, reduction metadata, and provider notes.

**Rationale:** Existing bootstrap derivations already consume the normalized
contract. Provider-specific raw paths would couple later stages to temporary
live-bootstrap layout and make future provider swaps harder to validate.

### 3. Proof metadata gates status promotion

**Choice:** Full-source status requires command transcripts plus provider kind,
manifest digest, provider output digest, proof bundle digest, and docs/trust-root
separation evidence. StageX-class status additionally requires audited seed
digest, lineage manifest digest, stage graph digest, normalized provider digest,
staged source digest, stage1/stage2 binary digests, bootstrap-tool digests,
protected execution audit digest when used, final proof bundle digest, and
canonical reproducibility report digest.

**Rationale:** Byte-identical stage1/stage2 alone is insufficient if it was built
with the legacy fetched provider. The proof must bind the binary result to the
selected source-built provider.

## Per-stage implementation plan

Each placeholder is replaced by a normal crunch derivation, not a status stub:

1. `bootstrap/binutils-tcc.ncl`: build the live-bootstrap TinyCC-era helper
   chain first (`sed`, `patch`, `gawk`, archive/decompression tools), build
   musl-1.1.24 with TinyCC as the libc boundary, then build post-musl `m4`,
   `flex`, `bison`, and `grep` linked against musl before building
   `binutils-2.30` so `as`, `ld`, `ar`, `ranlib`, `nm`, and `objcopy` are
   available to GCC 4.0.4.
2. `bootstrap/gcc-4.0.ncl`: build gcc-4.0.4 from TinyCC plus the TinyCC-era
   `make` and `binutils-2.30` outputs. This stage is mandatory; replacing it
   requires a separate delta to the GCC ladder requirement.
3. `bootstrap/gcc-4.7.ncl`: build the intermediate libc/tooling surface needed
   by gcc-4.7.4, then build gcc-4.7.4 from gcc-4.0.4.
4. `bootstrap/gcc-10.ncl`: build `gmp`, `mpfr`, and `mpc` with gcc-4.7.4, then
   build gcc-10.x or newer as the modern compiler transition.
5. `bootstrap/musl-full.ncl`: build musl-1.2.x with the source-built gcc-10
   compiler and install headers/libraries into the normalized target sysroot.
6. `bootstrap/binutils-full.ncl`: build final binutils with gcc-10 and install
   the target-prefixed tool set consumed by `seed-full.ncl`.
7. `bootstrap/seed-full.ncl`: normalize gcc, musl, and binutils outputs into the
   same public shape as `seed-legacy.ncl`, including `bin/`, `<target>/include`,
   `<target>/lib`, retained tool names, reduction metadata, provider notes, and
   `share/crunch-bootstrap/provider.json`.

Implementation must keep the functional-core boundary simple: derivation file
content declares sources and dependencies; any Rust status/proof helpers parse
plain data and keep filesystem writes in the CLI shell.

## Source artifact and pinning strategy

Every source tarball, generated patch, and carried live-bootstrap patch must be
pinned where it is consumed. Existing placeholders already pin binutils-2.30,
gcc-4.0.4, gcc-4.7.4, gcc-10.5.0, musl-1.2.5, and binutils-2.41 with upstream
archive hashes. New helper-tool sources must follow the same pattern. Crunch-owned
manifest and status fingerprints use BLAKE3; upstream fixed-output fetch hashes
may stay SHA-256 for Cargo/Nix-style interoperability when the derivation names
that reason.

Validation must include a source-pin audit that reads the changed bootstrap
`.ncl` files and fails if a fetch source, patch, or generated artifact lacks a
URL/path, digest, and provenance note. This change adds a repo-owned Rust script
`./scripts/check-bootstrap-source-pins.rs`; its audit output is stored beside the
stage transcript that first consumes the source.

## Transcript and fail-closed validation design

Validation evidence lives under this change's `evidence/` directory until the
change archives. Each V task owns one markdown transcript file with `Task-ID`
and `Covers` metadata. Every stage transcript records:

- command line;
- provider selection (`legacy`, `source-root`, or `stagex-lineage`);
- exit status;
- output path or failure class;
- placeholder/deferred/archive rejection result.

The required command set is literal:

- `crunch build bootstrap/stage0-posix.ncl`
- `crunch build bootstrap/mes.ncl`
- `crunch build bootstrap/tinycc.ncl`
- `crunch build bootstrap/binutils-tcc.ncl`
- `crunch build bootstrap/gcc-4.0.ncl`
- `crunch build bootstrap/gcc-4.7.ncl`
- `crunch build bootstrap/gcc-10.ncl`
- `crunch build bootstrap/musl-full.ncl`
- `crunch build bootstrap/binutils-full.ncl`
- `crunch build bootstrap/seed-full.ncl`
- `crunch build bootstrap/selftest.ncl`
- `crunch build bootstrap/integration-test.ncl`
- `crunch self-build --no-substitute --source-root <manifest>` for the
  non-StageX source-root proof path.
- `crunch self-build --no-substitute --stagex-lineage <manifest> --no-host-tools --stage0-inventory <inventory>`
  for the StageX-class proof path added by this change.
- `./scripts/check-bootstrap-source-pins.rs bootstrap/*.ncl` for source and patch
  digest/provenance coverage.

A stage is incomplete if its log contains `ERROR: ... is a placeholder`, if the
matching task is deferred, if a referenced archive is only partial scaffolding,
or if provider selection reports the legacy musl.cc seed for a source-built or
StageX-class claim.

## Proof metadata and status-promotion mechanics

`crunch self-build` proof output is the binding source for final status. This
change must add the `self-build --stagex-lineage <manifest>` selection path,
thread `BootstrapProviderMode::StagexLineage` into `SelfBuildReport`, and reject
StageX completion unless `validate_stagex_proof_eligibility()` passes. The
protected execution audit source is the existing seccomp user-notification event
stream emitted as `self-build-proof: protected-seccomp-event=...`; the protected
transition source is `self-build-proof: protected-transition=bootstrap-tools-selected`
plus the crunch-built bwrap/busybox digests.

The proof bundle summary must expose provider kind, manifest digest, provider
output digest, proof bundle digest, and stage1/stage2 binary digests. StageX-class
proof also records audited seed digest, lineage manifest digest, stage graph
digest, normalized provider digest, staged source digest, bootstrap-tool digests,
protected execution audit digest when used, final proof bundle digest, and
canonical reproducibility report digest.

Bootstrap maturity docs/status may promote full-source or StageX-class status
only by reading those proof fields plus the stage transcript index. The docs
update must name remaining trusted roots separately from eliminated musl.cc
binary-provider trust. If any required field or transcript is missing, status
remains blocked and the report names the missing item.

## Risks / Trade-offs

**Long build chain** → Keep tasks stage-sized and preserve evidence per stage so
failed runs do not erase earlier proof.

**Accidental legacy fallback** → Tests and proof metadata must name provider
kind and reject legacy-provider proof for full-source status.

**Archive-status regression** → Bootstrap maturity reports must continue to
reject placeholder/deferred/archive evidence until this change records fresh
proof transcripts.
