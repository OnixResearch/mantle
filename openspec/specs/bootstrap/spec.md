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

Crunch MUST withhold the full-source bootstrap claim until the source-root manifest validates for the full-source profile, the lineage manifest validates for the StageX-class profile, every named live-bootstrap placeholder is replaced, source-built stage transcripts exist, and self-build proof completes with the selected source-built provider.
ID: bootstrap.fullsource.claim.evidence

The claim evidence MUST include provider kind (`source-root` for the full-source profile or StageX-class lineage serialized as `stagex-lineage` for the StageX-class profile), manifest digest, provider output digest, proof bundle digest, stage-by-stage build transcripts through `bootstrap/seed-full.ncl`, `bootstrap/selftest.ncl` and `bootstrap/integration-test.ncl` transcripts, proof metadata bound to the selected source-built provider, explicit `self-build-proof: fallback-event=<kind>` markers or `self-build-proof: fallback-event=none`, and a docs update that separates remaining trusted roots from eliminated binary-provider trust. A prerequisite-only check, placeholder derivation, deferred task, archived partial-scaffolding change, unfinished successor task, legacy-provider fallback, host-bwrap fallback, or checkout/source-discovery fallback MUST NOT count as full-source bootstrap evidence.

#### Scenario: Placeholder blocks full-source claim

- GIVEN any of `bootstrap/binutils-tcc.ncl`, `bootstrap/gcc-4.0.ncl`, `bootstrap/gcc-4.7.ncl`, `bootstrap/gcc-10.ncl`, `bootstrap/musl-full.ncl`, `bootstrap/binutils-full.ncl`, or `bootstrap/seed-full.ncl` still emits placeholder text
- WHEN bootstrap maturity is reported
- THEN full-source bootstrap evidence remains absent
- AND the report names the unresolved placeholder stage

#### Scenario: Final source proof records docs separation

- GIVEN all stage transcripts and self-build proof complete with the source-built provider
- WHEN bootstrap maturity docs are updated
- THEN the docs name remaining trust roots and eliminated binary-provider trust separately
- AND the proof bundle digest is recorded next to provider kind, manifest digest, and provider output digest

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

Crunch MUST require the full StageX-class evidence tuple before this live-bootstrap source-chain change can satisfy any StageX-class bootstrap claim.
ID: bootstrap.stagex.selfbuild.proof

The proof metadata MUST bind audited seed digest, lineage manifest digest, stage graph digest, normalized provider digest, staged source digest, stage1 and stage2 crunch binary digests, bootstrap-tool digests, protected execution audit digest when used, final proof bundle digest, canonical reproducibility report digest, and explicit `self-build-proof: fallback-event=<kind>` markers or `self-build-proof: fallback-event=none`. The proof MUST fail closed when any named live-bootstrap placeholder remains, when the legacy provider is selected, when host-bwrap or checkout/source-discovery fallback appears, or when forbidden host executables run during the protected stage.

#### Scenario: StageX proof records complete evidence tuple

- GIVEN the live-bootstrap source chain materializes a normalized provider
- AND a full protected self-build proof completes with that provider
- WHEN StageX-class proof metadata is written
- THEN it records audited seed digest, lineage manifest digest, stage graph digest, normalized provider digest, staged source digest, stage1 digest, stage2 digest, bootstrap-tool digests, protected execution audit digest when used, final proof bundle digest, canonical reproducibility report digest, and explicit `self-build-proof: fallback-event=<kind>` markers or `self-build-proof: fallback-event=none`
- AND it records provider kind as StageX-class lineage serialized as `stagex-lineage`

#### Scenario: StageX proof rejects legacy fallback

- GIVEN any stage selected the legacy musl.cc provider
- WHEN StageX-class evidence is requested
- THEN the proof fails closed
- AND no StageX-class claim is emitted

<!-- synced from openspec change: live-part-grep-2-4 -->
<!-- ADDED Requirements -->

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

<!-- synced from openspec change: repair-tinycc-0-9-27-amd64-link -->

### Requirement: TinyCC 0.9.27 amd64 static link succeeds [r[bootstrap.compiler.tinycc.0.9.27.amd64.static-link]]
The amd64 bootstrap TinyCC 0.9.27 output MUST support statically linking a trivial object with its declared Mes runtime inputs.

#### Scenario: Trivial static executable links and runs [r[bootstrap.compiler.tinycc.0.9.27.amd64.static-link.scenario.hello]]
- GIVEN the Crunch-built `bootstrap/tinycc.ncl` output on amd64
- WHEN `tcc -c hello.c` and `tcc -static -o hello hello.o` run in a Crunch diagnostic derivation
- THEN the link exits successfully
- AND executing `./hello` exits 0
- AND no host compiler, host libc, or undeclared runtime object is used

### Requirement: Make validation resumes after TinyCC link repair [r[bootstrap.compiler.tinycc.0.9.27.amd64.static-link.make-handoff]]
The TinyCC link repair MUST hand control back to the Make 3.82 runtime-validation successor after the compiler link smoke passes.

#### Scenario: Make runtime validation is unblocked or reclassified [r[bootstrap.compiler.tinycc.0.9.27.amd64.static-link.make-handoff.scenario.resume]]
- GIVEN the TinyCC link smoke passes
- WHEN `bootstrap/make-tcc.ncl` runtime validation is rerun
- THEN either the Make output path is produced for smoke testing
- OR the next concrete Make-specific blocker is recorded with transcript evidence

<!-- synced from openspec change: live-bootstrap-binutils-tcc-chain -->

<!-- synced from openspec change: spike-i386-live-bootstrap-path -->

### Requirement: i386 live-bootstrap path spike [r[bootstrap.i386-live-bootstrap-spike]]

Crunch MUST provide decision evidence before pivoting Make 3.82 runtime validation from the current amd64 TinyCC/Mes repair path to an i386-first live-bootstrap path.

#### Scenario: Reference audit is not runtime proof [r[bootstrap.i386-live-bootstrap-spike.reference-audit]]

