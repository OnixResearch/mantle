# Bootstrap Specification

## Purpose

Defines crunch bootstrap requirements for source lineage, provider contracts,
self-build proof evidence, seed-chain replacement, and intermediate tool
derivations.
## Requirements
### Requirement: Full-source bootstrap root manifest

Crunch MUST define a versioned full-source bootstrap root manifest that names every source artifact, patch, digest, extraction rule, and expected provider output needed before the normalized seed contract is available.
ID: bootstrap.fullsource.root.manifest

The manifest MUST use BLAKE3 for crunch-owned artifact digests unless an upstream archive format or interoperability check requires another algorithm. Any non-BLAKE3 digest MUST name the reason. The manifest MUST fail validation if an artifact, patch, output, network trust root, or trust note is missing a digest or provenance field. Any remaining tiny seed or bootstrap assumption MUST be represented as a trust note with digest, provenance, scope, and rationale; undeclared non-source inputs MUST fail validation.

#### Scenario: Valid root manifest is accepted

- GIVEN a manifest with version, source artifacts, patches, BLAKE3 digests,
  extraction rules, declared trust notes, and expected provider outputs
- WHEN the manifest validator runs
- THEN validation succeeds
- AND the output lists the exact normalized seed contract outputs it expects

#### Scenario: Missing source digest is rejected

- GIVEN a manifest source artifact without a digest
- WHEN the manifest validator runs
- THEN validation fails
- AND the diagnostic names the artifact and the missing digest field

#### Scenario: Non-BLAKE3 digest requires a reason

- GIVEN a manifest source artifact with a SHA-256 digest
- WHEN the manifest omits the interoperability reason
- THEN validation fails
- AND the diagnostic says why crunch-owned manifests default to BLAKE3

#### Scenario: Undeclared patch is rejected

- GIVEN a source artifact references a patch name
- BUT the manifest patch list does not declare that patch with digest and provenance
- WHEN the manifest validator runs
- THEN validation fails
- AND the diagnostic names the undeclared patch

#### Scenario: Expected provider output metadata is required

- GIVEN a manifest expected provider output without digest or provenance
- WHEN the manifest validator runs
- THEN validation fails
- AND the diagnostic names the output and missing field

#### Scenario: Unexpected provider output is rejected

- GIVEN a source-built provider emits an output not listed by the manifest
- WHEN provider output validation runs
- THEN validation fails
- AND the diagnostic names the unexpected output

#### Scenario: Unmanifested network trust root is rejected

- GIVEN provider construction needs a network URL not present in manifest source artifacts or network trust roots
- WHEN provider dependency validation runs
- THEN validation fails before build execution
- AND the diagnostic names the unmanifested URL

#### Scenario: Trust note provenance is required

- GIVEN a remaining tiny seed or bootstrap assumption trust note
- WHEN the trust note omits digest, provenance, scope, or rationale
- THEN validation fails
- AND the diagnostic names the incomplete trust note

### Requirement: Source-built provider satisfies normalized seed contract

Crunch MUST support a source-built bootstrap provider that satisfies the existing normalized `bootstrap/seed.ncl` contract without deriving from the current musl.cc binary toolchain tarball.
ID: bootstrap.fullsource.provider.contract

The source-built provider MUST expose the same contract fields later bootstrap stages consume today: target-prefixed tool paths, headers, libraries, retained-tool metadata, reduction metadata, and provider notes. Later bootstrap derivations MUST keep depending on the normalized contract instead of provider-specific raw layouts. `crunch bootstrap --source-root <manifest>` MUST select the source-built provider; the existing `crunch bootstrap --fetch` path MUST remain the seed-assisted legacy provider; specifying both MUST fail before provider work starts.

#### Scenario: Source-built provider feeds make

- GIVEN a valid full-source root manifest and a source-built provider output
- WHEN `crunch build bootstrap/make.ncl` consumes that provider through
  `bootstrap/seed.ncl`
- THEN `make` builds successfully
- AND no later bootstrap derivation reads provider-specific raw paths

#### Scenario: Ambiguous provider selection fails closed

- GIVEN an operator passes both `--fetch` and `--source-root <manifest>`
- WHEN bootstrap provider selection runs
- THEN it exits non-zero before fetching or building provider inputs
- AND the diagnostic says the provider selection is ambiguous

