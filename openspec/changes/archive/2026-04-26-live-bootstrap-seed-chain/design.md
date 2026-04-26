# Design: Live-bootstrap seed chain

## Context

The crunch bootstrap chain currently has one opaque trust root: a ~50 MB
prebuilt GCC+musl+binutils tarball from musl.cc fetched by `bootstrap/seed.ncl`.
Everything after the seed is built from source. The bootstrappable.org ecosystem
(stage0-posix, live-bootstrap, StagEx) has proven a full path from a 256-byte
hex0 seed to modern GCC. We need to translate those proven build scripts into
Nickel derivations.

The existing chain: `seed.ncl → make.ncl → dash.ncl → binutils.ncl → musl.ncl
→ gcc.ncl → {busybox, bwrap}.ncl + rust.ncl → crunch.ncl`

The seed replacement chain: `hex0 → stage0-posix → mes → tinycc → gcc-4.0 →
gcc-4.7 → gcc-10 → normalized-seed → [existing chain continues]`

## Goals / Non-Goals

**Goals:** replace the musl.cc binary blob with a from-source chain; reuse
proven build scripts from live-bootstrap/stage0-posix; keep the normalized seed
contract stable so downstream derivations don't change.

**Non-Goals:** bootstrapping Rust from source (mrustc); multi-arch support;
switching to LLVM; formal seed verification; performance optimization of
the full chain build time.

## Decisions

### 1. Adapt live-bootstrap scripts, don't reimplement

**Choice:** take build scripts from live-bootstrap's `steps/` directory and
stage0-posix's `kaem.run`, adapt them into crunch derivation build phases.

**Rationale:** these scripts are battle-tested across multiple distros.
Reimplementing from scratch would be months of work and introduce new bugs.
StagEx and Guix both build on the same upstream scripts.

**Why not Guix:** Guix's bootstrap is tightly coupled to Guile scheme and Guix's
own derivation model. Extracting just the build logic means reverse-engineering
`(package ...)` forms. live-bootstrap's scripts are plain shell.

**Why not StagEx directly:** StagEx packages stages as OCI images with a
Makefile+Dockerfile build system. Crunch needs Nickel derivations. The
underlying build scripts are the same as live-bootstrap's -- StagEx is a
packaging layer on top.

### 2. Stage0-posix as a single derivation

**Choice:** run all 28 stage0-posix phases as one crunch derivation.

**Rationale:** stage0-posix is designed to run as a single `kaem.run` script.
The phases are tightly coupled (each builds on the previous in the same
directory). Splitting into 28 separate derivations would require artificial
I/O boundaries where none exist. One derivation, one kaem.run, one output
containing all mescc-tools.

**Implementation:** the derivation mounts the hex0 seed, fetches stage0-posix
source, and runs `kaem.run` in the sandbox. Build uses the hex0 seed as the
initial binary -- no host shell or compiler needed inside the sandbox.

### 3. GCC version ladder follows live-bootstrap's proven path

**Choice:** tinycc → gcc-4.0.4 → gcc-4.7.4 → gcc-10.5.0 (or newest in
live-bootstrap).

**Rationale:** this is the exact path live-bootstrap uses. gcc-4.0.4 is the
oldest GCC that tinycc can build. gcc-4.7.4 adds C++11 support needed by
modern GCC. Skipping versions risks hitting compiler bugs that live-bootstrap
already worked around.

**Alternative rejected:** going directly from tinycc to modern GCC. Doesn't
work -- modern GCC requires C++11, which requires gcc-4.7+. The version ladder
is unavoidable.

### 4. Supporting tools built at each level as needed

**Choice:** make, sed, patch, tar, gawk, etc. are built as separate derivations
at the level they're first needed, following live-bootstrap's dependency graph.

**Rationale:** each GCC version has different build requirements. gcc-4.0 needs
make and basic coreutils. gcc-4.7 needs additional autotools. Modern GCC needs
even more. Building each tool at the right level matches what live-bootstrap
already does.

**Implementation:** each supporting tool is a `bootstrap/<tool>-<version>.ncl`
file. The dependency chain is explicit in Nickel imports.

### 5. Normalized seed contract as adapter

**Choice:** a new `bootstrap/seed.ncl` takes the final GCC+musl+binutils output
and normalizes it into the same contract the rest of the chain expects.

**Rationale:** downstream derivations (`make.ncl` through `crunch.ncl`) import
`seed.ncl` and expect a specific layout. The adapter pattern means the
downstream chain doesn't change at all. The adapter does the mapping between
live-bootstrap's output layout and crunch's expected seed interface.

### 6. Legacy seed preserved as opt-in fast path

**Choice:** rename current `seed.ncl` to `seed-legacy.ncl`. New `seed.ncl`
defaults to the full-source chain. A `--legacy-seed` flag or env var switches
to the fast path for development.

**Rationale:** the full chain will take much longer to build than a tarball
fetch. Developers iterating on non-bootstrap code shouldn't have to wait.
Once the full chain is proven equivalent, the legacy path can be removed.

## Risks / Trade-offs

**[Build time]** Full hex0-to-gcc chain will take significantly longer than
fetching a 50 MB tarball. Mitigation: cache intermediate derivation outputs;
keep legacy fast path.

**[Sandbox constraints]** stage0-posix's kaem.run expects to run phases
sequentially in the same directory, writing intermediate binaries alongside
source. The bwrap sandbox must allow this pattern (it does -- each derivation
gets a writable scratch directory).

**[Version drift]** live-bootstrap updates their scripts over time. Pinning
source tarballs by hash freezes our chain at a point in time. Mitigation:
document which live-bootstrap commit each derivation was adapted from;
updating is a future change.

**[Intermediate tool sprawl]** the chain needs ~20 derivations for tools
that only exist to build the next tool. Mitigation: these are honest
dependencies of a source-only bootstrap; there's no shortcut. The derivation
graph makes the chain auditable.