- GIVEN a StageX or upstream live-bootstrap reference sequence that builds on `linux/386`
- WHEN Crunch records that sequence as evidence
- THEN Crunch MUST classify it as reference evidence only until a Crunch-local proof target runs.

#### Scenario: Pivot decision has explicit criteria [r[bootstrap.i386-live-bootstrap-spike.decision]]

- GIVEN the amd64 Make 3.82 path remains blocked by runtime segfaults
- WHEN the i386 proof target is evaluated
- THEN Crunch MUST record whether to pivot, continue amd64 repair, or carry both paths with explicit scope boundaries.

<!-- synced from openspec change: repair-i386-tinycc26-emission -->

### Requirement: i386 TinyCC 0.9.26 emission diagnostics [r[bootstrap.i386-tinycc26-emission.diagnostics]]

Crunch MUST isolate the i386 TinyCC 0.9.26 output-generation blocker before using the i386 path as evidence for Make 3.82 runtime validation.

#### Scenario: Emission stages are separated [r[bootstrap.i386-tinycc26-emission.diagnostics.stages]]

- GIVEN a x86_64-hosted/i386-targeting TinyCC 0.9.26 built by Crunch
- WHEN Crunch evaluates the i386 emission proof
- THEN it MUST record version, assemble-only, link-from-assembly, link-from-object, and run-output results separately.

#### Scenario: Diagnostic failure does not imply production pivot [r[bootstrap.i386-tinycc26-emission.diagnostics.no-production-pivot]]

- GIVEN any i386 emission stage fails or segfaults
- WHEN the diagnostic evidence is recorded
- THEN Crunch MUST keep production Make 3.82 validation blocked rather than claiming the i386 path is production-ready.

### Requirement: i386 TinyCC 0.9.26 repair decision [r[bootstrap.i386-tinycc26-emission.decision]]

Crunch MUST record the next repair target after the diagnostic stage identifies where `tcc26-i386` fails.

#### Scenario: Next repair target is evidence-backed [r[bootstrap.i386-tinycc26-emission.decision.target]]

- GIVEN diagnostic transcript evidence for each emission stage
- WHEN choosing the next implementation slice
- THEN Crunch MUST identify whether the next target is assembly parsing, object emission, static linking, ELF materialization, or runtime execution.

<!-- ADDED Requirements -->

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
<!-- ADDED Requirements -->

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
<!-- ADDED Requirements -->

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

<!-- synced from openspec change: live-bootstrap-source-chain -->
<!-- ADDED Requirements -->

### Requirement: Source-built bootstrap chain implementation

Crunch MUST implement the live-bootstrap stage chain through a source-built provider that satisfies the normalized seed contract without using the legacy musl.cc binary provider.
ID: bootstrap.source.chain.implementation

The chain MUST replace `bootstrap/binutils-tcc.ncl`, `bootstrap/gcc-4.0.ncl`, `bootstrap/gcc-4.7.ncl`, `bootstrap/gcc-10.ncl`, `bootstrap/musl-full.ncl`, `bootstrap/binutils-full.ncl`, and `bootstrap/seed-full.ncl` placeholder derivations before they can satisfy bootstrap completion status. Stage validation MUST record command transcripts for the ordered inventory (`bootstrap/stage0-posix.ncl`, `bootstrap/mes.ncl`, `bootstrap/tinycc.ncl`, `bootstrap/binutils-tcc.ncl`, `bootstrap/gcc-4.0.ncl`, `bootstrap/gcc-4.7.ncl`, `bootstrap/gcc-10.ncl`, `bootstrap/musl-full.ncl`, `bootstrap/binutils-full.ncl`, and `bootstrap/seed-full.ncl`), `bootstrap/selftest.ncl`, `bootstrap/integration-test.ncl`, and final self-build proof. Validation MUST fail closed when a stage emits placeholder text, exits through a deferred task, or falls back to the legacy provider. The final provider MUST expose the normalized seed contract fields consumed by later bootstrap derivations.

#### Scenario: Stage placeholder is rejected

- GIVEN a bootstrap stage emits `ERROR: ... is a placeholder`
- WHEN source-chain validation evaluates completion status
- THEN the stage is reported incomplete
- AND full-source bootstrap status remains blocked

#### Scenario: Stage chain validates in order

- GIVEN each source-chain derivation has been implemented
- WHEN validation runs the ordered stage inventory from `bootstrap/stage0-posix.ncl` through `bootstrap/seed-full.ncl`
- THEN every stage transcript records the command, provider selection, exit status, output path, and stage-local fallback status
- AND later stages consume only the normalized provider contract

#### Scenario: Final proof binds provider evidence

- GIVEN `bootstrap/seed-full.ncl` satisfies the normalized provider contract
- WHEN `crunch self-build` completes with the source-built provider
- THEN proof metadata records provider kind, manifest digest, provider output digest, proof bundle digest, and explicit `self-build-proof: fallback-event=<kind>` markers or `self-build-proof: fallback-event=none`
- AND docs separate remaining trust roots from eliminated binary-provider trust

#### Scenario: Final bootstrap tests use source-built provider

- GIVEN the source-built provider satisfies the normalized seed contract
- WHEN `bootstrap/selftest.ncl` and `bootstrap/integration-test.ncl` are built
- THEN both transcripts record source-built provider selection
- AND neither transcript uses the legacy musl.cc provider

<!-- MODIFIED Requirements -->

### Requirement: Full-source bootstrap claim requires evidence

Crunch MUST withhold the full-source bootstrap claim until the source-root manifest validates for the full-source profile, the lineage manifest validates for the StageX-class profile, every named live-bootstrap placeholder is replaced, source-built stage transcripts exist, and self-build proof completes with the selected source-built provider.
ID: bootstrap.fullsource.claim.evidence