#### Scenario: Legacy fetched provider remains explicitly labeled

- GIVEN the operator selects the existing fetched provider path
- WHEN bootstrap docs or proof reports describe the run
- THEN they label it as seed-assisted legacy provider evidence
- AND they do not call it full-source bootstrap evidence

### Requirement: Full-source bootstrap claim requires evidence

Crunch MUST withhold the full-source bootstrap claim until a source-root manifest validates, the source-built provider satisfies the normalized seed contract, and a self-build proof completes with that provider.
ID: bootstrap.fullsource.claim.evidence

The claim evidence MUST include the provider kind, manifest digest, provider output digest, proof bundle digest, and a docs update that separates remaining trusted roots from eliminated binary-provider trust. A prerequisite-only check, placeholder derivation, deferred task, or archived partial-scaffolding change MUST NOT count as full-source bootstrap evidence. Self-build proof metadata MUST bind those digests to the selected source-built provider so a legacy fetched-provider run cannot satisfy the full-source claim.

#### Scenario: Claim remains blocked before proof

- GIVEN the source-root manifest validates
- BUT no self-build proof has completed with the source-built provider
- WHEN docs or release evidence summarize bootstrap maturity
- THEN they keep the status below full-source bootstrap
- AND they name the missing proof evidence

#### Scenario: Proof records selected source-built provider

- GIVEN `crunch self-build --no-substitute --source-root <manifest>` completes
- WHEN proof metadata is written
- THEN it records provider kind `source-root`, manifest digest, provider output digest, and proof bundle digest
- AND those fields are included in the proof bundle summary

#### Scenario: Successful proof promotes the claim

- GIVEN a valid source-root manifest
- AND a source-built provider satisfying the normalized seed contract
- AND a full self-build proof bundle produced with that provider
- WHEN bootstrap maturity is reported
- THEN the report may state full-source bootstrap root evidence exists
- AND it includes the manifest, provider, and proof bundle digests

#### Scenario: Deferred live-bootstrap archive is not completion evidence

- GIVEN a live-bootstrap archive contains deferred validation or placeholder derivations
- WHEN an operator checks whether the full-source bootstrap chain is complete
- THEN the archive is treated as partial scaffolding only
- AND full-source bootstrap status remains blocked until real build proof exists

#### Scenario: Deferred successor is not completion evidence

- GIVEN unfinished live-bootstrap work has been moved to an active successor change
- AND that successor still has unchecked implementation or proof tasks
- WHEN an operator checks whether the full-source bootstrap chain is complete
- THEN the deferral is treated as work tracking only
- AND full-source bootstrap status remains blocked until the successor records fresh proof transcripts

### Requirement: StageX-class bootstrap lineage root

Crunch MUST define a StageX-class bootstrap lineage whose trusted bootstrap root is an auditable seed plus source artifacts, not a prebuilt compiler, prebuilt build tool, Nix store path, or musl.cc-derived binary provider.
ID: bootstrap.stagex.lineage.root

The lineage MUST name every source artifact, generated artifact, patch,
transition tool, seed byte sequence, digest, provenance note, and expected
normalized provider output from the audited seed to the first provider that
satisfies `bootstrap/seed.ncl`. An accepted audited seed MUST declare a
`seed_class` from the closed allowlist `hex0-seed`, its instruction set or
bytecode language, entry point, I/O contract, allowed syscall or host-interface
surface, checked-in human-readable source representation, source-to-byte
reproduction transcript, manual audit note, and `audit_seed_max_bytes` budget.
The `hex0-seed` class is limited to a hand-audited byte seed whose checked-in
hex0 source can rebuild the first hex0 compiler before any general compiler or
shell is trusted. The default `audit_seed_max_bytes` budget is 4096 bytes; any
larger seed or new seed class requires a separate OpenSpec change and ADR before
it can satisfy this profile. Crunch-owned fingerprints MUST use BLAKE3 unless an
upstream interoperability check requires another algorithm and records the
reason. Host kernel, CPU, firmware, container runtime, and filesystem behavior
MAY remain environmental assumptions, but they MUST be recorded outside the
source lineage so they do not masquerade as source-built inputs.

#### Scenario: Auditable seed lineage is accepted

- GIVEN a lineage manifest with seed bytes, stage0 source artifacts, patches,
  generated artifact rules, BLAKE3 digests, provenance, and expected normalized
  provider outputs
- WHEN lineage validation runs
- THEN validation succeeds
- AND the output lists the complete source-to-provider stage graph

#### Scenario: Seed without audit bounds is rejected

- GIVEN a lineage manifest whose seed omits `seed_class`, entry point, I/O
  contract, allowed host-interface surface, human-readable source, reproduction
  transcript, audit note, or `audit_seed_max_bytes`
- WHEN lineage validation runs for the StageX-class profile
- THEN validation fails
- AND the diagnostic names the missing audit-bound field

#### Scenario: Unsupported seed class is rejected

- GIVEN a lineage manifest whose `seed_class` is not `hex0-seed`
- WHEN lineage validation runs for the StageX-class profile
- THEN validation fails
- AND the diagnostic says new seed classes require separate OpenSpec and ADR
  approval

#### Scenario: Oversized seed needs separate approval

- GIVEN a lineage manifest whose seed byte length exceeds its declared
  `audit_seed_max_bytes` budget
- WHEN lineage validation runs for the StageX-class profile
- THEN validation fails
- AND the diagnostic says a larger seed requires separate OpenSpec and ADR
  approval

#### Scenario: Prebuilt compiler root is rejected

- GIVEN a lineage manifest that declares a host `cc`, `c++`, `make`, archive
  tool, Nix store path, or musl.cc binary tarball as a trusted bootstrap root
- WHEN lineage validation runs for the StageX-class profile
- THEN validation fails
- AND the diagnostic names the forbidden prebuilt root

#### Scenario: Environmental assumptions stay separate

- GIVEN a lineage manifest that records Linux kernel or CPU assumptions
- WHEN lineage validation runs
- THEN those assumptions are reported as environment assumptions
- AND they are not counted as source-built lineage nodes

### Requirement: Stage0 lineage produces normalized seed provider

Crunch MUST materialize the normalized seed provider through the declared StageX-class lineage before later bootstrap derivations consume `bootstrap/seed.ncl`.
ID: bootstrap.stagex.lineage.provider

The provider output MUST satisfy the existing normalized seed contract fields
used by later bootstrap derivations, including target-prefixed tools, headers,
libraries, provider metadata, retained-tool metadata, reduction metadata, and
provider notes. Later bootstrap derivations MUST consume only the normalized
contract, not stage0-posix, live-bootstrap, container, or provider-specific raw
layouts. A legacy fetched provider MUST remain available as seed-assisted
fallback evidence, but it MUST NOT satisfy this requirement.

#### Scenario: Lineage provider feeds existing bootstrap stages

- GIVEN a valid StageX-class lineage manifest
- AND the lineage materializes a normalized provider output
- WHEN `crunch build bootstrap/make.ncl` consumes the provider through
  `bootstrap/seed.ncl`
- THEN `make` builds successfully
- AND no later bootstrap derivation reads provider-specific raw paths

#### Scenario: Legacy provider cannot satisfy lineage provider requirement

- GIVEN a self-build or bootstrap run selected the legacy fetched provider
- WHEN StageX-class provider evidence is requested
- THEN the run is classified as seed-assisted only
- AND the StageX-class provider requirement remains unsatisfied

#### Scenario: Raw layout coupling is rejected

- GIVEN a later bootstrap derivation reads a live-bootstrap, stage0-posix, OCI,
  or temporary provider raw path directly
- WHEN provider-boundary validation runs
- THEN validation fails
- AND the diagnostic names the derivation and forbidden raw path

### Requirement: StageX-class self-build proof binds lineage evidence

Crunch MUST require a full self-build proof with the StageX-class lineage provider before reporting StageX-class bootstrap evidence.
ID: bootstrap.stagex.selfbuild.proof