The claim evidence MUST include provider kind (`source-root` for the full-source profile or StageX-class lineage serialized as `stagex-lineage` for the StageX-class profile), manifest digest, provider output digest, proof bundle digest, stage-by-stage build transcripts through `bootstrap/seed-full.ncl`, `bootstrap/selftest.ncl` and `bootstrap/integration-test.ncl` transcripts, proof metadata bound to the selected source-built provider, explicit `self-build-proof: fallback-event=<kind>` markers or `self-build-proof: fallback-event=none`, and a docs update that separates remaining trusted roots from eliminated binary-provider trust. A prerequisite-only check, placeholder derivation, deferred task, archived partial-scaffolding change, unfinished successor task, legacy-provider fallback, host-bwrap fallback, or checkout/source-discovery fallback MUST NOT count as full-source bootstrap evidence.

#### Scenario: Placeholder blocks full-source claim

- GIVEN any of `bootstrap/binutils-tcc.ncl`, `bootstrap/gcc-4.0.ncl`, `bootstrap/gcc-4.7.ncl`, `bootstrap/gcc-10.ncl`, `bootstrap/musl-full.ncl`, `bootstrap/binutils-full.ncl`, or `bootstrap/seed-full.ncl` still emits placeholder text
- WHEN bootstrap maturity is reported
- THEN full-source bootstrap evidence remains absent
- AND the report names the unresolved placeholder stage

#### Scenario: Final source proof records docs separation

- GIVEN all stage transcripts and self-build proof complete with the source-built provider
- WHEN bootstrap maturity docs are updated
- THEN the docs name remaining trust roots and eliminated binary-provider trust separately
- AND the proof bundle digest is recorded next to provider kind, manifest digest, and provider output digest

### Requirement: StageX-class self-build proof binds lineage evidence

Crunch MUST require the full StageX-class evidence tuple before this live-bootstrap source-chain change can satisfy any StageX-class bootstrap claim.
ID: bootstrap.stagex.selfbuild.proof

The proof metadata MUST bind audited seed digest, lineage manifest digest, stage graph digest, normalized provider digest, staged source digest, stage1 and stage2 crunch binary digests, bootstrap-tool digests, protected execution audit digest when used, final proof bundle digest, canonical reproducibility report digest, and explicit `self-build-proof: fallback-event=<kind>` markers or `self-build-proof: fallback-event=none`. The proof MUST fail closed when any named live-bootstrap placeholder remains, when the legacy provider is selected, when host-bwrap or checkout/source-discovery fallback appears, or when forbidden host executables run during the protected stage.

#### Scenario: StageX proof records complete evidence tuple

- GIVEN the live-bootstrap source chain materializes a normalized provider
- AND a full protected self-build proof completes with that provider
- WHEN StageX-class proof metadata is written
- THEN it records audited seed digest, lineage manifest digest, stage graph digest, normalized provider digest, staged source digest, stage1 digest, stage2 digest, bootstrap-tool digests, protected execution audit digest when used, final proof bundle digest, canonical reproducibility report digest, and explicit `self-build-proof: fallback-event=<kind>` markers or `self-build-proof: fallback-event=none`
- AND it records provider kind as StageX-class lineage serialized as `stagex-lineage`

#### Scenario: StageX proof rejects legacy fallback

- GIVEN any stage selected the legacy musl.cc provider
- WHEN StageX-class evidence is requested
- THEN the proof fails closed
- AND no StageX-class claim is emitted

<!-- synced from openspec change: live-part-grep-2-4 -->
<!-- ADDED Requirements -->

### Requirement: Live-bootstrap part grep 2.4 is independently tracked
Crunch MUST track the live-bootstrap part `grep 2.4` as an independent bootstrap change bound to `bootstrap/grep-2.4-musl.ncl`.
ID: bootstrap.part.grep.2.4

The part scope MUST include only the source pins, patches, derivation logic, output contract, and evidence needed for `bootstrap/grep-2.4-musl.ncl` and direct predecessor compatibility. Broader chain validation MAY depend on this part, but MUST NOT replace this part's own build and smoke evidence.

#### Scenario: Part evidence is isolated

- GIVEN implementation work touches `bootstrap/grep-2.4-musl.ncl`
- WHEN the part is marked complete
- THEN the change records source-pin audit evidence for `bootstrap/grep-2.4-musl.ncl`
- AND it records a successful `crunch build bootstrap/grep-2.4-musl.ncl` transcript
- AND it records a smoke check for the produced output contract

#### Scenario: Downstream blockers stay local

- GIVEN a downstream bootstrap stage fails after `bootstrap/grep-2.4-musl.ncl` succeeds
- WHEN this part's evidence is reviewed
- THEN the downstream failure does not invalidate this part's completed evidence
- AND a separate part change tracks the downstream failure

<!-- synced from openspec change: live-part-make-3-82 -->
<!-- ADDED Requirements -->

### Requirement: Live-bootstrap part make 3.82 is independently tracked
Crunch MUST track the live-bootstrap part `make 3.82` as an independent bootstrap change bound to `bootstrap/make-tcc.ncl`.
ID: bootstrap.part.make.3.82

The part scope MUST include only the source pins, patches, derivation logic, output contract, and evidence needed for `bootstrap/make-tcc.ncl` and direct predecessor compatibility. Broader chain validation MAY depend on this part, but MUST NOT replace this part's own build and smoke evidence.

#### Scenario: Part evidence is isolated

- GIVEN implementation work touches `bootstrap/make-tcc.ncl`
- WHEN the part is marked complete
- THEN the change records source-pin audit evidence for `bootstrap/make-tcc.ncl`
- AND it records a successful `crunch build bootstrap/make-tcc.ncl` transcript
- AND it records a smoke check for the produced output contract

#### Scenario: Downstream blockers stay local

- GIVEN a downstream bootstrap stage fails after `bootstrap/make-tcc.ncl` succeeds
- WHEN this part's evidence is reviewed
- THEN the downstream failure does not invalidate this part's completed evidence
- AND a separate part change tracks the downstream failure

<!-- synced from openspec change: live-part-musl-1-1-24-tcc -->
<!-- ADDED Requirements -->

### Requirement: Live-bootstrap part musl 1.1.24 (tcc) is independently tracked
Crunch MUST track the live-bootstrap part `musl 1.1.24 and musl_target` as an independent bootstrap change bound to `bootstrap/musl-1.1.24-tcc.ncl`.
ID: bootstrap.part.musl.1.1.24.tcc

The part scope MUST include only the source pins, patches, derivation logic, output contract, and evidence needed for `bootstrap/musl-1.1.24-tcc.ncl` and direct predecessor compatibility. Broader chain validation MAY depend on this part, but MUST NOT replace this part's own build and smoke evidence.

#### Scenario: Part evidence is isolated

- GIVEN implementation work touches `bootstrap/musl-1.1.24-tcc.ncl`
- WHEN the part is marked complete
- THEN the change records source-pin audit evidence for `bootstrap/musl-1.1.24-tcc.ncl`
- AND it records a successful `crunch build bootstrap/musl-1.1.24-tcc.ncl` transcript
- AND it records a smoke check for the produced output contract

#### Scenario: Downstream blockers stay local

- GIVEN a downstream bootstrap stage fails after `bootstrap/musl-1.1.24-tcc.ncl` succeeds
- WHEN this part's evidence is reviewed
- THEN the downstream failure does not invalidate this part's completed evidence
- AND a separate part change tracks the downstream failure

<!-- synced from openspec change: live-part-musl-1-1-24-tcc-musl -->
<!-- ADDED Requirements -->

### Requirement: Live-bootstrap part musl 1.1.24 (tcc-musl) is independently tracked
Crunch MUST track the live-bootstrap part `musl 1.1.24 and musl_target` as an independent bootstrap change bound to `bootstrap/musl-1.1.24-tcc-musl.ncl`.
ID: bootstrap.part.musl.1.1.24.tcc.musl

The part scope MUST include only the source pins, patches, derivation logic, output contract, and evidence needed for `bootstrap/musl-1.1.24-tcc-musl.ncl` and direct predecessor compatibility. Broader chain validation MAY depend on this part, but MUST NOT replace this part's own build and smoke evidence.

#### Scenario: Part evidence is isolated

- GIVEN implementation work touches `bootstrap/musl-1.1.24-tcc-musl.ncl`
- WHEN the part is marked complete
- THEN the change records source-pin audit evidence for `bootstrap/musl-1.1.24-tcc-musl.ncl`
- AND it records a successful `crunch build bootstrap/musl-1.1.24-tcc-musl.ncl` transcript
- AND it records a smoke check for the produced output contract

#### Scenario: Downstream blockers stay local

- GIVEN a downstream bootstrap stage fails after `bootstrap/musl-1.1.24-tcc-musl.ncl` succeeds
- WHEN this part's evidence is reviewed
- THEN the downstream failure does not invalidate this part's completed evidence
- AND a separate part change tracks the downstream failure

<!-- synced from openspec change: live-part-patch-2-5-9 -->
<!-- ADDED Requirements -->

### Requirement: Live-bootstrap part patch 2.5.9 is independently tracked
Crunch MUST track the live-bootstrap part `patch 2.5.9` as an independent bootstrap change bound to `bootstrap/patch-tcc.ncl`.
ID: bootstrap.part.patch.2.5.9

The part scope MUST include only the source pins, patches, derivation logic, output contract, and evidence needed for `bootstrap/patch-tcc.ncl` and direct predecessor compatibility. Broader chain validation MAY depend on this part, but MUST NOT replace this part's own build and smoke evidence.

#### Scenario: Part evidence is isolated

- GIVEN implementation work touches `bootstrap/patch-tcc.ncl`
- WHEN the part is marked complete
- THEN the change records source-pin audit evidence for `bootstrap/patch-tcc.ncl`
- AND it records a successful `crunch build bootstrap/patch-tcc.ncl` transcript
- AND it records a smoke check for the produced output contract

#### Scenario: Downstream blockers stay local

- GIVEN a downstream bootstrap stage fails after `bootstrap/patch-tcc.ncl` succeeds
- WHEN this part's evidence is reviewed
- THEN the downstream failure does not invalidate this part's completed evidence
- AND a separate part change tracks the downstream failure

<!-- synced from openspec change: live-part-tcc-musl -->
<!-- ADDED Requirements -->

### Requirement: Live-bootstrap part tcc linked to musl is independently tracked
Crunch MUST track the live-bootstrap part `musl 1.1.24 and musl_target` as an independent bootstrap change bound to `bootstrap/tcc-musl.ncl`.
ID: bootstrap.part.tcc.musl

The part scope MUST include only the source pins, patches, derivation logic, output contract, and evidence needed for `bootstrap/tcc-musl.ncl` and direct predecessor compatibility. Broader chain validation MAY depend on this part, but MUST NOT replace this part's own build and smoke evidence.

#### Scenario: Part evidence is isolated

- GIVEN implementation work touches `bootstrap/tcc-musl.ncl`
- WHEN the part is marked complete
- THEN the change records source-pin audit evidence for `bootstrap/tcc-musl.ncl`
- AND it records a successful `crunch build bootstrap/tcc-musl.ncl` transcript
- AND it records a smoke check for the produced output contract

#### Scenario: Downstream blockers stay local

- GIVEN a downstream bootstrap stage fails after `bootstrap/tcc-musl.ncl` succeeds
- WHEN this part's evidence is reviewed
- THEN the downstream failure does not invalidate this part's completed evidence
- AND a separate part change tracks the downstream failure

<!-- synced from openspec change: live-part-tcc-musl-prep -->
<!-- ADDED Requirements -->

### Requirement: Live-bootstrap part tcc musl prep is independently tracked
Crunch MUST track the live-bootstrap part `musl 1.1.24 and musl_target` as an independent bootstrap change bound to `bootstrap/tcc-musl-prep.ncl`.
ID: bootstrap.part.tcc.musl.prep

The part scope MUST include only the source pins, patches, derivation logic, output contract, and evidence needed for `bootstrap/tcc-musl-prep.ncl` and direct predecessor compatibility. Broader chain validation MAY depend on this part, but MUST NOT replace this part's own build and smoke evidence.