The proof metadata MUST bind the audited seed digest, lineage manifest digest,
stage graph digest, normalized provider digest, staged source digest, stage1 and
stage2 crunch binary digests, bootstrap-tool digests, protected execution audit
digest when used, and final proof bundle digest. The release profile MUST bind
that proof bundle digest to the canonical reproducibility report digest before
any StageX-class verified-build claim is emitted. The proof MUST fail closed
when the lineage provider is missing, legacy-fetched, unvalidated, or when
forbidden host executables run during the protected stage. A prerequisite-only
check MUST NOT count as StageX-class proof evidence.

#### Scenario: Successful proof records lineage evidence

- GIVEN a validated StageX-class lineage provider
- AND a full self-build proof completes with that provider
- WHEN proof metadata is written
- THEN it records the audited seed digest, lineage manifest digest, stage graph
  digest, provider digest, stage1 digest, stage2 digest, and proof bundle digest
- AND it records that the provider kind is StageX-class lineage

#### Scenario: Host executable escape blocks the proof

- GIVEN the protected stage observes an undeclared host compiler, build tool,
  archive tool, shell, Nix command, or legacy provider executable
- WHEN the proof run evaluates StageX-class evidence
- THEN the proof fails closed
- AND no StageX-class claim is emitted

#### Scenario: Prerequisite-only check stays insufficient

- GIVEN `./scripts/prove-self-hosting.sh --check` succeeds
- WHEN bootstrap maturity is reported
- THEN StageX-class proof evidence remains absent
- AND the report names the missing full proof run

The following seed-chain requirements define the live-bootstrap-derived
replacement path from hex0 to a modern GCC, musl, and binutils toolchain.

### Requirement: Hex0 seed as trust root

The bootstrap chain MUST start from a hex0 seed binary of at most 512 bytes
for the target architecture, checked into the repository under `bootstrap/seeds/`.
The seed binary MUST be the exact output of assembling the stage0-posix hex0
source for AMD64.

#### Scenario: Seed is present and correctly sized

- GIVEN the repository checkout
- WHEN `bootstrap/seeds/AMD64/hex0-seed` is read
- THEN it is at most 512 bytes and matches the stage0-posix hex0 AMD64 binary

### Requirement: Stage0-posix as crunch derivation

The stage0-posix bootstrap (phases 0-28) MUST run as a single crunch derivation
that takes only the hex0 seed and the stage0-posix source tarball as inputs.
The derivation MUST produce mescc-tools (M1, hex2, kaem, blood-elf), M2-Planet,
and mescc-tools-extra (catm, cp, chmod, mkdir, untar, ungz, unbz2, unxz,
sha256sum).

#### Scenario: Stage0-posix builds from hex0

- GIVEN the hex0 seed and stage0-posix source pinned by hash
- WHEN `crunch build bootstrap/stage0-posix.ncl` runs
- THEN the output contains working M2-Planet, hex2, M1, and kaem binaries

### Requirement: GNU mes from stage0-posix output

GNU mes MUST be built as a crunch derivation using only M2-Planet and
mescc-tools from the stage0-posix output. The mes output MUST include both
the Scheme interpreter and the mes C compiler (`mescc`).

#### Scenario: Mes compiles a C program

- GIVEN the stage0-posix output
- WHEN `crunch build bootstrap/mes.ncl` runs
- THEN mes can compile a trivial C program to a working executable

### Requirement: Tinycc from mes

Tinycc 0.9.26 MUST be built using the mes C compiler. Tinycc 0.9.27 MUST
then be built using tinycc 0.9.26 (self-hosting). The final tinycc output
MUST be capable of building early GCC.

#### Scenario: Tinycc self-hosts

- GIVEN the mes output
- WHEN `crunch build bootstrap/tinycc.ncl` runs
- THEN tinycc 0.9.27 can compile C programs including early GCC prerequisites

### Requirement: GCC version ladder

GCC MUST be built through a version ladder where each version is compiled
by the previous. The minimum chain MUST include: tinycc → gcc-4.0.4 →
gcc-4.7.4 → gcc-10.x (or newer). Supporting tools (make, binutils, musl)
MUST be built at each level as needed by the next GCC version.

#### Scenario: Each GCC version builds from the previous

- GIVEN the tinycc output
- WHEN the GCC chain derivations are built in sequence
- THEN each GCC version produces a working C/C++ compiler

#### Scenario: Modern GCC can build the existing bootstrap chain