#### Scenario: Part evidence is isolated

- GIVEN implementation work touches `bootstrap/tcc-musl-prep.ncl`
- WHEN the part is marked complete
- THEN the change records source-pin audit evidence for `bootstrap/tcc-musl-prep.ncl`
- AND it records a successful `crunch build bootstrap/tcc-musl-prep.ncl` transcript
- AND it records a smoke check for the produced output contract

#### Scenario: Downstream blockers stay local

- GIVEN a downstream bootstrap stage fails after `bootstrap/tcc-musl-prep.ncl` succeeds
- WHEN this part's evidence is reviewed
- THEN the downstream failure does not invalidate this part's completed evidence
- AND a separate part change tracks the downstream failure

<!-- synced from openspec change: live-part-tcc-musl-v2 -->
<!-- ADDED Requirements -->

### Requirement: Live-bootstrap part tcc musl v2 is independently tracked
Crunch MUST track the live-bootstrap part `musl 1.1.24 and musl_target` as an independent bootstrap change bound to `bootstrap/tcc-musl-v2.ncl`.
ID: bootstrap.part.tcc.musl.v2

The part scope MUST include only the source pins, patches, derivation logic, output contract, and evidence needed for `bootstrap/tcc-musl-v2.ncl` and direct predecessor compatibility. Broader chain validation MAY depend on this part, but MUST NOT replace this part's own build and smoke evidence.

#### Scenario: Part evidence is isolated

- GIVEN implementation work touches `bootstrap/tcc-musl-v2.ncl`
- WHEN the part is marked complete
- THEN the change records source-pin audit evidence for `bootstrap/tcc-musl-v2.ncl`
- AND it records a successful `crunch build bootstrap/tcc-musl-v2.ncl` transcript
- AND it records a smoke check for the produced output contract

#### Scenario: Downstream blockers stay local

- GIVEN a downstream bootstrap stage fails after `bootstrap/tcc-musl-v2.ncl` succeeds
- WHEN this part's evidence is reviewed
- THEN the downstream failure does not invalidate this part's completed evidence
- AND a separate part change tracks the downstream failure

<!-- synced from openspec change: repair-make-tcc-amd64-varargs -->
<!-- ADDED Requirements -->

### Requirement: First GNU make pass is amd64 executable [r[bootstrap.part.make.3.82.amd64.execution]]
Crunch MUST build `bootstrap/make-tcc.ncl` into a `make 3.82` output that executes basic Makefiles on amd64.

The output MUST include `bin/make`, report `GNU Make 3.82`, execute a simple Makefile target successfully, and reject a missing target with a controlled nonzero exit rather than a signal or segmentation fault. Source-level repair completion evidence MUST include a source-pin audit transcript and either direct successful runtime transcripts or an active runtime-validation successor that records the long-build, smoke, and host-leakage proof still required before downstream completion claims.

#### Scenario: Simple Makefile runs [r[bootstrap.part.make.3.82.amd64.execution.scenario.simple-makefile]]

- GIVEN `bootstrap/make-tcc.ncl` has been built with the documented bootstrap environment
- WHEN the produced `bin/make -f Makefile` runs a Makefile whose `all` target echoes `make-smoke-ok`
- THEN stdout contains `make-smoke-ok`
- AND the process exits successfully

#### Scenario: Missing target fails cleanly [r[bootstrap.part.make.3.82.amd64.execution.scenario.missing-target]]

- GIVEN the same produced `bin/make`
- WHEN it is asked to build a missing target
- THEN it exits nonzero
- AND the exit status is not a signal-derived segmentation fault

#### Scenario: Evidence is complete [r[bootstrap.part.make.3.82.amd64.execution.scenario.evidence-complete]]

- GIVEN the repair is marked complete
- WHEN reviewers inspect this change
- THEN they can find the source-pin audit transcript
- AND they can find either successful build, smoke, and host-leakage transcripts
- OR they can find an active runtime-validation successor that owns those remaining proof transcripts

<!-- synced from openspec change: repair-make-tcc-amd64-varargs-runtime-validation -->
<!-- ADDED Requirements -->

### Requirement: Make 3.82 amd64 runtime validation completes [r[bootstrap.part.make.3.82.amd64.runtime-validation]]
Crunch MUST preserve runtime proof for the repaired `bootstrap/make-tcc.ncl` output before the first GNU make amd64 repair is treated as complete.

The proof MUST include a long-budget build transcript, the produced output path or concrete failure diagnostics, version smoke evidence for `GNU Make 3.82`, a simple Makefile positive smoke, a missing-target negative smoke that fails without a signal/segfault, and a host-leakage scan over the derivation, transcript, and output.

#### Scenario: Long build produces make output [r[bootstrap.part.make.3.82.amd64.runtime-validation.scenario.build-output]]
- GIVEN the parent repair source state
- WHEN `crunch build bootstrap/make-tcc.ncl` runs with the documented bootstrap environment and a long validation budget
- THEN the transcript is preserved
- AND either a make output path is recorded or concrete failure diagnostics are recorded

#### Scenario: Make runtime smokes are complete [r[bootstrap.part.make.3.82.amd64.runtime-validation.scenario.smoke]]
- GIVEN a produced make output path
- WHEN the successor runs version, positive Makefile, and missing-target negative checks
- THEN version output contains `GNU Make 3.82`
- AND the positive Makefile prints `make-smoke-ok`
- AND the missing target exits nonzero without a signal-derived segmentation fault

#### Scenario: Host leakage is scanned [r[bootstrap.part.make.3.82.amd64.runtime-validation.scenario.host-leakage]]
- GIVEN the build transcript and output metadata
- WHEN the host-leakage scan runs
- THEN undeclared host tools, host paths, and environment-derived inputs are either absent or recorded as concrete failures

<!-- synced from openspec change: repair-i386-tinycc26-object-emission -->