- GIVEN the final GCC from the version ladder
- WHEN `bootstrap/selftest.ncl` and `bootstrap/integration-test.ncl` are built
- THEN both tests pass using the from-source toolchain

### Requirement: Normalized seed contract preserved

The final output of the full-source chain MUST expose the same normalized seed
contract as the current `bootstrap/seed.ncl`: target-prefixed binutils in
`bin/`, headers at `<target>/include`, `libgcc_s.so*` under `<target>/lib/`,
and provider metadata in `share/crunch-bootstrap/provider.json`.

#### Scenario: Downstream chain unchanged

- GIVEN the from-source seed output
- WHEN `bootstrap/make.ncl` through `bootstrap/crunch.ncl` are built
- THEN all derivations succeed using the new seed with no modifications

#### Scenario: Self-build produces identical binary

- GIVEN the from-source seed
- WHEN `crunch self-build` runs
- THEN stage1 and stage2 binaries are byte-identical

### Requirement: Source tarballs pinned by hash

Every source tarball fetched during the bootstrap chain MUST be pinned by a
content hash in its `.ncl` file. The hash MUST use the same algorithm as
`crunch.fetchTarball` (currently SHA-256 for Nix compatibility).

#### Scenario: Tampered source detected

- GIVEN a stage's source tarball with a modified byte
- WHEN the derivation is built
- THEN the build fails with a hash mismatch error

### Requirement: Legacy seed as development fast-path

The musl.cc-based seed MUST remain available as an opt-in development fast-path
while the full-source chain is being built out. Once the full chain passes all
validation (selftest, integration-test, self-build), the legacy seed MAY be
removed.
ID: bootstrap.legacy.seed.fastpath

#### Scenario: Developer uses legacy seed

- GIVEN a developer who does not want to wait for the full chain
- WHEN they build with the legacy seed option
- THEN the existing musl.cc tarball path is used and all downstream builds work

#### Scenario: Legacy seed selector is concrete

- GIVEN the full-source seed chain still contains placeholder derivations
- WHEN `bootstrap/seed.ncl` selects the legacy seed path
- THEN `bootstrap/seed-legacy.ncl` provides the concrete reduced musl.cc provider
- AND the legacy path does not import `seed-legacy.ncl` recursively

### Requirement: Intermediate tools build from tcc only

Each intermediate tool derivation MUST use only tinycc and previously built
intermediate tools as compilers and build tools.  No host compiler, host libc,
or host shell tools may appear in the sandbox.

#### Scenario: Tool builds without host leakage

- GIVEN a tcc-compiled make and tcc
- WHEN an intermediate tool derivation is built
- THEN the build succeeds using only chain-internal tools

### Requirement: musl-1.1.24 as libc boundary

musl-1.1.24 MUST be built using tcc.  All tools built after musl MUST link
against musl rather than mes libc.

#### Scenario: Post-musl tool links against musl

- GIVEN musl-1.1.24 built by tcc
- WHEN m4, flex, bison, or grep is built
- THEN the resulting binary links against musl libc

### Requirement: binutils-2.30 functional

binutils-2.30 MUST produce working `as`, `ld`, and `ar` binaries that
gcc-4.0.4 can use as its assembler and linker.

#### Scenario: Assembler produces object files

- GIVEN binutils-2.30 built from the intermediate chain
- WHEN `as` assembles a trivial `.s` file
- THEN a valid ELF object file is produced

### Requirement: Live-bootstrap part stage0-posix seed tools is independently tracked
Crunch MUST track the live-bootstrap part `bootstrap-seeds through mescc-tools-extra` as an independent bootstrap change bound to `bootstrap/stage0-posix.ncl`.
ID: bootstrap.part.stage0.posix

The part scope MUST include only the source pins, patches, derivation logic, output contract, and evidence needed for `bootstrap/stage0-posix.ncl` and direct predecessor compatibility. Broader chain validation MAY depend on this part, but MUST NOT replace this part's own build and smoke evidence.

#### Scenario: Part evidence is isolated

- GIVEN implementation work touches `bootstrap/stage0-posix.ncl`
- WHEN the part is marked complete
- THEN the change records source-pin audit evidence for `bootstrap/stage0-posix.ncl`
- AND it records a successful `crunch build bootstrap/stage0-posix.ncl` transcript
- AND it records a smoke check for the produced output contract

#### Scenario: Downstream blockers stay local

- GIVEN a downstream bootstrap stage fails after `bootstrap/stage0-posix.ncl` succeeds
- WHEN this part's evidence is reviewed
- THEN the downstream failure does not invalidate this part's completed evidence
- AND a separate part change tracks the downstream failure

### Requirement: Live-bootstrap part mes 0.27 is independently tracked
Crunch MUST track the live-bootstrap part `mes 0.27` as an independent bootstrap change bound to `bootstrap/mes.ncl`.
ID: bootstrap.part.mes.0.27

The part scope MUST include only the source pins, patches, derivation logic, output contract, and evidence needed for `bootstrap/mes.ncl` and direct predecessor compatibility. Broader chain validation MAY depend on this part, but MUST NOT replace this part's own build and smoke evidence.

#### Scenario: Part evidence is isolated

- GIVEN implementation work touches `bootstrap/mes.ncl`
- WHEN the part is marked complete
- THEN the change records source-pin audit evidence for `bootstrap/mes.ncl`
- AND it records a successful `crunch build bootstrap/mes.ncl` transcript
- AND it records a smoke check for the produced output contract

#### Scenario: Downstream blockers stay local

- GIVEN a downstream bootstrap stage fails after `bootstrap/mes.ncl` succeeds
- WHEN this part's evidence is reviewed
- THEN the downstream failure does not invalidate this part's completed evidence
- AND a separate part change tracks the downstream failure

### Requirement: Tinycc Mes self-compile blocker is resolved
Crunch MUST resolve the `BufferedFile`/first-self-compile blocker before `live-part-tinycc-0-9-26` can claim successful build evidence.
ID: bootstrap.part.tinycc.0.9.26.selfcompile

#### Scenario: Full tinycc output contract is required evidence

- GIVEN `bootstrap/tinycc-mes.ncl` builds `tcc-mes`
- WHEN the blocker is marked resolved
- THEN the evidence MUST show `tcc-mes` compiles at least `tcc-boot0` without segfaulting
- AND `crunch build bootstrap/tinycc-mes.ncl` MUST finish successfully
- AND the produced output MUST include executable `bin/tcc` and `bin/tcc-0.9.26`
- AND the produced compiler MUST compile a trivial C program
- AND `tcc-mes -version` alone MUST NOT be accepted as the smoke boundary

### Requirement: Live-bootstrap part tinycc 0.9.26 is independently tracked
Crunch MUST track the live-bootstrap part `tinycc 0.9.26` as an independent bootstrap change bound to `bootstrap/tinycc-mes.ncl`.
ID: bootstrap.part.tinycc.0.9.26

The part scope MUST include only the source pins, patches, derivation logic, output contract, and evidence needed for `bootstrap/tinycc-mes.ncl` and direct predecessor compatibility. Broader chain validation MAY depend on this part, but MUST NOT replace this part's own build and smoke evidence.

#### Scenario: Part evidence is isolated

- GIVEN implementation work touches `bootstrap/tinycc-mes.ncl`
- WHEN the part is marked complete
- THEN the change records source-pin audit evidence for `bootstrap/tinycc-mes.ncl`
- AND it records a successful `crunch build bootstrap/tinycc-mes.ncl` transcript
- AND it records a smoke check for the produced output contract

#### Scenario: Downstream blockers stay local

- GIVEN a downstream bootstrap stage fails after `bootstrap/tinycc-mes.ncl` succeeds
- WHEN this part's evidence is reviewed
- THEN the downstream failure does not invalidate this part's completed evidence
- AND a separate part change tracks the downstream failure

### Requirement: Live-bootstrap part tinycc 0.9.27 is independently tracked
Crunch MUST track the live-bootstrap part `tinycc 0.9.27` as an independent bootstrap change bound to `bootstrap/tinycc.ncl`.
ID: bootstrap.part.tinycc.0.9.27

The part scope MUST include only the source pins, patches, derivation logic, output contract, and evidence needed for `bootstrap/tinycc.ncl` and direct predecessor compatibility. Broader chain validation MAY depend on this part, but MUST NOT replace this part's own build and smoke evidence.