### Requirement: i386 TinyCC 0.9.26 object emission repair [r[bootstrap.i386-tinycc26-object-emission.repair]]

Crunch MUST repair the i386 TinyCC 0.9.26 proof so the generated x86_64-hosted/i386-targeting compiler can emit object files without segfaulting.

#### Scenario: Object emission succeeds [r[bootstrap.i386-tinycc26-object-emission.repair.object]]

- GIVEN the generated `tcc26-i386` diagnostic compiler
- WHEN it compiles assembly or C input with `-c`
- THEN it MUST produce an object file without a segmentation fault.

### Requirement: i386 TinyCC 0.9.26 runtime proof [r[bootstrap.i386-tinycc26-object-emission.runtime-proof]]

Crunch MUST prove the repaired i386 TinyCC 0.9.26 path can produce and execute a no-libc i386 ELF before using it as a basis for later i386 bootstrap stages.

#### Scenario: i386 exit42 runs [r[bootstrap.i386-tinycc26-object-emission.runtime-proof.exit42]]

- GIVEN `tcc26-i386` can emit objects
- WHEN it links the no-libc `_start` smoke executable
- THEN the produced i386 ELF MUST execute inside Crunch's sandbox with exit code 42.

<!-- synced from openspec change: spike-i386-tcc27-make-pass1 -->

### Requirement: i386 TinyCC 0.9.27 Make pass1 spike [r[bootstrap.i386-tcc27-make-pass1.spike]]

Crunch MUST provide a bounded sibling proof for the i386 live-bootstrap sequence from TinyCC 0.9.26 through TinyCC 0.9.27 to GNU Make 3.82 pass1 before changing production bootstrap routing.

#### Scenario: First blocker is recorded [r[bootstrap.i386-tcc27-make-pass1.spike.blocker]]

- GIVEN the proven `tcc26-i386` predecessor compiler
- WHEN the sibling proof attempts TinyCC 0.9.27 and Make 3.82 pass1
- THEN it MUST either pass the Make smoke or save the first concrete failing step with logs.

#### Scenario: Make smoke is required for success [r[bootstrap.i386-tcc27-make-pass1.spike.make-smoke]]

- GIVEN the sibling proof produces a `make` binary
- WHEN the proof claims success
- THEN `make --version` and a trivial Makefile execution MUST both pass inside Crunch's sandbox.

<!-- synced from openspec change: spike-i386-mes-runtime-layout -->

### Requirement: i386 Mes runtime/header layout spike [r[bootstrap.i386-mes-runtime-layout.spike]]

Crunch MUST keep the i386 Mes runtime/header layout investigation as a bounded sibling diagnostic before changing production Make/TinyCC bootstrap routing.

#### Scenario: Layout proof records first blocker [r[bootstrap.i386-mes-runtime-layout.evidence]]

- GIVEN the proven `tcc26-i386` predecessor
- WHEN Crunch builds the i386 Mes runtime/header layout spike
- THEN the output MUST include logs and a summary naming the first blocked step or the next successful handoff boundary.

### Requirement: i386 TinyCC 0.9.27 handoff blocker [r[bootstrap.i386-mes-runtime-layout.blocker]]

Crunch MUST distinguish Mes header/CRT layout progress from complete runtime library availability before attempting the TinyCC 0.9.27 -> Make 3.82 handoff.

#### Scenario: Runtime library blocker is explicit [r[bootstrap.i386-mes-runtime-layout.blocker.runtime-library]]

- GIVEN an i386 Mes header tree and CRT object can be created
- WHEN runtime library or TinyCC 0.9.27 object compilation fails
- THEN the evidence MUST record the exact step, return code, and stderr excerpt.

<!-- synced from openspec change: repair-i386-mes-libtcc1-flags -->

### Requirement: i386 Mes libtcc1 compile flags [r[bootstrap.i386-mes-libtcc1-flags.repair]]

Crunch MUST compile the i386 Mes `libtcc1.c` proof with flags that avoid unsupported broad float and long-long helper emission in the `tcc26-i386` predecessor.

#### Scenario: Real libtcc1 archive is created [r[bootstrap.i386-mes-libtcc1-flags.evidence]]

- GIVEN the i386 Mes runtime-layout sibling proof
- WHEN it builds Mes `lib/libtcc1.c`
- THEN `runtime_libtcc1_object` and `runtime_libtcc1_archive` MUST exit 0 without using the prior placeholder object path.

### Requirement: Post-libtcc1 TinyCC 0.9.27 blocker [r[bootstrap.i386-mes-libtcc1-flags.next-blocker]]

Crunch MUST record the next TinyCC handoff blocker after real i386 `libtcc1.a` creation.

#### Scenario: Next blocked step is explicit [r[bootstrap.i386-mes-libtcc1-flags.next-blocker.explicit]]

- GIVEN a real i386 Mes `libtcc1.a` archive exists
- WHEN the TinyCC 0.9.27 object probe fails
- THEN the evidence MUST name the failing step, return code, and stderr excerpt.

<!-- synced from openspec change: repair-i386-tcc27-source-diagnostics -->

### Requirement: i386 TinyCC 0.9.27 source diagnostics [r[bootstrap.i386-tcc27-source-diagnostics.narrowing]]

Crunch MUST keep the i386 TinyCC 0.9.27 handoff diagnostic narrow enough to distinguish predecessor compiler source-emission failures from Mes runtime-library failures.

#### Scenario: Focused diagnostic matrix is preserved [r[bootstrap.i386-tcc27-source-diagnostics.evidence]]

- GIVEN the sibling i386 Mes runtime-layout proof has built `runtime_libtcc1_object` and `runtime_libtcc1_archive` successfully
- WHEN the TinyCC 0.9.27 pass1 object probe fails
- THEN the proof output MUST include diagnostic return codes for no-line preprocessing, line-marker preprocessing, representative unit compiles, and the gating `tcc27_compile_object` step.

### Requirement: i386 TinyCC 0.9.27 pass1 parity patches [r[bootstrap.i386-tcc27-source-diagnostics.pass1-parity]]