#### Scenario: Part evidence is isolated

- GIVEN implementation work touches `bootstrap/tinycc.ncl`
- WHEN the part is marked complete
- THEN the change records source-pin audit evidence for `bootstrap/tinycc.ncl`
- AND it records a successful `crunch build bootstrap/tinycc.ncl` transcript
- AND it records a smoke check for the produced output contract

#### Scenario: Downstream blockers stay local

- GIVEN a downstream bootstrap stage fails after `bootstrap/tinycc.ncl` succeeds
- WHEN this part's evidence is reviewed
- THEN the downstream failure does not invalidate this part's completed evidence
- AND a separate part change tracks the downstream failure

### Requirement: TinyCC 0.9.27 compiles trivial C on amd64
Crunch MUST build `bootstrap/tinycc.ncl` into a TinyCC 0.9.27 output that can compile a trivial C source file to an object on amd64.
ID: bootstrap.part.tinycc.0.9.27.amd64.compile

The output MUST include `bin/tcc`, report `tcc version 0.9.27 (x86_64 Linux)`, complete `tcc -c hello.c -o hello.o` within the bounded smoke timeout, and produce a non-empty object file. The compiler MUST reject malformed C with a controlled nonzero exit rather than a hang, timeout, or segmentation fault. Completion evidence MUST include a source-pin audit transcript, successful `crunch build bootstrap/tinycc.ncl` transcript, version smoke transcript, positive object-compile transcript, malformed-input negative transcript, and host-leakage scan transcript.

#### Scenario: Trivial object compile succeeds

- GIVEN `bootstrap/tinycc.ncl` has been built with the documented bootstrap environment
- WHEN the produced `bin/tcc -c hello.c -o hello.o` runs on `int main(){return 0;}`
- THEN the command exits successfully
- AND `hello.o` exists and is non-empty

#### Scenario: Malformed input fails cleanly

- GIVEN the same produced compiler
- WHEN it compiles malformed C under a bounded timeout
- THEN it exits nonzero
- AND the exit status is not timeout-derived
- AND the exit status is not signal-derived

### Requirement: Mes-built TinyCC emits x86_64 immediate shifts correctly
Crunch MUST build the Mes-hosted TinyCC 0.9.26 predecessor so x86_64 constant shift expressions emit nonzero immediate shift counts.
ID: bootstrap.part.tinycc.0.9.26.shift-immediates

The predecessor compiler MUST compile a shift reproducer containing `x >> 8` and `x << 3` to an object whose disassembly contains `shr $0x8` and `shl $0x3` (or equivalent nonzero immediate encodings). The rebuilt TinyCC 0.9.27 MUST then compile both a trivial C source and GNU make 3.82 `getopt.c` to non-empty objects, and malformed C MUST fail with a controlled nonzero status rather than a timeout or signal. Evidence MUST include source-pin audit, build transcript, shift disassembly transcript, object compile transcripts, malformed-input transcript, and host-leakage scan.

#### Scenario: Shift reproducer emits nonzero immediates

- GIVEN the repaired `bootstrap/tinycc-mes.ncl` output
- WHEN it compiles `unsigned f(unsigned x){ return x >> 8; }` and `unsigned g(unsigned x){ return x << 3; }`
- THEN object disassembly contains nonzero immediate shift counts for both functions

#### Scenario: TinyCC 0.9.27 compiles make getopt

- GIVEN `bootstrap/tinycc.ncl` has been rebuilt from the repaired predecessor
- WHEN the compiler compiles GNU make 3.82 `getopt.c`
- THEN the command exits successfully
- AND the produced object is non-empty

<!-- synced from openspec change: live-bootstrap-binutils-tcc-chain -->
## ADDED Requirements

### Requirement: Binutils-TCC chain implementation

Crunch MUST build `bootstrap/binutils-tcc.ncl` from chain-internal TinyCC-era and post-musl derivations without using host compiler, host libc, host shell tools, or the legacy musl.cc provider.
ID: bootstrap.binutils.tcc.chain

The chain MUST implement the scoped ladder groups named by the proposal: early tcc-hosted utilities (`bzip2`, `coreutils-5.0`, `oyacc`, `bash-2.05b`), first musl/tcc rebuilds, post-musl text/parser tools (`grep`, rebuilt `sed`, rebuilt `bzip2`, `m4`, Heirloom devtools, `flex`, `bison`), diffutils/coreutils/gawk, Perl/autoconf/automake/libtool, and binutils 2.30. The chain MUST pin every source, carried patch, and generated artifact with URL or repository path, digest, and provenance at the first consuming derivation. Validation transcripts MUST record command, provider selection, exit status, output path, fallback status/event marker, and placeholder rejection result. Validation MUST prove post-musl `m4`, `flex`, `bison`, and `grep` link against musl, and MUST prove binutils 2.30 can assemble a trivial ELF object for the gcc-4.0.4 transition.

#### Scenario: Placeholder is replaced

- GIVEN `bootstrap/binutils-tcc.ncl` is evaluated
- WHEN the derivation builds
- THEN it does not emit `ERROR: binutils-tcc.ncl is a placeholder`
- AND it produces working `as`, `ld`, `ar`, `ranlib`, `nm`, and `objcopy` tools

#### Scenario: Host leakage is rejected

- GIVEN any intermediate tool derivation invokes a host compiler, host libc, host shell tool, Nix command, or legacy provider executable
- WHEN validation audits the transcript
- THEN the chain is marked incomplete
- AND full-source bootstrap status remains blocked

#### Scenario: Source provenance is complete

- GIVEN a new source, patch, or generated artifact is consumed by the binutils-TCC chain
- WHEN the source-pin audit runs
- THEN it records URL or repository path, digest, provenance, and first consuming derivation

<!-- synced from openspec change: live-bootstrap-gcc-4-0-stage -->
## ADDED Requirements

### Requirement: GCC 4.0.4 transition stage

Crunch MUST build `bootstrap/gcc-4.0.ncl` as gcc 4.0.4 C and C++ compiler outputs using only the chain-internal TinyCC/musl/binutils 2.30 inputs.
ID: bootstrap.gcc40.transition

The stage MUST pin gcc 4.0.4 C/C++ source inputs and every carried patch or generated artifact with URL/path, digest, provenance, and first-consuming derivation metadata. Validation MUST prove the output compiles C and C++ smoke programs and MUST reject host compiler, host libc, host shell, Nix, or legacy-provider fallback by scanning sandbox command transcripts and proof markers.

#### Scenario: GCC 4.0.4 placeholder is replaced

- GIVEN `bootstrap/gcc-4.0.ncl` is built
- WHEN the build completes
- THEN it does not emit `ERROR: gcc-4.0.ncl is a placeholder`
- AND it produces working C and C++ compiler binaries

#### Scenario: GCC 4.0.4 rejects host leakage

- GIVEN the stage transcript contains host compiler, host libc, host shell, Nix, or legacy provider execution
- WHEN validation runs
- THEN the stage is marked incomplete
- AND parent full-source bootstrap status remains blocked

<!-- synced from openspec change: live-bootstrap-gcc-4-7-stage -->
## ADDED Requirements

### Requirement: GCC 4.7.4 transition stage

Crunch MUST build `bootstrap/gcc-4.7.ncl` as gcc 4.7.4 C and C++ compiler outputs using only chain-internal gcc-4.0.4-era inputs.
ID: bootstrap.gcc47.transition

The stage MUST pin gcc 4.7.4 and support artifacts with URL/path, digest, provenance, and first-consuming derivation metadata. Validation MUST prove the output compiles C, C++, and minimal C++11 smoke programs, and MUST reject host compiler, host libc, host shell, Nix, or legacy-provider fallback by scanning sandbox command transcripts and proof markers.

#### Scenario: GCC 4.7.4 placeholder is replaced

- GIVEN `bootstrap/gcc-4.7.ncl` is built
- WHEN the build completes
- THEN it does not emit `ERROR: gcc-4.7.ncl is a placeholder`
- AND it produces working C and C++ compiler binaries

#### Scenario: C++11 transition is usable

- GIVEN gcc 4.7.4 output exists
- WHEN validation compiles a minimal C++11 source
- THEN the compile succeeds using only chain-internal inputs