Crunch MUST keep the sibling tcc27 pass1 probe aligned with live-bootstrap pass1 source edits and flags when narrowing the handoff blocker.

#### Scenario: Missing pass1 edit and unrelated flags are corrected [r[bootstrap.i386-tcc27-source-diagnostics.pass1-parity.flags]]

- GIVEN the tcc27 pass1 sibling proof patches TinyCC 0.9.27 sources
- WHEN it reaches the pass1 object and link probes
- THEN it SHOULD include the live-bootstrap `check-reloc-null` edit and avoid unrelated Mes feature toggles on the tcc27 source compile/link commands.

<!-- synced from openspec change: repair-i386-tcc27-preprocessor-tccgen -->

### Requirement: i386 TinyCC 0.9.27 handoff diagnostics [r[i386-tcc27-handoff-diagnostics]]

The system MUST maintain a sibling i386 TinyCC 0.9.27 handoff diagnostic derivation that records source-emission, per-unit compile, full object compile, link, and downstream Make smoke boundaries before production bootstrap routing depends on that path.

#### Scenario: predecessor source-emission blocker is repaired or narrowed [r[i386-tcc27-handoff-diagnostics.source-emission]]

- GIVEN the sibling i386 Mes runtime layout derivation
- WHEN the diagnostic derivation runs after a source-normalization patch
- THEN the evidence MUST show whether line-marker preprocessing, `tccgen.c` compilation, full `ONE_SOURCE=1` compilation, and the first subsequent blocker pass or fail with captured rc/stdout/stderr logs.

### Requirement: make 3.82 runtime validation waits for amd64 repair [r[bootstrap.part.make.3.82.runtime-validation]]
The system MUST keep make 3.82 runtime proof incomplete until the amd64 varargs repair is complete and `bootstrap/make-tcc.ncl` executes a simple Makefile successfully.

#### Scenario: Version output alone is insufficient [r[bootstrap.part.make.3.82.runtime-validation.version-insufficient]]
- **GIVEN** the produced make binary reports GNU Make 3.82 with `--version`
- **WHEN** simple Makefile execution still segfaults or fails
- **THEN** runtime validation remains incomplete

#### Scenario: Runtime proof closes after repair [r[bootstrap.part.make.3.82.runtime-validation.after-repair]]
- **GIVEN** `repair-make-tcc-amd64-varargs` is complete
- **WHEN** `bootstrap/make-tcc.ncl` builds and runs a simple Makefile successfully
- **THEN** the evidence records output path, smoke results, fallback status, and leakage-scan results

### Requirement: patch 2.5.9 runtime validation waits for prerequisite execution proof [r[bootstrap.part.patch.2.5.9.runtime-validation]]
The system MUST keep patch 2.5.9 runtime proof incomplete until prerequisite make/tcc execution blockers are resolved and the produced patch binary applies a simple diff successfully.

#### Scenario: Version output alone is insufficient [r[bootstrap.part.patch.2.5.9.runtime-validation.version-insufficient]]
- **GIVEN** the produced patch binary reports its version
- **WHEN** applying a simple unified diff fails or is not tested
- **THEN** runtime validation remains incomplete

#### Scenario: Patch application is proven [r[bootstrap.part.patch.2.5.9.runtime-validation.diff-application]]
- **GIVEN** prerequisite runtime blockers are resolved
- **WHEN** `bootstrap/patch-tcc.ncl` builds and applies a simple unified diff successfully
- **THEN** the evidence records output path, smoke results, fallback status, and leakage-scan results

### Requirement: Live-bootstrap part bzip2 1.0.8 (tcc) is independently tracked [r[bootstrap.part.bzip2.1.0.8.tcc]]
Crunch MUST track the live-bootstrap part `bzip2 1.0.8` as an independent bootstrap change bound to `bootstrap/bzip2-tcc.ncl`.

The part scope MUST include only the source pins, patches, derivation logic, output contract, and evidence needed for `bootstrap/bzip2-tcc.ncl` and direct predecessor compatibility. Broader chain validation MAY depend on this part, but MUST NOT replace this part's own build and smoke evidence.

#### Scenario: Part evidence is isolated [r[bootstrap.part.bzip2.1.0.8.tcc.evidence-isolated]]

- GIVEN implementation work touches `bootstrap/bzip2-tcc.ncl`
- WHEN the part is marked complete
- THEN the change records source-pin audit evidence for `bootstrap/bzip2-tcc.ncl`
- AND it records a successful `crunch build bootstrap/bzip2-tcc.ncl` transcript
- AND it records a smoke check for the produced output contract

#### Scenario: Downstream blockers stay local [r[bootstrap.part.bzip2.1.0.8.tcc.downstream-local]]

- GIVEN a downstream bootstrap stage fails after `bootstrap/bzip2-tcc.ncl` succeeds
- WHEN this part's evidence is reviewed
- THEN the downstream failure does not invalidate this part's completed evidence
- AND a separate part change tracks the downstream failure

### Requirement: tcc musl prep runtime validation waits for prerequisite execution proof [r[bootstrap.part.tcc.musl.prep.runtime-validation]]
The system MUST keep the musl-prep TinyCC bridge runtime proof incomplete until prerequisite runtime blockers are resolved and the produced bridge/compiler carry-forward contract is smoke-tested.

#### Scenario: Bridge carry-forward artifacts are mandatory [r[bootstrap.part.tcc.musl.prep.runtime-validation.bridge-artifacts]]
- **GIVEN** `bootstrap/tcc-musl-prep.ncl` produces `bin/tcc`
- **WHEN** the Mes libc archive or Mes headers are missing
- **THEN** runtime validation remains incomplete

#### Scenario: Bridge smoke proves output [r[bootstrap.part.tcc.musl.prep.runtime-validation.smoke]]
- **GIVEN** prerequisite runtime blockers are resolved
- **WHEN** `bootstrap/tcc-musl-prep.ncl` builds and `tcc -v` succeeds
- **THEN** the evidence records output path, smoke results, fallback status, and leakage-scan results

### Requirement: tcc musl runtime validation waits for prerequisite execution proof [r[bootstrap.part.tcc.musl.runtime-validation]]
The system MUST keep the musl-linked TinyCC runtime proof incomplete until prerequisite runtime blockers are resolved and the produced compiler can compile a trivial C program using its installed runtime archive.

#### Scenario: Runtime archive is mandatory [r[bootstrap.part.tcc.musl.runtime-validation.libtcc1]]
- **GIVEN** `bootstrap/tcc-musl.ncl` produces `bin/tcc`
- **WHEN** `lib/tcc/libtcc1.a` is missing
- **THEN** runtime validation remains incomplete

#### Scenario: Compiler smoke proves output [r[bootstrap.part.tcc.musl.runtime-validation.compile-smoke]]
- **GIVEN** prerequisite runtime blockers are resolved
- **WHEN** `bootstrap/tcc-musl.ncl` builds and compiles a trivial C program successfully
- **THEN** the evidence records output path, smoke results, fallback status, and leakage-scan results

### Requirement: rebuilt musl 1.1.24 runtime validation waits for prerequisite execution proof [r[bootstrap.part.musl.1.1.24.tcc.musl.runtime-validation]]
The system MUST keep the rebuilt musl pass runtime proof incomplete until prerequisite runtime blockers are resolved and the produced musl output contract is smoke-tested.

#### Scenario: Source-level hardening is not runtime proof [r[bootstrap.part.musl.1.1.24.tcc.musl.runtime-validation.source-hardening]]
- **GIVEN** `bootstrap/musl-1.1.24-tcc-musl.ncl` fails closed on missing startup objects
- **WHEN** the derivation has not been built successfully in Crunch
- **THEN** runtime validation remains incomplete

#### Scenario: Rebuilt musl contract is proven [r[bootstrap.part.musl.1.1.24.tcc.musl.runtime-validation.output-contract]]
- **GIVEN** prerequisite runtime blockers are resolved
- **WHEN** `bootstrap/musl-1.1.24-tcc-musl.ncl` builds successfully
- **THEN** the evidence proves `libc.a`, installed headers, and a startup object exist without host fallback

### Requirement: tcc musl v2 runtime validation waits for prerequisite execution proof [r[bootstrap.part.tcc.musl.v2.runtime-validation]]
The system MUST keep the final musl TinyCC runtime proof incomplete until prerequisite runtime blockers are resolved and the produced compiler can compile a trivial C program using its installed runtime archive.

#### Scenario: Runtime archive is mandatory [r[bootstrap.part.tcc.musl.v2.runtime-validation.libtcc1]]
- **GIVEN** `bootstrap/tcc-musl-v2.ncl` produces `bin/tcc`
- **WHEN** `lib/tcc/libtcc1.a` is missing
- **THEN** runtime validation remains incomplete

#### Scenario: Final compiler smoke proves output [r[bootstrap.part.tcc.musl.v2.runtime-validation.compile-smoke]]
- **GIVEN** prerequisite runtime blockers are resolved
- **WHEN** `bootstrap/tcc-musl-v2.ncl` builds and compiles a trivial C program successfully
- **THEN** the evidence records output path, smoke results, fallback status, and leakage-scan results

### Requirement: Live-bootstrap part sed 4.0.9 (tcc) is independently tracked
Crunch MUST track the live-bootstrap part `sed 4.0.9` as an independent bootstrap change bound to `bootstrap/sed-tcc.ncl`.
ID: bootstrap.part.sed.4.0.9.tcc

The part scope MUST include only the source pins, patches, derivation logic, output contract, and evidence needed for `bootstrap/sed-tcc.ncl` and direct predecessor compatibility. Broader chain validation MAY depend on this part, but MUST NOT replace this part's own build and smoke evidence.

#### Scenario: Part evidence is isolated

- GIVEN implementation work touches `bootstrap/sed-tcc.ncl`
- WHEN the part is marked complete
- THEN the change records source-pin audit evidence for `bootstrap/sed-tcc.ncl`
- AND it records a successful `crunch build bootstrap/sed-tcc.ncl` transcript
- AND it records a smoke check for the produced output contract

#### Scenario: Downstream blockers stay local

- GIVEN a downstream bootstrap stage fails after `bootstrap/sed-tcc.ncl` succeeds
- WHEN this part's evidence is reviewed
- THEN the downstream failure does not invalidate this part's completed evidence
- AND a separate part change tracks the downstream failure

### Requirement: Live-bootstrap part bzip2 1.0.8 (musl) is independently tracked [r[bootstrap.part.bzip2.1.0.8.musl]]
Crunch MUST track the live-bootstrap part `bzip2 1.0.8` as an independent bootstrap change bound to `bootstrap/bzip2-1.0.8-musl.ncl`.

The part scope MUST include only the source pins, patches, derivation logic, output contract, and evidence needed for `bootstrap/bzip2-1.0.8-musl.ncl` and direct predecessor compatibility. Broader chain validation MAY depend on this part, but MUST NOT replace this part's own build and smoke evidence.

#### Scenario: Part evidence is isolated [r[bootstrap.part.bzip2.1.0.8.musl.evidence-isolated]]

- GIVEN implementation work touches `bootstrap/bzip2-1.0.8-musl.ncl`
- WHEN the part is marked complete
- THEN the change records source-pin audit evidence for `bootstrap/bzip2-1.0.8-musl.ncl`
- AND it records a successful `crunch build bootstrap/bzip2-1.0.8-musl.ncl` transcript
- AND it records a smoke check for the produced output contract

#### Scenario: Downstream blockers stay local [r[bootstrap.part.bzip2.1.0.8.musl.downstream-local]]

- GIVEN a downstream bootstrap stage fails after `bootstrap/bzip2-1.0.8-musl.ncl` succeeds
- WHEN this part's evidence is reviewed
- THEN the downstream failure does not invalidate this part's completed evidence
- AND a separate part change tracks the downstream failure

