# Bootstrap Specification

## Purpose

Defines mantle bootstrap requirements for source lineage, provider contracts,
self-build proof evidence, seed-chain replacement, and intermediate tool
derivations.
## Requirements
### Requirement: Full-source bootstrap root manifest

Mantle MUST define a versioned full-source bootstrap root manifest that names every source artifact, patch, digest, extraction rule, and expected provider output needed before the normalized seed contract is available.
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

Mantle MUST support a source-built bootstrap provider that satisfies the existing normalized `bootstrap/seed.ncl` contract without deriving from the current musl.cc binary toolchain tarball.
ID: bootstrap.fullsource.provider.contract

The source-built provider MUST expose the same contract fields later bootstrap stages consume today: target-prefixed tool paths, headers, libraries, retained-tool metadata, reduction metadata, and provider notes. Later bootstrap derivations MUST keep depending on the normalized contract instead of provider-specific raw layouts. `mantle bootstrap --source-root <manifest>` MUST select the source-built provider; the existing `mantle bootstrap --fetch` path MUST remain the seed-assisted legacy provider; specifying both MUST fail before provider work starts.

#### Scenario: Source-built provider feeds make

- GIVEN a valid full-source root manifest and a source-built provider output
- WHEN `mantle build bootstrap/make.ncl` consumes that provider through
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

Mantle MUST withhold the full-source bootstrap claim until the source-root manifest validates for the full-source profile, the lineage manifest validates for the StageX-class profile, every named live-bootstrap placeholder is replaced, source-built stage transcripts exist, and self-build proof completes with the selected source-built provider.
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

Mantle MUST define a StageX-class bootstrap lineage whose trusted bootstrap root is an auditable seed plus source artifacts, not a prebuilt compiler, prebuilt build tool, Nix store path, or musl.cc-derived binary provider.
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
it can satisfy this profile. Mantle-owned fingerprints MUST use BLAKE3 unless an
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

Mantle MUST materialize the normalized seed provider through the declared StageX-class lineage before later bootstrap derivations consume `bootstrap/seed.ncl`.
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
- WHEN `mantle build bootstrap/make.ncl` consumes the provider through
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

Mantle MUST require the full StageX-class evidence tuple before this live-bootstrap source-chain change can satisfy any StageX-class bootstrap claim.
ID: bootstrap.stagex.selfbuild.proof

The proof metadata MUST bind selected provider kind, audited seed digest, lineage manifest digest, stage graph digest, normalized provider digest, staged source digest, stage1 and stage2 mantle binary digests, bootstrap-tool digests, protected execution audit digest when used, final proof bundle digest, canonical reproducibility report digest, and explicit `self-build-proof: fallback-event=<kind>` markers or `self-build-proof: fallback-event=none`. The proof MUST fail closed when any named live-bootstrap placeholder remains, when the legacy provider is selected for a StageX claim, when host-bwrap or checkout/source-discovery fallback appears, or when forbidden host executables run during the protected stage. The bootstrap parity report MUST require a checked provider-kind linkage receipt for `mantle.self-build`; the receipt MUST use the schema `crunch-self-build-provider-kind-linkage-v1` and MUST prove that `proof_identity.selected_provider_kind`, `proof_linkage.selected_provider_kind`, and `prerequisites.provider_kind` are identical closed provider-kind values. The bootstrap parity report MUST also require a checked StageX lineage provider receipt for `seed-full.stagex-lineage`; scaffold receipts MUST be explicitly marked `lineage_receipt_status = scaffold-only`, MUST serialize `provider_kind = stagex-lineage`, MUST record digest-shaped audited seed, lineage manifest, stage graph, and normalized provider fields, and MUST record no fallback events. The bootstrap parity report MUST consume real self-build proof evidence for the `crunch.self-build` row only from a validated evidence bundle that links a release manifest, `mantle-deterministic-proof-receipt-v1` deterministic proof receipt, `mantle-proof-sandbox-v1:*` sandbox evidence, release verify receipt with deterministic status `eligible`, and optional portable summary artifacts. Such evidence MAY make `crunch.self-build` evidence-backed partial for the selected provider kind, but MUST NOT satisfy Guix or StageX parity unless the selected provider kind and all axis-specific source-root or StageX lineage evidence requirements are also satisfied.

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

#### Scenario: Release evidence binds selected provider kind [r[bootstrap.stagex.selfbuild.proof.provider-linkage]]

- GIVEN a full self-hosting proof bundle whose prerequisites name a selected provider kind
- WHEN release evidence is created from that proof bundle
- THEN `proof_linkage.selected_provider_kind` equals the proof bundle's selected provider kind
- AND verification fails if either side is missing, unknown, or mismatched

#### Scenario: Parity rejects missing provider-kind linkage receipt [r[bootstrap.stagex.selfbuild.proof.parity-missing-receipt]]

- GIVEN `bootstrap/crunch.ncl` exists
- BUT `bootstrap/evidence/crunch-self-build-provider-kind-linkage.json` is absent
- WHEN `mantle bootstrap parity-report --require guix` or `--require stagex` runs
- THEN `mantle.self-build` remains a blocker
- AND the row notes identify the missing provider-kind linkage receipt

#### Scenario: Parity accepts matching provider-kind linkage receipt as partial evidence [r[bootstrap.stagex.selfbuild.proof.parity-matching-receipt]]

- GIVEN a checked self-build provider-kind linkage receipt records the same closed provider kind in proof identity, proof linkage, and prerequisites
- WHEN the parity report loads that evidence
- THEN the `mantle.self-build` row may report evidence-backed `partial`
- AND it MUST NOT report `complete` until the full self-build proof and axis-specific evidence are present

#### Scenario: Parity rejects missing StageX lineage receipt [r[bootstrap.stagex.selfbuild.proof.stagex-lineage-missing-receipt]]

- GIVEN no StageX lineage provider receipt exists
- WHEN the bootstrap parity report evaluates `seed-full.stagex-lineage`
- THEN the row remains a StageX blocker
- AND the row notes identify the missing lineage receipt

#### Scenario: Parity accepts scaffold lineage receipt only as partial evidence [r[bootstrap.stagex.selfbuild.proof.stagex-lineage-scaffold-partial]]

- GIVEN a StageX lineage provider receipt has schema `crunch-stagex-lineage-provider-receipt-v1`
- AND it records `provider_kind = stagex-lineage`, `lineage_receipt_status = scaffold-only`, digest-shaped lineage fields, and no fallback events
- WHEN the bootstrap parity report evaluates `seed-full.stagex-lineage`
- THEN the row may report evidence-backed `partial`
- AND it MUST NOT report `complete` or unblock StageX parity until real audited lineage and self-build proof evidence exist

#### Scenario: Parity consumes bounded real self-build proof evidence [r[bootstrap.stagex.selfbuild.proof.real-proof-parity-consumption]]

- GIVEN a checked real self-build proof evidence bundle for `crunch.self-build`
- AND the bundle links a release manifest, deterministic proof receipt, sandbox evidence, release verify receipt, and summary artifacts with matching BLAKE3 digests
- AND the deterministic proof receipt has workflow `mantle-deterministic-proof-receipt-v1`, verdict `self-rebuild-match`, selected provider kind matching the release proof linkage, two distinct clean rebuild roots, matching artifact digest sets, and sandbox profile identity `mantle-proof-sandbox-v1:*`
- AND the release verify receipt reports deterministic release status `eligible`
- WHEN the bootstrap parity report evaluates `crunch.self-build`
- THEN the row reports the real proof evidence digest and selected provider kind
- AND the row may report evidence-backed `partial` for the selected provider kind
- AND the row MUST NOT report `complete` or unblock Guix/StageX parity unless all axis-specific source-root or StageX lineage requirements are also satisfied

#### Scenario: Parity rejects malformed or unsafe real self-build proof evidence [r[bootstrap.stagex.selfbuild.proof.real-proof-fail-closed]]

- GIVEN a real self-build proof evidence bundle is missing, malformed, names an unsupported workflow version, records mismatched provider kinds, records mismatched proof or summary digests, omits supported sandbox evidence, uses a direct-host or unsupported sandbox profile, reuses a proof store/root, or has release verify status other than `eligible`
- WHEN the bootstrap parity report evaluates `crunch.self-build`
- THEN the row remains a blocker for Guix and StageX axes that require self-build proof
- AND the row notes name the specific failed evidence check
- AND no deterministic, full-source, Guix, or StageX parity claim is emitted

### Requirement: Hex0 seed as trust root

The bootstrap chain MUST start from a hex0 seed binary of at most 512 bytes
for the target architecture, checked into the repository under `bootstrap/seeds/`.
The seed binary MUST be the exact output of assembling the stage0-posix hex0
source for AMD64.

#### Scenario: Seed is present and correctly sized

- GIVEN the repository checkout
- WHEN `bootstrap/seeds/AMD64/hex0-seed` is read
- THEN it is at most 512 bytes and matches the stage0-posix hex0 AMD64 binary

### Requirement: Stage0-posix as mantle derivation

The stage0-posix bootstrap (phases 0-28) MUST run as a single mantle derivation
that takes only the hex0 seed and the stage0-posix source tarball as inputs.
The derivation MUST produce mescc-tools (M1, hex2, kaem, blood-elf), M2-Planet,
and mescc-tools-extra (catm, cp, chmod, mkdir, untar, ungz, unbz2, unxz,
sha256sum).

#### Scenario: Stage0-posix builds from hex0

- GIVEN the hex0 seed and stage0-posix source pinned by hash
- WHEN `mantle build bootstrap/stage0-posix.ncl` runs
- THEN the output contains working M2-Planet, hex2, M1, and kaem binaries

### Requirement: GNU mes from stage0-posix output

GNU mes MUST be built as a mantle derivation using only M2-Planet and
mescc-tools from the stage0-posix output. The mes output MUST include both
the Scheme interpreter and the mes C compiler (`mescc`).

#### Scenario: Mes compiles a C program

- GIVEN the stage0-posix output
- WHEN `mantle build bootstrap/mes.ncl` runs
- THEN mes can compile a trivial C program to a working executable

### Requirement: Tinycc from mes

Tinycc 0.9.26 MUST be built using the mes C compiler. Tinycc 0.9.27 MUST
then be built using tinycc 0.9.26 (self-hosting). The final tinycc output
MUST be capable of building early GCC.

#### Scenario: Tinycc self-hosts

- GIVEN the mes output
- WHEN `mantle build bootstrap/tinycc.ncl` runs
- THEN tinycc 0.9.27 can compile C programs including early GCC prerequisites

### Requirement: GCC version ladder

GCC 4.0 pass1 libgcc member promotions MUST be evidence-backed one member at a time. The `__gcc_bcmp` member MUST implement byte-wise comparison semantics: it MUST return `0` for equal byte ranges and a non-zero value for the first differing byte over the requested length. The promotion MUST include a derivation-local smoke that exercises equal and unequal comparisons. GCC 4.0 pass1 driver query semantics MUST also be evidence-backed: `-dumpversion`, `-dumpmachine`, `-print-libgcc-file-name`, and `-print-search-dirs` MUST return deterministic GCC-shaped values tied to the installed artifact. GCC 4.0 installed `cc1` object-output semantics MUST be evidence-backed for the bounded GCC-shaped frontend invocation already accepted by the pass1 bridge. A promoted native `cc1` arithmetic slice MUST prove that the installed frontend handles a bounded C function containing integer arithmetic, comparison, branch, and return semantics without delegating object emission to TinyCC; it MUST record a checked receipt and smoke transcript, and it MUST NOT claim full GCC 4.0 compiler correctness. Promoted native generator-frontier slices MUST prove selected generator members' bounded output contracts with checked receipt evidence, while leaving unpromoted generator members and full native GCC 4.0 correctness blocked. Promoted native libiberty demangle slices MUST prove selected bounded Itanium demangle shapes with checked receipt evidence, while leaving full `cp-demangle` and native GCC 4.0 correctness blocked. A promoted native libiberty deeper nested-name demangle slice MUST prove a selected deeper zero-argument Itanium nested-name shape with checked receipt evidence, while leaving full `cp-demangle` and native GCC 4.0 correctness blocked. A promoted native libiberty single-int-argument demangle slice MUST prove a selected bounded Itanium single-`int` function-argument shape with checked receipt evidence, while leaving arbitrary type decoding, full `cp-demangle`, and native GCC 4.0 correctness blocked. A promoted native libiberty single-char-argument demangle slice MUST prove a selected bounded Itanium single-`char` function-argument shape with checked receipt evidence, while leaving arbitrary type decoding, full `cp-demangle`, and native GCC 4.0 correctness blocked. A promoted native libiberty single-long-argument demangle slice MUST prove a selected bounded Itanium single-`long` function-argument shape with checked receipt evidence, while leaving arbitrary type decoding, full `cp-demangle`, and native GCC 4.0 correctness blocked. A promoted native `cc1` logical/control-flow slice MUST prove that the installed frontend handles a selected C function containing comparison, logical `&&`/`||`, branch, and return behavior without delegating object emission to TinyCC; it MUST record checked receipt and transcript evidence, preserve the prior arithmetic/control-flow regression, and MUST NOT claim full GCC 4.0 compiler correctness. A promoted native `cc1` local-variable assignment slice MUST prove that the installed frontend handles a selected C function containing local `int` declarations, assignment or reassignment, expression use of those locals, and return behavior without delegating object emission to TinyCC; it MUST record checked receipt and transcript evidence, preserve the prior arithmetic/control-flow and logical/control-flow regressions, and MUST NOT claim full GCC 4.0 compiler correctness. A promoted native `cc1` helper-call slice MUST prove that the installed frontend handles a selected C input containing a helper function definition, a caller function, a call expression, argument passing, and return behavior without delegating object emission to TinyCC; it MUST record checked receipt and transcript evidence, preserve the prior arithmetic/control-flow, logical/control-flow, and local-variable regressions, and MUST NOT claim full GCC 4.0 compiler correctness. A promoted native `cc1` array-index slice MUST prove that the installed frontend handles a selected C input containing a local fixed `int` array declaration, indexed stores, indexed loads, expression use, and return behavior without delegating object emission to TinyCC; it MUST record checked receipt and transcript evidence, preserve the prior arithmetic/control-flow, logical/control-flow, local-variable, and helper-call regressions, and MUST NOT claim full GCC 4.0 compiler correctness. A promoted native `cc1` struct-field slice MUST prove that the installed frontend handles a selected C input containing a local `struct` type declaration, named field stores, named field loads, expression use, and return behavior without delegating object emission to TinyCC; it MUST record checked receipt and transcript evidence, preserve the prior arithmetic/control-flow, logical/control-flow, local-variable, helper-call, and array-index regressions, and MUST NOT claim full GCC 4.0 compiler correctness. GCC 4.0 native source-build frontier evidence MUST be checked separately from bounded installed-`cc1` semantic slices. A promoted native `cc1` build-frontier receipt MUST name exact derivation markers for the native make attempt, source-boundary diagnostic, pass1 bridge fallback, current source-frontier notes, and a retirement condition, and it MUST NOT claim full GCC 4.0 compiler correctness. A promoted native `cc1` source-frontier reduction MUST name the prior frontier, attempted bounded probe, exact derivation/source markers, observed new frontier or unchanged blocker, partial-only parity effect, and retirement condition, and it MUST NOT claim full GCC 4.0 compiler correctness.

#### Scenario: GCC 4.0 native demangle deeper nested-name slice is accepted [r[bootstrap.gcc.version-ladder.gcc40-native-demangle-deep-nested-name-slice]]

- GIVEN `bootstrap/gcc-4.0.ncl` contains a checked bounded libiberty demangle semantic marker for GCC 4.0
- AND a checked native-demangle receipt names the selected deeper nested-name shape, the derivation, schema version, bounded input/output contract, source markers, and digest or transcript evidence
- WHEN the bootstrap parity report evaluates `gcc.4.0`
- THEN the row may report evidence-backed `partial` with native demangle deeper nested-name slice evidence
- AND the row MUST continue blocking live-bootstrap and Guix until full native GCC 4.0 compiler and demangler correctness evidence exists

#### Scenario: GCC 4.0 native demangle deeper nested-name slice rejects stale boundary evidence [r[bootstrap.gcc.version-ladder.gcc40-native-demangle-deep-nested-name-slice-drift]]

- GIVEN a native-demangle receipt references a missing marker, stale digest, unsupported schema, unsupported selected shape, missing bounded input/output contract, forbidden stale marker, or unsupported parity effect
- WHEN the bootstrap parity report evaluates `gcc.4.0`
- THEN the row remains a blocker
- AND the row notes the specific failed native-demangle evidence check

#### Scenario: GCC 4.0 native demangle deeper nested-name slice cannot complete GCC 4.0 parity [r[bootstrap.gcc.version-ladder.gcc40-native-demangle-deep-nested-name-slice-no-overclaim]]

- GIVEN bounded libiberty demangle evidence exists for the selected deeper nested-name shape
- WHEN bootstrap parity claim gating evaluates live-bootstrap or Guix
- THEN `gcc.4.0` remains `partial`
- AND live-bootstrap and Guix requirements still fail closed until all remaining native compiler and demangler correctness blockers have evidence

#### Scenario: GCC 4.0 native demangle single-int-argument slice is accepted [r[bootstrap.gcc.version-ladder.gcc40-native-demangle-int-arg-slice]]

- GIVEN `bootstrap/gcc-4.0.ncl` contains a checked bounded libiberty demangle semantic marker for the selected single-`int` Itanium shape
- AND a checked native-demangle receipt names the selected single-`int` shape, the derivation, schema version, bounded input/output contract, source markers, and digest or transcript evidence
- WHEN the bootstrap parity report evaluates `gcc.4.0`
- THEN the row may report evidence-backed `partial` with native demangle single-`int` argument slice evidence
- AND the row MUST continue blocking live-bootstrap and Guix until full native GCC 4.0 compiler and demangler correctness evidence exists

#### Scenario: GCC 4.0 native demangle single-int-argument slice rejects stale boundary evidence [r[bootstrap.gcc.version-ladder.gcc40-native-demangle-int-arg-slice-drift]]

- GIVEN a native-demangle receipt references a missing marker, stale digest, unsupported schema, unsupported selected shape, missing bounded input/output contract, forbidden stale marker, or unsupported parity effect
- WHEN the bootstrap parity report evaluates `gcc.4.0`
- THEN the row remains a blocker
- AND the row notes the specific failed native-demangle evidence check

#### Scenario: GCC 4.0 native demangle single-int-argument slice cannot complete GCC 4.0 parity [r[bootstrap.gcc.version-ladder.gcc40-native-demangle-int-arg-slice-no-overclaim]]

- GIVEN bounded libiberty demangle evidence exists for the selected single-`int` argument shape
- WHEN bootstrap parity claim gating evaluates live-bootstrap or Guix
- THEN `gcc.4.0` remains `partial`
- AND live-bootstrap and Guix requirements still fail closed until all remaining native compiler and demangler correctness blockers have evidence


#### Scenario: GCC 4.0 native demangle single-char-argument slice is accepted [r[bootstrap.gcc.version-ladder.gcc40-native-demangle-char-arg-slice]]

- GIVEN `bootstrap/gcc-4.0.ncl` contains checked bounded libiberty demangle semantic markers for the selected single-`char` Itanium shape
- AND a checked native-demangle receipt names schema `mantle-gcc40-native-demangle-slice-v4`, the selected shape, source markers, bounded input/output contract, preserved regressions, rejected unsupported shapes, and transcript digest evidence
- WHEN the bootstrap parity report evaluates the `gcc.4.0` row
- THEN the row may report evidence-backed `partial` with native demangle single-`char` argument slice evidence
- AND the row MUST continue blocking live-bootstrap, Guix, and StageX until full native GCC 4.0 compiler and demangler correctness evidence exists

#### Scenario: GCC 4.0 native demangle single-char-argument slice rejects stale boundary evidence [r[bootstrap.gcc.version-ladder.gcc40-native-demangle-char-arg-slice-drift]]

- GIVEN the native demangle receipt references stale v3 single-`int` boundary markers or omits the v4 char markers
- WHEN the bootstrap parity report evaluates the `gcc.4.0` row
- THEN the row MUST fail closed instead of accepting stale demangle evidence
- AND the row notes the specific failed native-demangle evidence check

#### Scenario: GCC 4.0 native demangle single-char-argument slice cannot complete GCC 4.0 parity [r[bootstrap.gcc.version-ladder.gcc40-native-demangle-char-arg-slice-no-overclaim]]

- GIVEN bounded libiberty demangle evidence exists for the selected single-`char` argument shape
- WHEN live-bootstrap, Guix, or StageX parity is required
- THEN `gcc.4.0` MUST remain `partial` until native compiler/generator correctness and broader `cp-demangle` evidence are complete
- AND live-bootstrap, Guix, and StageX requirements still fail closed until all remaining native compiler and demangler correctness blockers have evidence


#### Scenario: GCC 4.0 native demangle single-long-argument slice is accepted [r[bootstrap.gcc.version-ladder.gcc40-native-demangle-long-arg-slice]]

- GIVEN `bootstrap/gcc-4.0.ncl` contains checked bounded libiberty demangle semantic markers for the selected single-`long` Itanium shape
- AND a checked native-demangle receipt names schema `mantle-gcc40-native-demangle-slice-v5`, the selected shape, source markers, bounded input/output contract, preserved regressions, rejected unsupported shapes, and transcript digest evidence
- WHEN the bootstrap parity report evaluates the `gcc.4.0` row
- THEN the row may report evidence-backed `partial` with native demangle single-`long` argument slice evidence
- AND the row MUST continue blocking live-bootstrap, Guix, and StageX until full native GCC 4.0 compiler and demangler correctness evidence exists

#### Scenario: GCC 4.0 native demangle single-long-argument slice rejects stale boundary evidence [r[bootstrap.gcc.version-ladder.gcc40-native-demangle-long-arg-slice-drift]]

- GIVEN the native demangle receipt references stale v4 single-`char` boundary markers or omits the v5 long markers
- WHEN the bootstrap parity report evaluates the `gcc.4.0` row
- THEN the row MUST fail closed instead of accepting stale demangle evidence
- AND the row notes the specific failed native-demangle evidence check

#### Scenario: GCC 4.0 native demangle single-long-argument slice cannot complete GCC 4.0 parity [r[bootstrap.gcc.version-ladder.gcc40-native-demangle-long-arg-slice-no-overclaim]]

- GIVEN bounded libiberty demangle evidence exists for the selected single-`long` argument shape
- WHEN live-bootstrap, Guix, or StageX parity is required
- THEN `gcc.4.0` MUST remain `partial` until native compiler/generator correctness and broader `cp-demangle` evidence are complete
- AND live-bootstrap, Guix, and StageX requirements still fail closed until all remaining native compiler and demangler correctness blockers have evidence

#### Scenario: GCC 4.0 native genoutput slice is accepted [r[bootstrap.gcc.version-ladder.gcc40-native-genoutput-slice]]

- GIVEN `bootstrap/gcc-4.0.ncl` contains a checked `genoutput` frontier boundary for GCC 4.0
- AND a checked native-generator receipt names `genoutput`, the derivation, schema version, bounded output contract, source markers, and digest or transcript evidence
- WHEN the bootstrap parity report evaluates `gcc.4.0`
- THEN the row may report evidence-backed `partial` with native `genoutput` slice evidence
- AND the row MUST continue blocking live-bootstrap and Guix until full native GCC 4.0 compiler and generator correctness evidence exists

#### Scenario: GCC 4.0 native genoutput slice rejects stale boundary evidence [r[bootstrap.gcc.version-ladder.gcc40-native-genoutput-slice-drift]]

- GIVEN a native-generator receipt for `genoutput` references a missing marker, stale digest, unsupported schema, unsupported selected generator, missing bounded output contract, or unsupported parity effect
- WHEN the bootstrap parity report evaluates `gcc.4.0`
- THEN the row remains a blocker
- AND the row notes the specific failed native-generator evidence check

#### Scenario: GCC 4.0 native genoutput slice cannot complete GCC 4.0 parity [r[bootstrap.gcc.version-ladder.gcc40-native-genoutput-slice-no-overclaim]]

- GIVEN bounded generator member evidence exists for `genoutput`
- WHEN bootstrap parity claim gating evaluates live-bootstrap or Guix
- THEN `gcc.4.0` remains `partial`
- AND live-bootstrap and Guix requirements still fail closed until all remaining native compiler and generator correctness blockers have evidence

#### Scenario: GCC 4.0 native cc1 logical/control-flow slice is accepted [r[bootstrap.gcc.version-ladder.gcc40-native-cc1-logical-slice]]

- GIVEN `bootstrap/gcc-4.0.ncl` contains a checked no-TinyCC-delegation marker for the selected bounded logical/control-flow input
- AND a checked native-cc1 receipt names schema `mantle-gcc40-native-cc1-arithmetic-v2`, selected slice `logical-boolean-control-flow-v2`, the derivation, bounded input, transcript, output digest, and preserved arithmetic regression
- WHEN the bootstrap parity report evaluates the `gcc.4.0` row
- THEN the row may report evidence-backed `partial` with native `cc1` logical/control-flow slice evidence
- AND the row MUST continue blocking live-bootstrap, Guix, and StageX until full native GCC 4.0 compiler correctness exists

#### Scenario: GCC 4.0 native cc1 logical/control-flow slice rejects stale or delegated evidence [r[bootstrap.gcc.version-ladder.gcc40-native-cc1-logical-slice-drift]]

- GIVEN the native-cc1 receipt references stale v1-only schema, omits the selected logical marker, has digest drift, contains TinyCC delegation markers in the selected transcript, or omits the arithmetic regression
- WHEN the bootstrap parity report evaluates the `gcc.4.0` row
- THEN the row MUST remain a blocker
- AND the row notes the specific failed native-cc1 evidence check

#### Scenario: GCC 4.0 native cc1 logical/control-flow slice cannot complete GCC 4.0 parity [r[bootstrap.gcc.version-ladder.gcc40-native-cc1-logical-slice-no-overclaim]]

- GIVEN bounded native `cc1` evidence exists for the selected arithmetic and logical/control-flow inputs
- WHEN live-bootstrap, Guix, or StageX parity is required
- THEN `gcc.4.0` MUST remain `partial` until full native compiler/generator/demangler correctness evidence is complete
- AND parity requirements still fail closed on the remaining GCC 4.0 blockers

#### Scenario: GCC 4.0 native cc1 local-variable assignment slice is accepted [r[bootstrap.gcc.version-ladder.gcc40-native-cc1-local-vars-slice]]

- GIVEN `bootstrap/gcc-4.0.ncl` contains a checked no-TinyCC-delegation marker for the selected bounded local-variable assignment input
- AND a checked native-cc1 receipt names the new schema, selected slice `local-variable-assignment-v3`, the derivation, bounded input, transcript, output digest, and preserved arithmetic and logical regressions
- WHEN the bootstrap parity report evaluates the `gcc.4.0` row
- THEN the row may report evidence-backed `partial` with native `cc1` local-variable assignment slice evidence
- AND the row MUST continue blocking live-bootstrap, Guix, and StageX until full native GCC 4.0 compiler correctness exists

#### Scenario: GCC 4.0 native cc1 local-variable assignment slice rejects stale or delegated evidence [r[bootstrap.gcc.version-ladder.gcc40-native-cc1-local-vars-slice-drift]]

- GIVEN the native-cc1 receipt references an older schema, omits the selected local-variable marker, has digest drift, contains TinyCC delegation markers in the selected transcript, or omits either preserved regression
- WHEN the bootstrap parity report evaluates the `gcc.4.0` row
- THEN the row MUST remain a blocker
- AND the row notes the specific failed native-cc1 evidence check

#### Scenario: GCC 4.0 native cc1 local-variable assignment slice cannot complete GCC 4.0 parity [r[bootstrap.gcc.version-ladder.gcc40-native-cc1-local-vars-slice-no-overclaim]]

- GIVEN bounded native `cc1` evidence exists for the selected arithmetic, logical/control-flow, and local-variable assignment inputs
- WHEN live-bootstrap, Guix, or StageX parity is required
- THEN `gcc.4.0` MUST remain `partial` until full native compiler/generator/demangler correctness evidence is complete
- AND parity requirements still fail closed on the remaining GCC 4.0 blockers

#### Scenario: GCC 4.0 native cc1 helper-call slice is accepted [r[bootstrap.gcc.version-ladder.gcc40-native-cc1-function-call-slice]]

- GIVEN `bootstrap/gcc-4.0.ncl` contains a checked no-TinyCC-delegation marker for the selected bounded helper-call input
- AND a checked native-cc1 receipt names the new schema, selected slice `function-call-v4`, the derivation, bounded input, transcript, output digest, and preserved arithmetic, logical, and local-variable regressions
- WHEN the bootstrap parity report evaluates the `gcc.4.0` row
- THEN the row may report evidence-backed `partial` with native `cc1` helper-call slice evidence
- AND the row MUST continue blocking live-bootstrap, Guix, and StageX until full native GCC 4.0 compiler correctness exists

#### Scenario: GCC 4.0 native cc1 helper-call slice rejects stale or delegated evidence [r[bootstrap.gcc.version-ladder.gcc40-native-cc1-function-call-slice-drift]]

- GIVEN the native-cc1 receipt references an older schema, omits the selected helper-call marker, has digest drift, contains TinyCC delegation markers in the selected transcript, or omits any preserved regression
- WHEN the bootstrap parity report evaluates the `gcc.4.0` row
- THEN the row MUST remain a blocker
- AND the row notes the specific failed native-cc1 evidence check

#### Scenario: GCC 4.0 native cc1 helper-call slice cannot complete GCC 4.0 parity [r[bootstrap.gcc.version-ladder.gcc40-native-cc1-function-call-slice-no-overclaim]]

- GIVEN bounded native `cc1` evidence exists for the selected arithmetic, logical/control-flow, local-variable assignment, and helper-call inputs
- WHEN live-bootstrap, Guix, or StageX parity is required
- THEN `gcc.4.0` MUST remain `partial` until full native compiler/generator/demangler correctness evidence is complete
- AND parity requirements still fail closed on the remaining GCC 4.0 blockers

#### Scenario: GCC 4.0 native cc1 array-index slice is accepted [r[bootstrap.gcc.version-ladder.gcc40-native-cc1-array-slice]]

- GIVEN `bootstrap/gcc-4.0.ncl` contains a checked no-TinyCC-delegation marker for the selected bounded array-index input
- AND a checked native-cc1 receipt names schema `mantle-gcc40-native-cc1-arithmetic-v5`, selected slice `array-index-v5`, the derivation, bounded input, transcript, output digest, and preserved arithmetic, logical, local-variable, and helper-call regressions
- WHEN the bootstrap parity report evaluates the `gcc.4.0` row
- THEN the row may report evidence-backed `partial` with native `cc1` array-index slice evidence
- AND the row MUST continue blocking live-bootstrap, Guix, and StageX until full native GCC 4.0 compiler correctness exists

#### Scenario: GCC 4.0 native cc1 array-index slice rejects stale or delegated evidence [r[bootstrap.gcc.version-ladder.gcc40-native-cc1-array-slice-drift]]

- GIVEN the native-cc1 receipt references an older schema, omits the selected array-index marker, has digest drift, contains TinyCC delegation markers in the selected transcript, or omits any preserved regression
- WHEN the bootstrap parity report evaluates the `gcc.4.0` row
- THEN the row MUST remain a blocker
- AND the row notes the specific failed native-cc1 evidence check

#### Scenario: GCC 4.0 native cc1 array-index slice cannot complete GCC 4.0 parity [r[bootstrap.gcc.version-ladder.gcc40-native-cc1-array-slice-no-overclaim]]

- GIVEN bounded native `cc1` evidence exists for the selected arithmetic, logical/control-flow, local-variable assignment, helper-call, and array-index inputs
- WHEN live-bootstrap, Guix, or StageX parity is required
- THEN `gcc.4.0` MUST remain `partial` until full native compiler/generator/demangler correctness evidence is complete
- AND parity requirements still fail closed on the remaining GCC 4.0 blockers

#### Scenario: GCC 4.0 native cc1 struct-field slice is accepted [r[bootstrap.gcc.version-ladder.gcc40-native-cc1-struct-field-slice]]

- GIVEN `bootstrap/gcc-4.0.ncl` contains a checked no-TinyCC-delegation marker for the selected bounded struct-field input
- AND a checked native-cc1 receipt names schema `mantle-gcc40-native-cc1-arithmetic-v6`, selected slice `struct-field-v6`, the derivation, bounded input, transcript, output digest, and preserved arithmetic, logical, local-variable, helper-call, and array-index regressions
- WHEN the bootstrap parity report evaluates the `gcc.4.0` row
- THEN the row may report evidence-backed `partial` with native `cc1` struct-field slice evidence
- AND the row MUST continue blocking live-bootstrap, Guix, and StageX until full native GCC 4.0 compiler correctness exists

#### Scenario: GCC 4.0 native cc1 struct-field slice rejects stale or delegated evidence [r[bootstrap.gcc.version-ladder.gcc40-native-cc1-struct-field-slice-drift]]

- GIVEN the native-cc1 receipt references an older schema, omits the selected struct-field marker, has digest drift, contains TinyCC delegation markers in the selected transcript, or omits any preserved regression
- WHEN the bootstrap parity report evaluates the `gcc.4.0` row
- THEN the row MUST remain a blocker
- AND the row notes the specific failed native-cc1 evidence check

#### Scenario: GCC 4.0 native cc1 struct-field slice cannot complete GCC 4.0 parity [r[bootstrap.gcc.version-ladder.gcc40-native-cc1-struct-field-slice-no-overclaim]]

- GIVEN bounded native `cc1` evidence exists for the selected arithmetic, logical/control-flow, local-variable assignment, helper-call, array-index, and struct-field inputs
- WHEN live-bootstrap, Guix, or StageX parity is required
- THEN `gcc.4.0` MUST remain `partial` until full native compiler/generator/demangler correctness evidence is complete
- AND parity requirements still fail closed on the remaining GCC 4.0 blockers

#### Scenario: GCC 4.0 native cc1 pointer-deref slice is accepted [r[bootstrap.gcc.version-ladder.gcc40-native-cc1-pointer-slice]]

- GIVEN `bootstrap/gcc-4.0.ncl` contains a checked no-TinyCC-delegation marker for the selected bounded pointer-deref input
- AND a checked native-cc1 receipt names schema `mantle-gcc40-native-cc1-arithmetic-v7`, selected slice `pointer-deref-v7`, the derivation, bounded input, transcript, output digest, and preserved arithmetic, logical, local-variable, helper-call, array-index, and struct-field regressions
- WHEN the bootstrap parity report evaluates the `gcc.4.0` row
- THEN the row may report evidence-backed `partial` with native `cc1` pointer-deref slice evidence
- AND the row MUST continue blocking live-bootstrap, Guix, and StageX until full native GCC 4.0 compiler correctness exists

#### Scenario: GCC 4.0 native cc1 pointer-deref slice rejects stale or delegated evidence [r[bootstrap.gcc.version-ladder.gcc40-native-cc1-pointer-slice-drift]]

- GIVEN the native-cc1 receipt references an older schema, omits the selected pointer-deref marker, has digest drift, contains TinyCC delegation markers in the selected transcript, or omits any preserved regression
- WHEN the bootstrap parity report evaluates the `gcc.4.0` row
- THEN the row MUST remain a blocker
- AND the row notes the specific failed native-cc1 evidence check

#### Scenario: GCC 4.0 native cc1 pointer-deref slice cannot complete GCC 4.0 parity [r[bootstrap.gcc.version-ladder.gcc40-native-cc1-pointer-slice-no-overclaim]]

- GIVEN bounded native `cc1` evidence exists for the selected arithmetic, logical/control-flow, local-variable assignment, helper-call, array-index, struct-field, and pointer-deref inputs
- WHEN live-bootstrap, Guix, or StageX parity is required
- THEN `gcc.4.0` MUST remain `partial` until full native compiler/generator/demangler correctness evidence is complete
- AND parity requirements still fail closed on the remaining GCC 4.0 blockers

#### Scenario: GCC 4.0 native cc1 build-frontier receipt is accepted [r[bootstrap.gcc.version-ladder.gcc40-native-cc1-build-frontier]]

- GIVEN `bootstrap/gcc-4.0.ncl` contains exact markers for the native `cc1` source-build attempt, the TinyCC/Mes source-boundary diagnostic, and the pass1 bridge fallback
- AND a checked native-cc1 build-frontier receipt names the derivation, schema, required markers, source-frontier notes, partial-only parity effect, and retirement condition
- WHEN the bootstrap parity report evaluates the `gcc.4.0` row
- THEN the row may report evidence-backed `partial` with native source-build frontier evidence
- AND the row MUST continue blocking live-bootstrap, Guix, and StageX until full native GCC 4.0 compiler correctness exists

#### Scenario: GCC 4.0 native cc1 build-frontier receipt rejects stale evidence [r[bootstrap.gcc.version-ladder.gcc40-native-cc1-build-frontier-drift]]

- GIVEN the build-frontier receipt references missing derivation markers, stale source-frontier notes, an unsupported schema, missing retirement condition, or unsupported parity effect
- WHEN the bootstrap parity report evaluates the `gcc.4.0` row
- THEN the row MUST remain a blocker
- AND the row notes the specific failed native source-build frontier evidence check

#### Scenario: GCC 4.0 native cc1 build-frontier receipt cannot complete GCC 4.0 parity [r[bootstrap.gcc.version-ladder.gcc40-native-cc1-build-frontier-no-overclaim]]

- GIVEN native `cc1` build-frontier receipt evidence exists alongside bounded installed-`cc1` semantic slices
- WHEN live-bootstrap, Guix, or StageX parity is required
- THEN `gcc.4.0` MUST remain `partial` until full native compiler/generator/demangler correctness evidence is complete
- AND parity requirements still fail closed on the remaining GCC 4.0 blockers

#### Scenario: GCC 4.0 native cc1 source frontier reduction is accepted [r[bootstrap.gcc.version-ladder.gcc40-native-cc1-source-frontier-reduction]]

- GIVEN `bootstrap/gcc-4.0.ncl` contains exact markers for a bounded native `cc1` source-build probe near the current TinyCC/Mes frontier
- AND checked source-frontier evidence names the prior frontier, attempted command or patch scope, observed frontier result, exact markers, checked diagnostic derivation markers, partial-only parity effect, and retirement condition
- WHEN the bootstrap parity report evaluates the `gcc.4.0` row
- THEN the row may report evidence-backed `partial` with updated native source-frontier evidence
- AND the row MUST continue blocking live-bootstrap, Guix, and StageX until full native GCC 4.0 compiler correctness exists

#### Scenario: GCC 4.0 native cc1 source frontier evidence rejects stale or missing results [r[bootstrap.gcc.version-ladder.gcc40-native-cc1-source-frontier-reduction-drift]]

- GIVEN the source-frontier evidence references missing derivation markers, omits the prior frontier, omits the observed frontier result, uses an unsupported schema/status, names diagnostic markers absent from the diagnostic derivation, or lacks a retirement condition
- WHEN the bootstrap parity report evaluates the `gcc.4.0` row
- THEN the row MUST remain a blocker
- AND the row notes the specific failed native source-frontier evidence check

#### Scenario: GCC 4.0 native cc1 source frontier evidence cannot complete GCC 4.0 parity [r[bootstrap.gcc.version-ladder.gcc40-native-cc1-source-frontier-reduction-no-overclaim]]

- GIVEN native `cc1` source-frontier reduction evidence exists alongside bounded installed-`cc1` semantic slices and build-frontier metadata
- WHEN live-bootstrap, Guix, or StageX parity is required
- THEN `gcc.4.0` MUST remain `partial` until full native compiler/generator/demangler correctness evidence is complete
- AND parity requirements still fail closed on the remaining GCC 4.0 blockers

#### Scenario: GCC 4.0 native cc1 c-parse generated-header frontier is accepted [r[bootstrap.gcc.version-ladder.gcc40-native-cc1-cparse-generated-header-frontier]]

- GIVEN `bootstrap/evidence/gcc-4.0-native-cc1-build-frontier.json` uses source-frontier schema `mantle-gcc40-native-cc1-source-frontier-reduction-v5`
- AND the evidence records the focused `c-parse.o` make attempt advancing beyond the v4 autohost macro seam with six targeted undefines
- AND the evidence names exact diagnostic markers for representative `insn-modes.h`, `machmode.h`, and `tree.h` generated-header prefix outcomes
- WHEN the bootstrap parity report evaluates the `gcc.4.0` row
- THEN the source-frontier evidence check passes
- AND `gcc.4.0` remains evidence-backed `partial` without completing live-bootstrap, Guix, or StageX parity

#### Scenario: GCC 4.0 native cc1 c-parse generated-header frontier rejects stale evidence [r[bootstrap.gcc.version-ladder.gcc40-native-cc1-cparse-generated-header-frontier-drift]]

- GIVEN the source-frontier evidence uses a stale schema, still claims the copied `fd_bad`, fdopen/output-return, or autohost macro seam is the active remaining frontier, omits the generated-header diagnostic markers, or references diagnostic markers absent from the diagnostic derivation
- WHEN the bootstrap parity report evaluates the `gcc.4.0` row
- THEN the row MUST remain a blocker
- AND the row notes the specific failed native source-frontier evidence check

#### Scenario: GCC 4.0 native cc1 c-parse full generated-header frontier is accepted [r[bootstrap.gcc.version-ladder.gcc40-native-cc1-cparse-full-generated-header-frontier]]

- GIVEN `bootstrap/evidence/gcc-4.0-native-cc1-build-frontier.json` uses source-frontier schema `mantle-gcc40-native-cc1-source-frontier-reduction-v6`
- AND the evidence records the focused `c-parse.o` make attempt advancing beyond v5 representative generated-header prefix probes into later `machmode.h`, wider `tree.h`, builtin enum, and real `c-parse.o` make probes
- AND the evidence names exact diagnostic markers from `bootstrap/diag-gcc40-c-parse-boundary.ncl` for those probes
- WHEN the bootstrap parity report evaluates the `gcc.4.0` row
- THEN the source-frontier evidence check passes
- AND `gcc.4.0` remains evidence-backed `partial` without completing live-bootstrap, Guix, or StageX parity

#### Scenario: GCC 4.0 native cc1 c-parse full generated-header frontier rejects stale evidence [r[bootstrap.gcc.version-ladder.gcc40-native-cc1-cparse-full-generated-header-frontier-drift]]

- GIVEN the source-frontier evidence uses a stale schema, omits the later generated-header sweep diagnostic markers, omits the real `c-parse.o` make failure marker, or references diagnostic markers absent from the diagnostic derivation
- WHEN the bootstrap parity report evaluates the `gcc.4.0` row
- THEN the row MUST remain a blocker
- AND the row notes the specific failed native source-frontier evidence check

#### Scenario: GCC 4.0 native cc1 c-parse make-error frontier is accepted [r[bootstrap.gcc.version-ladder.gcc40-native-cc1-cparse-make-error-frontier]]

- GIVEN `bootstrap/evidence/gcc-4.0-native-cc1-build-frontier.json` uses source-frontier schema `mantle-gcc40-native-cc1-source-frontier-reduction-v7`
- AND the evidence records the focused `c-parse.o` make target's nonzero rc and bounded diagnostic output tail after the v6 generated-header sweep
- AND the evidence names exact diagnostic markers from `bootstrap/diag-gcc40-c-parse-boundary.ncl` for the captured make-error boundary
- WHEN the bootstrap parity report evaluates the `gcc.4.0` row
- THEN the source-frontier evidence check passes
- AND `gcc.4.0` remains evidence-backed `partial` without completing live-bootstrap, Guix, or StageX parity

#### Scenario: GCC 4.0 native cc1 c-parse make-error frontier rejects stale evidence [r[bootstrap.gcc.version-ladder.gcc40-native-cc1-cparse-make-error-frontier-drift]]

- GIVEN the source-frontier evidence uses a stale schema, omits the captured make rc/output-tail markers, omits the real `c-parse.o` make failure observation, or references diagnostic markers absent from the diagnostic derivation
- WHEN the bootstrap parity report evaluates the `gcc.4.0` row
- THEN the row MUST remain a blocker
- AND the row notes the specific failed native source-frontier evidence check

#### Scenario: GCC 4.0 native cc1 c-parse compact make frontier is accepted [r[bootstrap.gcc.version-ladder.gcc40-native-cc1-cparse-compact-make-frontier]]

- GIVEN `bootstrap/evidence/gcc-4.0-native-cc1-build-frontier.json` uses source-frontier schema `mantle-gcc40-native-cc1-source-frontier-reduction-v8`
- AND the evidence records the diagnostic derivation compaction marker plus the focused `c-parse.o` make rc/output-tail markers after the v7 make-error boundary
- AND the diagnostic derivation no longer carries the archived v5/v6 generated-header sweep matrix inline
- WHEN the bootstrap parity report evaluates the `gcc.4.0` row
- THEN the source-frontier evidence check passes
- AND `gcc.4.0` remains evidence-backed `partial` without completing live-bootstrap, Guix, or StageX parity

#### Scenario: GCC 4.0 native cc1 c-parse compact make frontier rejects stale evidence [r[bootstrap.gcc.version-ladder.gcc40-native-cc1-cparse-compact-make-frontier-drift]]

- GIVEN the source-frontier evidence uses a stale schema, omits the compacted-matrix marker, omits the captured make rc/output-tail markers, claims promotion, or reintroduces the archived generated-header sweep matrix into diagnostic markers
- WHEN the bootstrap parity report evaluates the `gcc.4.0` row
- THEN the row MUST remain a blocker
- AND the row notes the specific failed native source-frontier evidence check

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
- WHEN `mantle self-build` runs
- THEN stage1 and stage2 binaries are byte-identical

### Requirement: Source tarballs pinned by hash

Every source tarball fetched during the bootstrap chain MUST be pinned by a
content hash in its `.ncl` file. The hash MUST use the same algorithm as
`mantle.fetchTarball` (currently SHA-256 for Nix compatibility).

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
Mantle MUST track the live-bootstrap part `bootstrap-seeds through mescc-tools-extra` as an independent bootstrap change bound to `bootstrap/stage0-posix.ncl`.
ID: bootstrap.part.stage0.posix

The part scope MUST include only the source pins, patches, derivation logic, output contract, and evidence needed for `bootstrap/stage0-posix.ncl` and direct predecessor compatibility. Broader chain validation MAY depend on this part, but MUST NOT replace this part's own build and smoke evidence.

#### Scenario: Part evidence is isolated

- GIVEN implementation work touches `bootstrap/stage0-posix.ncl`
- WHEN the part is marked complete
- THEN the change records source-pin audit evidence for `bootstrap/stage0-posix.ncl`
- AND it records a successful `mantle build bootstrap/stage0-posix.ncl` transcript
- AND it records a smoke check for the produced output contract

#### Scenario: Downstream blockers stay local

- GIVEN a downstream bootstrap stage fails after `bootstrap/stage0-posix.ncl` succeeds
- WHEN this part's evidence is reviewed
- THEN the downstream failure does not invalidate this part's completed evidence
- AND a separate part change tracks the downstream failure

### Requirement: Live-bootstrap part mes 0.27 is independently tracked
Mantle MUST track the live-bootstrap part `mes 0.27` as an independent bootstrap change bound to `bootstrap/mes.ncl`.
ID: bootstrap.part.mes.0.27

The part scope MUST include only the source pins, patches, derivation logic, output contract, and evidence needed for `bootstrap/mes.ncl` and direct predecessor compatibility. Broader chain validation MAY depend on this part, but MUST NOT replace this part's own build and smoke evidence.

#### Scenario: Part evidence is isolated

- GIVEN implementation work touches `bootstrap/mes.ncl`
- WHEN the part is marked complete
- THEN the change records source-pin audit evidence for `bootstrap/mes.ncl`
- AND it records a successful `mantle build bootstrap/mes.ncl` transcript
- AND it records a smoke check for the produced output contract

#### Scenario: Downstream blockers stay local

- GIVEN a downstream bootstrap stage fails after `bootstrap/mes.ncl` succeeds
- WHEN this part's evidence is reviewed
- THEN the downstream failure does not invalidate this part's completed evidence
- AND a separate part change tracks the downstream failure

### Requirement: Tinycc Mes self-compile blocker is resolved
Mantle MUST resolve the `BufferedFile`/first-self-compile blocker before `live-part-tinycc-0-9-26` can claim successful build evidence.
ID: bootstrap.part.tinycc.0.9.26.selfcompile

#### Scenario: Full tinycc output contract is required evidence

- GIVEN `bootstrap/tinycc-mes.ncl` builds `tcc-mes`
- WHEN the blocker is marked resolved
- THEN the evidence MUST show `tcc-mes` compiles at least `tcc-boot0` without segfaulting
- AND `mantle build bootstrap/tinycc-mes.ncl` MUST finish successfully
- AND the produced output MUST include executable `bin/tcc` and `bin/tcc-0.9.26`
- AND the produced compiler MUST compile a trivial C program
- AND `tcc-mes -version` alone MUST NOT be accepted as the smoke boundary

### Requirement: Live-bootstrap part tinycc 0.9.26 is independently tracked
Mantle MUST track the live-bootstrap part `tinycc 0.9.26` as an independent bootstrap change bound to `bootstrap/tinycc-mes.ncl`.
ID: bootstrap.part.tinycc.0.9.26

The part scope MUST include only the source pins, patches, derivation logic, output contract, and evidence needed for `bootstrap/tinycc-mes.ncl` and direct predecessor compatibility. Broader chain validation MAY depend on this part, but MUST NOT replace this part's own build and smoke evidence.

#### Scenario: Part evidence is isolated

- GIVEN implementation work touches `bootstrap/tinycc-mes.ncl`
- WHEN the part is marked complete
- THEN the change records source-pin audit evidence for `bootstrap/tinycc-mes.ncl`
- AND it records a successful `mantle build bootstrap/tinycc-mes.ncl` transcript
- AND it records a smoke check for the produced output contract

#### Scenario: Downstream blockers stay local

- GIVEN a downstream bootstrap stage fails after `bootstrap/tinycc-mes.ncl` succeeds
- WHEN this part's evidence is reviewed
- THEN the downstream failure does not invalidate this part's completed evidence
- AND a separate part change tracks the downstream failure

### Requirement: Live-bootstrap part tinycc 0.9.27 is independently tracked
Mantle MUST track the live-bootstrap part `tinycc 0.9.27` as an independent bootstrap change bound to `bootstrap/tinycc.ncl`.
ID: bootstrap.part.tinycc.0.9.27

The part scope MUST include only the source pins, patches, derivation logic, output contract, and evidence needed for `bootstrap/tinycc.ncl` and direct predecessor compatibility. Broader chain validation MAY depend on this part, but MUST NOT replace this part's own build and smoke evidence.

#### Scenario: Part evidence is isolated

- GIVEN implementation work touches `bootstrap/tinycc.ncl`
- WHEN the part is marked complete
- THEN the change records source-pin audit evidence for `bootstrap/tinycc.ncl`
- AND it records a successful `mantle build bootstrap/tinycc.ncl` transcript
- AND it records a smoke check for the produced output contract

#### Scenario: Downstream blockers stay local

- GIVEN a downstream bootstrap stage fails after `bootstrap/tinycc.ncl` succeeds
- WHEN this part's evidence is reviewed
- THEN the downstream failure does not invalidate this part's completed evidence
- AND a separate part change tracks the downstream failure

### Requirement: TinyCC 0.9.27 compiles trivial C on amd64
Mantle MUST build `bootstrap/tinycc.ncl` into a TinyCC 0.9.27 output that can compile a trivial C source file to an object on amd64.
ID: bootstrap.part.tinycc.0.9.27.amd64.compile

The output MUST include `bin/tcc`, report `tcc version 0.9.27 (x86_64 Linux)`, complete `tcc -c hello.c -o hello.o` within the bounded smoke timeout, and produce a non-empty object file. The compiler MUST reject malformed C with a controlled nonzero exit rather than a hang, timeout, or segmentation fault. Completion evidence MUST include a source-pin audit transcript, successful `mantle build bootstrap/tinycc.ncl` transcript, version smoke transcript, positive object-compile transcript, malformed-input negative transcript, and host-leakage scan transcript.

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
Mantle MUST build the Mes-hosted TinyCC 0.9.26 predecessor so x86_64 constant shift expressions emit nonzero immediate shift counts.
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
- GIVEN the Mantle-built `bootstrap/tinycc.ncl` output on amd64
- WHEN `tcc -c hello.c` and `tcc -static -o hello hello.o` run in a Mantle diagnostic derivation
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

Mantle MUST provide decision evidence before pivoting Make 3.82 runtime validation from the current amd64 TinyCC/Mes repair path to an i386-first live-bootstrap path.

#### Scenario: Reference audit is not runtime proof [r[bootstrap.i386-live-bootstrap-spike.reference-audit]]

- GIVEN a StageX or upstream live-bootstrap reference sequence that builds on `linux/386`
- WHEN Mantle records that sequence as evidence
- THEN Mantle MUST classify it as reference evidence only until a Mantle-local proof target runs.

#### Scenario: Pivot decision has explicit criteria [r[bootstrap.i386-live-bootstrap-spike.decision]]

- GIVEN the amd64 Make 3.82 path remains blocked by runtime segfaults
- WHEN the i386 proof target is evaluated
- THEN Mantle MUST record whether to pivot, continue amd64 repair, or carry both paths with explicit scope boundaries.

<!-- synced from openspec change: repair-i386-tinycc26-emission -->

### Requirement: i386 TinyCC 0.9.26 emission diagnostics [r[bootstrap.i386-tinycc26-emission.diagnostics]]

Mantle MUST isolate the i386 TinyCC 0.9.26 output-generation blocker before using the i386 path as evidence for Make 3.82 runtime validation.

#### Scenario: Emission stages are separated [r[bootstrap.i386-tinycc26-emission.diagnostics.stages]]

- GIVEN a x86_64-hosted/i386-targeting TinyCC 0.9.26 built by Mantle
- WHEN Mantle evaluates the i386 emission proof
- THEN it MUST record version, assemble-only, link-from-assembly, link-from-object, and run-output results separately.

#### Scenario: Diagnostic failure does not imply production pivot [r[bootstrap.i386-tinycc26-emission.diagnostics.no-production-pivot]]

- GIVEN any i386 emission stage fails or segfaults
- WHEN the diagnostic evidence is recorded
- THEN Mantle MUST keep production Make 3.82 validation blocked rather than claiming the i386 path is production-ready.

### Requirement: i386 TinyCC 0.9.26 repair decision [r[bootstrap.i386-tinycc26-emission.decision]]

Mantle MUST record the next repair target after the diagnostic stage identifies where `tcc26-i386` fails.

#### Scenario: Next repair target is evidence-backed [r[bootstrap.i386-tinycc26-emission.decision.target]]

- GIVEN diagnostic transcript evidence for each emission stage
- WHEN choosing the next implementation slice
- THEN Mantle MUST identify whether the next target is assembly parsing, object emission, static linking, ELF materialization, or runtime execution.

<!-- ADDED Requirements -->

### Requirement: Binutils-TCC chain implementation

Mantle MUST build `bootstrap/binutils-tcc.ncl` from chain-internal TinyCC-era and post-musl derivations without using host compiler, host libc, host shell tools, or the legacy musl.cc provider. The binutils-tcc stage MUST NOT satisfy live-bootstrap or Guix parity until reproducible evidence proves the produced assembler/linker/archive tools and records absence of host fallback.
ID: bootstrap.binutils.tcc.chain

The chain MUST implement the scoped ladder groups named by the proposal: early tcc-hosted utilities (`bzip2`, `coreutils-5.0`, `oyacc`, `bash-2.05b`), first musl/tcc rebuilds, post-musl text/parser tools (`grep`, rebuilt `sed`, rebuilt `bzip2`, `m4`, Heirloom devtools, `flex`, `bison`), diffutils/coreutils/gawk, Perl/autoconf/automake/libtool, and binutils 2.30. The chain MUST pin every source, carried patch, and generated artifact with URL or repository path, digest, and provenance at the first consuming derivation. Validation transcripts MUST record command, provider selection, exit status, output path, fallback status/event marker, and placeholder rejection result. Validation MUST prove post-musl `m4`, `flex`, `bison`, and `grep` link against musl, and MUST prove binutils 2.30 can assemble a trivial ELF object for the gcc-4.0.4 transition.

#### Scenario: Placeholder is replaced

- GIVEN `bootstrap/binutils-tcc.ncl` is evaluated
- WHEN the derivation builds
- THEN it does not emit `ERROR: binutils-tcc.ncl is a placeholder`
- AND it produces working `as`, `ld`, `ar`, `ranlib`, `nm`, and `objcopy` tools

#### Scenario: Placeholder is replaced with evidence [r[bootstrap.binutils.tcc.chain.evidence-promotion]]

- GIVEN `mantle bootstrap parity-report` evaluates the `binutils.tcc` row
- WHEN the row is considered for live-bootstrap or Guix parity
- THEN it remains `placeholder` or `partial` unless a checked transcript proves `as`, `ld`, `ar`, `ranlib`, `nm`, and `objcopy` from `bootstrap/binutils-tcc.ncl`
- AND the transcript records the build command, output path, provider kind, fallback markers, and smoke command exit statuses

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

Mantle MUST build `bootstrap/gcc-4.0.ncl` as gcc 4.0.4 C and C++ compiler outputs using only the chain-internal TinyCC/musl/binutils 2.30 inputs.
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

Mantle MUST build `bootstrap/gcc-4.7.ncl` as gcc 4.7.4 C and C++ compiler outputs using only chain-internal gcc-4.0.4-era inputs.
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

The chain MUST replace `bootstrap/binutils-tcc.ncl`, `bootstrap/gcc-4.0.ncl`, `bootstrap/gcc-4.7.ncl`, `bootstrap/gcc-10.ncl`, `bootstrap/musl-full.ncl`, `bootstrap/binutils-full.ncl`, and `bootstrap/seed-full.ncl` placeholder derivations before they can satisfy bootstrap completion status. Stage validation MUST record command transcripts for the ordered inventory (`bootstrap/stage0-posix.ncl`, `bootstrap/mes.ncl`, `bootstrap/tinycc.ncl`, `bootstrap/binutils-tcc.ncl`, `bootstrap/gcc-4.0.ncl`, `bootstrap/gcc-4.7.ncl`, `bootstrap/gcc-10.ncl`, `bootstrap/musl-full.ncl`, `bootstrap/binutils-full.ncl`, `bootstrap/seed-full.ncl`).

#### Scenario: Full musl/binutils provider-contract receipt is missing [r[bootstrap.source.chain.implementation.full-musl-binutils-missing-receipt]]

- GIVEN `bootstrap/musl-full.ncl` and `bootstrap/binutils-full.ncl` exist
- BUT `bootstrap/evidence/full-musl-binutils-provider-contract.json` is absent
- WHEN the bootstrap parity report evaluates `full-musl-binutils`
- THEN the row remains a live-bootstrap and Guix blocker
- AND the row notes identify the missing provider-contract receipt

#### Scenario: Full musl/binutils provider-contract receipt is evidence-backed partial [r[bootstrap.source.chain.implementation.full-musl-binutils-contract-partial]]

- GIVEN `bootstrap/evidence/full-musl-binutils-provider-contract.json` has the expected schema, derivation paths, `contract-only` status, required musl/binutils derivation markers, and explicit partial parity effect
- AND every required musl marker appears in `bootstrap/musl-full.ncl`
- AND every required binutils marker appears in `bootstrap/binutils-full.ncl`
- WHEN the bootstrap parity report evaluates `full-musl-binutils`
- THEN the row may report evidence-backed `partial`
- AND it MUST NOT report `complete` or unblock live-bootstrap or Guix parity until full toolchain correctness and source-root proof exist

#### Scenario: Full musl/binutils provider-contract marker drift fails closed [r[bootstrap.source.chain.implementation.full-musl-binutils-marker-drift]]

- GIVEN the full musl/binutils provider-contract receipt requires a marker that no longer appears in either derivation
- WHEN the bootstrap parity report evaluates `full-musl-binutils`
- THEN the row remains a blocker
- AND the row notes identify the missing marker and affected derivation

### Requirement: Full-source bootstrap claim requires evidence

Mantle MUST withhold the full-source bootstrap claim until the source-root manifest validates for the full-source profile, the lineage manifest validates for the StageX-class profile, every named live-bootstrap placeholder is replaced, source-built stage transcripts exist, and self-build proof completes with the selected source-built provider.
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

Mantle MUST require the full StageX-class evidence tuple before this live-bootstrap source-chain change can satisfy any StageX-class bootstrap claim.
ID: bootstrap.stagex.selfbuild.proof

The proof metadata MUST bind selected provider kind, audited seed digest, lineage manifest digest, stage graph digest, normalized provider digest, staged source digest, stage1 and stage2 mantle binary digests, bootstrap-tool digests, protected execution audit digest when used, final proof bundle digest, canonical reproducibility report digest, and explicit `self-build-proof: fallback-event=<kind>` markers or `self-build-proof: fallback-event=none`. The proof MUST fail closed when any named live-bootstrap placeholder remains, when the legacy provider is selected for a StageX claim, when host-bwrap or checkout/source-discovery fallback appears, or when forbidden host executables run during the protected stage. The bootstrap parity report MUST require a checked provider-kind linkage receipt for `mantle.self-build`; the receipt MUST use the schema `crunch-self-build-provider-kind-linkage-v1` and MUST prove that `proof_identity.selected_provider_kind`, `proof_linkage.selected_provider_kind`, and `prerequisites.provider_kind` are identical closed provider-kind values. The bootstrap parity report MUST also require a checked StageX lineage provider receipt for `seed-full.stagex-lineage`; scaffold receipts MUST be explicitly marked `lineage_receipt_status = scaffold-only`, MUST serialize `provider_kind = stagex-lineage`, MUST record digest-shaped audited seed, lineage manifest, stage graph, and normalized provider fields, and MUST record no fallback events. The bootstrap parity report MUST consume real self-build proof evidence for the `crunch.self-build` row only from a validated evidence bundle that links a release manifest, `mantle-deterministic-proof-receipt-v1` deterministic proof receipt, `mantle-proof-sandbox-v1:*` sandbox evidence, release verify receipt with deterministic status `eligible`, and optional portable summary artifacts. Such evidence MAY make `crunch.self-build` evidence-backed partial for the selected provider kind, but MUST NOT satisfy Guix or StageX parity unless the selected provider kind and all axis-specific source-root or StageX lineage evidence requirements are also satisfied.

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

#### Scenario: Release evidence binds selected provider kind [r[bootstrap.stagex.selfbuild.proof.provider-linkage]]

- GIVEN a full self-hosting proof bundle whose prerequisites name a selected provider kind
- WHEN release evidence is created from that proof bundle
- THEN `proof_linkage.selected_provider_kind` equals the proof bundle's selected provider kind
- AND verification fails if either side is missing, unknown, or mismatched

#### Scenario: Parity rejects missing provider-kind linkage receipt [r[bootstrap.stagex.selfbuild.proof.parity-missing-receipt]]

- GIVEN `bootstrap/crunch.ncl` exists
- BUT `bootstrap/evidence/crunch-self-build-provider-kind-linkage.json` is absent
- WHEN `mantle bootstrap parity-report --require guix` or `--require stagex` runs
- THEN `mantle.self-build` remains a blocker
- AND the row notes identify the missing provider-kind linkage receipt

#### Scenario: Parity accepts matching provider-kind linkage receipt as partial evidence [r[bootstrap.stagex.selfbuild.proof.parity-matching-receipt]]

- GIVEN a checked self-build provider-kind linkage receipt records the same closed provider kind in proof identity, proof linkage, and prerequisites
- WHEN the parity report loads that evidence
- THEN the `mantle.self-build` row may report evidence-backed `partial`
- AND it MUST NOT report `complete` until the full self-build proof and axis-specific evidence are present

#### Scenario: Parity rejects missing StageX lineage receipt [r[bootstrap.stagex.selfbuild.proof.stagex-lineage-missing-receipt]]

- GIVEN no StageX lineage provider receipt exists
- WHEN the bootstrap parity report evaluates `seed-full.stagex-lineage`
- THEN the row remains a StageX blocker
- AND the row notes identify the missing lineage receipt

#### Scenario: Parity accepts scaffold lineage receipt only as partial evidence [r[bootstrap.stagex.selfbuild.proof.stagex-lineage-scaffold-partial]]

- GIVEN a StageX lineage provider receipt has schema `crunch-stagex-lineage-provider-receipt-v1`
- AND it records `provider_kind = stagex-lineage`, `lineage_receipt_status = scaffold-only`, digest-shaped lineage fields, and no fallback events
- WHEN the bootstrap parity report evaluates `seed-full.stagex-lineage`
- THEN the row may report evidence-backed `partial`
- AND it MUST NOT report `complete` or unblock StageX parity until real audited lineage and self-build proof evidence exist

#### Scenario: Parity consumes bounded real self-build proof evidence [r[bootstrap.stagex.selfbuild.proof.real-proof-parity-consumption]]

- GIVEN a checked real self-build proof evidence bundle for `crunch.self-build`
- AND the bundle links a release manifest, deterministic proof receipt, sandbox evidence, release verify receipt, and summary artifacts with matching BLAKE3 digests
- AND the deterministic proof receipt has workflow `mantle-deterministic-proof-receipt-v1`, verdict `self-rebuild-match`, selected provider kind matching the release proof linkage, two distinct clean rebuild roots, matching artifact digest sets, and sandbox profile identity `mantle-proof-sandbox-v1:*`
- AND the release verify receipt reports deterministic release status `eligible`
- WHEN the bootstrap parity report evaluates `crunch.self-build`
- THEN the row reports the real proof evidence digest and selected provider kind
- AND the row may report evidence-backed `partial` for the selected provider kind
- AND the row MUST NOT report `complete` or unblock Guix/StageX parity unless all axis-specific source-root or StageX lineage requirements are also satisfied

#### Scenario: Parity rejects malformed or unsafe real self-build proof evidence [r[bootstrap.stagex.selfbuild.proof.real-proof-fail-closed]]

- GIVEN a real self-build proof evidence bundle is missing, malformed, names an unsupported workflow version, records mismatched provider kinds, records mismatched proof or summary digests, omits supported sandbox evidence, uses a direct-host or unsupported sandbox profile, reuses a proof store/root, or has release verify status other than `eligible`
- WHEN the bootstrap parity report evaluates `crunch.self-build`
- THEN the row remains a blocker for Guix and StageX axes that require self-build proof
- AND the row notes name the specific failed evidence check
- AND no deterministic, full-source, Guix, or StageX parity claim is emitted

### Requirement: Live-bootstrap part grep 2.4 is independently tracked
Mantle MUST track the live-bootstrap part `grep 2.4` as an independent bootstrap change bound to `bootstrap/grep-2.4-musl.ncl`.
ID: bootstrap.part.grep.2.4

The part scope MUST include only the source pins, patches, derivation logic, output contract, and evidence needed for `bootstrap/grep-2.4-musl.ncl` and direct predecessor compatibility. Broader chain validation MAY depend on this part, but MUST NOT replace this part's own build and smoke evidence.

#### Scenario: Part evidence is isolated

- GIVEN implementation work touches `bootstrap/grep-2.4-musl.ncl`
- WHEN the part is marked complete
- THEN the change records source-pin audit evidence for `bootstrap/grep-2.4-musl.ncl`
- AND it records a successful `mantle build bootstrap/grep-2.4-musl.ncl` transcript
- AND it records a smoke check for the produced output contract

#### Scenario: Downstream blockers stay local

- GIVEN a downstream bootstrap stage fails after `bootstrap/grep-2.4-musl.ncl` succeeds
- WHEN this part's evidence is reviewed
- THEN the downstream failure does not invalidate this part's completed evidence
- AND a separate part change tracks the downstream failure

<!-- synced from openspec change: live-part-make-3-82 -->
<!-- ADDED Requirements -->

### Requirement: Live-bootstrap part make 3.82 is independently tracked
Mantle MUST track the live-bootstrap part `make 3.82` as an independent bootstrap change bound to `bootstrap/make-tcc.ncl`.
ID: bootstrap.part.make.3.82

The part scope MUST include only the source pins, patches, derivation logic, output contract, and evidence needed for `bootstrap/make-tcc.ncl` and direct predecessor compatibility. Broader chain validation MAY depend on this part, but MUST NOT replace this part's own build and smoke evidence.

#### Scenario: Part evidence is isolated

- GIVEN implementation work touches `bootstrap/make-tcc.ncl`
- WHEN the part is marked complete
- THEN the change records source-pin audit evidence for `bootstrap/make-tcc.ncl`
- AND it records a successful `mantle build bootstrap/make-tcc.ncl` transcript
- AND it records a smoke check for the produced output contract

#### Scenario: Downstream blockers stay local

- GIVEN a downstream bootstrap stage fails after `bootstrap/make-tcc.ncl` succeeds
- WHEN this part's evidence is reviewed
- THEN the downstream failure does not invalidate this part's completed evidence
- AND a separate part change tracks the downstream failure

<!-- synced from openspec change: live-part-musl-1-1-24-tcc -->
<!-- ADDED Requirements -->

### Requirement: Live-bootstrap part musl 1.1.24 (tcc) is independently tracked
Mantle MUST track the live-bootstrap part `musl 1.1.24 and musl_target` as an independent bootstrap change bound to `bootstrap/musl-1.1.24-tcc.ncl`.
ID: bootstrap.part.musl.1.1.24.tcc

The part scope MUST include only the source pins, patches, derivation logic, output contract, and evidence needed for `bootstrap/musl-1.1.24-tcc.ncl` and direct predecessor compatibility. Broader chain validation MAY depend on this part, but MUST NOT replace this part's own build and smoke evidence.

#### Scenario: Part evidence is isolated

- GIVEN implementation work touches `bootstrap/musl-1.1.24-tcc.ncl`
- WHEN the part is marked complete
- THEN the change records source-pin audit evidence for `bootstrap/musl-1.1.24-tcc.ncl`
- AND it records a successful `mantle build bootstrap/musl-1.1.24-tcc.ncl` transcript
- AND it records a smoke check for the produced output contract

#### Scenario: Downstream blockers stay local

- GIVEN a downstream bootstrap stage fails after `bootstrap/musl-1.1.24-tcc.ncl` succeeds
- WHEN this part's evidence is reviewed
- THEN the downstream failure does not invalidate this part's completed evidence
- AND a separate part change tracks the downstream failure

<!-- synced from openspec change: live-part-musl-1-1-24-tcc-musl -->
<!-- ADDED Requirements -->

### Requirement: Live-bootstrap part musl 1.1.24 (tcc-musl) is independently tracked
Mantle MUST track the live-bootstrap part `musl 1.1.24 and musl_target` as an independent bootstrap change bound to `bootstrap/musl-1.1.24-tcc-musl.ncl`.
ID: bootstrap.part.musl.1.1.24.tcc.musl

The part scope MUST include only the source pins, patches, derivation logic, output contract, and evidence needed for `bootstrap/musl-1.1.24-tcc-musl.ncl` and direct predecessor compatibility. Broader chain validation MAY depend on this part, but MUST NOT replace this part's own build and smoke evidence.

#### Scenario: Part evidence is isolated

- GIVEN implementation work touches `bootstrap/musl-1.1.24-tcc-musl.ncl`
- WHEN the part is marked complete
- THEN the change records source-pin audit evidence for `bootstrap/musl-1.1.24-tcc-musl.ncl`
- AND it records a successful `mantle build bootstrap/musl-1.1.24-tcc-musl.ncl` transcript
- AND it records a smoke check for the produced output contract

#### Scenario: Downstream blockers stay local

- GIVEN a downstream bootstrap stage fails after `bootstrap/musl-1.1.24-tcc-musl.ncl` succeeds
- WHEN this part's evidence is reviewed
- THEN the downstream failure does not invalidate this part's completed evidence
- AND a separate part change tracks the downstream failure

<!-- synced from openspec change: live-part-patch-2-5-9 -->
<!-- ADDED Requirements -->

### Requirement: Live-bootstrap part patch 2.5.9 is independently tracked
Mantle MUST track the live-bootstrap part `patch 2.5.9` as an independent bootstrap change bound to `bootstrap/patch-tcc.ncl`.
ID: bootstrap.part.patch.2.5.9

The part scope MUST include only the source pins, patches, derivation logic, output contract, and evidence needed for `bootstrap/patch-tcc.ncl` and direct predecessor compatibility. Broader chain validation MAY depend on this part, but MUST NOT replace this part's own build and smoke evidence.

#### Scenario: Part evidence is isolated

- GIVEN implementation work touches `bootstrap/patch-tcc.ncl`
- WHEN the part is marked complete
- THEN the change records source-pin audit evidence for `bootstrap/patch-tcc.ncl`
- AND it records a successful `mantle build bootstrap/patch-tcc.ncl` transcript
- AND it records a smoke check for the produced output contract

#### Scenario: Downstream blockers stay local

- GIVEN a downstream bootstrap stage fails after `bootstrap/patch-tcc.ncl` succeeds
- WHEN this part's evidence is reviewed
- THEN the downstream failure does not invalidate this part's completed evidence
- AND a separate part change tracks the downstream failure

<!-- synced from openspec change: live-part-tcc-musl -->
<!-- ADDED Requirements -->

### Requirement: Live-bootstrap part tcc linked to musl is independently tracked
Mantle MUST track the live-bootstrap part `musl 1.1.24 and musl_target` as an independent bootstrap change bound to `bootstrap/tcc-musl.ncl`.
ID: bootstrap.part.tcc.musl

The part scope MUST include only the source pins, patches, derivation logic, output contract, and evidence needed for `bootstrap/tcc-musl.ncl` and direct predecessor compatibility. Broader chain validation MAY depend on this part, but MUST NOT replace this part's own build and smoke evidence.

#### Scenario: Part evidence is isolated

- GIVEN implementation work touches `bootstrap/tcc-musl.ncl`
- WHEN the part is marked complete
- THEN the change records source-pin audit evidence for `bootstrap/tcc-musl.ncl`
- AND it records a successful `mantle build bootstrap/tcc-musl.ncl` transcript
- AND it records a smoke check for the produced output contract

#### Scenario: Downstream blockers stay local

- GIVEN a downstream bootstrap stage fails after `bootstrap/tcc-musl.ncl` succeeds
- WHEN this part's evidence is reviewed
- THEN the downstream failure does not invalidate this part's completed evidence
- AND a separate part change tracks the downstream failure

<!-- synced from openspec change: live-part-tcc-musl-prep -->
<!-- ADDED Requirements -->

### Requirement: Live-bootstrap part tcc musl prep is independently tracked
Mantle MUST track the live-bootstrap part `musl 1.1.24 and musl_target` as an independent bootstrap change bound to `bootstrap/tcc-musl-prep.ncl`.
ID: bootstrap.part.tcc.musl.prep

The part scope MUST include only the source pins, patches, derivation logic, output contract, and evidence needed for `bootstrap/tcc-musl-prep.ncl` and direct predecessor compatibility. Broader chain validation MAY depend on this part, but MUST NOT replace this part's own build and smoke evidence.

#### Scenario: Part evidence is isolated

- GIVEN implementation work touches `bootstrap/tcc-musl-prep.ncl`
- WHEN the part is marked complete
- THEN the change records source-pin audit evidence for `bootstrap/tcc-musl-prep.ncl`
- AND it records a successful `mantle build bootstrap/tcc-musl-prep.ncl` transcript
- AND it records a smoke check for the produced output contract

#### Scenario: Downstream blockers stay local

- GIVEN a downstream bootstrap stage fails after `bootstrap/tcc-musl-prep.ncl` succeeds
- WHEN this part's evidence is reviewed
- THEN the downstream failure does not invalidate this part's completed evidence
- AND a separate part change tracks the downstream failure

<!-- synced from openspec change: live-part-tcc-musl-v2 -->
<!-- ADDED Requirements -->

### Requirement: Live-bootstrap part tcc musl v2 is independently tracked
Mantle MUST track the live-bootstrap part `musl 1.1.24 and musl_target` as an independent bootstrap change bound to `bootstrap/tcc-musl-v2.ncl`.
ID: bootstrap.part.tcc.musl.v2

The part scope MUST include only the source pins, patches, derivation logic, output contract, and evidence needed for `bootstrap/tcc-musl-v2.ncl` and direct predecessor compatibility. Broader chain validation MAY depend on this part, but MUST NOT replace this part's own build and smoke evidence.

#### Scenario: Part evidence is isolated

- GIVEN implementation work touches `bootstrap/tcc-musl-v2.ncl`
- WHEN the part is marked complete
- THEN the change records source-pin audit evidence for `bootstrap/tcc-musl-v2.ncl`
- AND it records a successful `mantle build bootstrap/tcc-musl-v2.ncl` transcript
- AND it records a smoke check for the produced output contract

#### Scenario: Downstream blockers stay local

- GIVEN a downstream bootstrap stage fails after `bootstrap/tcc-musl-v2.ncl` succeeds
- WHEN this part's evidence is reviewed
- THEN the downstream failure does not invalidate this part's completed evidence
- AND a separate part change tracks the downstream failure

<!-- synced from openspec change: repair-make-tcc-amd64-varargs -->
<!-- ADDED Requirements -->

### Requirement: First GNU make pass is amd64 executable [r[bootstrap.part.make.3.82.amd64.execution]]
Mantle MUST build `bootstrap/make-tcc.ncl` into a `make 3.82` output that executes basic Makefiles on amd64.

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
Mantle MUST preserve runtime proof for the repaired `bootstrap/make-tcc.ncl` output before the first GNU make amd64 repair is treated as complete.

The proof MUST include a long-budget build transcript, the produced output path or concrete failure diagnostics, version smoke evidence for `GNU Make 3.82`, a simple Makefile positive smoke, a missing-target negative smoke that fails without a signal/segfault, and a host-leakage scan over the derivation, transcript, and output.

#### Scenario: Long build produces make output [r[bootstrap.part.make.3.82.amd64.runtime-validation.scenario.build-output]]
- GIVEN the parent repair source state
- WHEN `mantle build bootstrap/make-tcc.ncl` runs with the documented bootstrap environment and a long validation budget
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

Mantle MUST repair the i386 TinyCC 0.9.26 proof so the generated x86_64-hosted/i386-targeting compiler can emit object files without segfaulting.

#### Scenario: Object emission succeeds [r[bootstrap.i386-tinycc26-object-emission.repair.object]]

- GIVEN the generated `tcc26-i386` diagnostic compiler
- WHEN it compiles assembly or C input with `-c`
- THEN it MUST produce an object file without a segmentation fault.

### Requirement: i386 TinyCC 0.9.26 runtime proof [r[bootstrap.i386-tinycc26-object-emission.runtime-proof]]

Mantle MUST prove the repaired i386 TinyCC 0.9.26 path can produce and execute a no-libc i386 ELF before using it as a basis for later i386 bootstrap stages.

#### Scenario: i386 exit42 runs [r[bootstrap.i386-tinycc26-object-emission.runtime-proof.exit42]]

- GIVEN `tcc26-i386` can emit objects
- WHEN it links the no-libc `_start` smoke executable
- THEN the produced i386 ELF MUST execute inside Mantle's sandbox with exit code 42.

<!-- synced from openspec change: spike-i386-tcc27-make-pass1 -->

### Requirement: i386 TinyCC 0.9.27 Make pass1 spike [r[bootstrap.i386-tcc27-make-pass1.spike]]

Mantle MUST provide a bounded sibling proof for the i386 live-bootstrap sequence from TinyCC 0.9.26 through TinyCC 0.9.27 to GNU Make 3.82 pass1 before changing production bootstrap routing.

#### Scenario: First blocker is recorded [r[bootstrap.i386-tcc27-make-pass1.spike.blocker]]

- GIVEN the proven `tcc26-i386` predecessor compiler
- WHEN the sibling proof attempts TinyCC 0.9.27 and Make 3.82 pass1
- THEN it MUST either pass the Make smoke or save the first concrete failing step with logs.

#### Scenario: Make smoke is required for success [r[bootstrap.i386-tcc27-make-pass1.spike.make-smoke]]

- GIVEN the sibling proof produces a `make` binary
- WHEN the proof claims success
- THEN `make --version` and a trivial Makefile execution MUST both pass inside Mantle's sandbox.

<!-- synced from openspec change: spike-i386-mes-runtime-layout -->

### Requirement: i386 Mes runtime/header layout spike [r[bootstrap.i386-mes-runtime-layout.spike]]

Mantle MUST keep the i386 Mes runtime/header layout investigation as a bounded sibling diagnostic before changing production Make/TinyCC bootstrap routing.

#### Scenario: Layout proof records first blocker [r[bootstrap.i386-mes-runtime-layout.evidence]]

- GIVEN the proven `tcc26-i386` predecessor
- WHEN Mantle builds the i386 Mes runtime/header layout spike
- THEN the output MUST include logs and a summary naming the first blocked step or the next successful handoff boundary.

### Requirement: i386 TinyCC 0.9.27 handoff blocker [r[bootstrap.i386-mes-runtime-layout.blocker]]

Mantle MUST distinguish Mes header/CRT layout progress from complete runtime library availability before attempting the TinyCC 0.9.27 -> Make 3.82 handoff.

#### Scenario: Runtime library blocker is explicit [r[bootstrap.i386-mes-runtime-layout.blocker.runtime-library]]

- GIVEN an i386 Mes header tree and CRT object can be created
- WHEN runtime library or TinyCC 0.9.27 object compilation fails
- THEN the evidence MUST record the exact step, return code, and stderr excerpt.

<!-- synced from openspec change: repair-i386-mes-libtcc1-flags -->

### Requirement: i386 Mes libtcc1 compile flags [r[bootstrap.i386-mes-libtcc1-flags.repair]]

Mantle MUST compile the i386 Mes `libtcc1.c` proof with flags that avoid unsupported broad float and long-long helper emission in the `tcc26-i386` predecessor.

#### Scenario: Real libtcc1 archive is created [r[bootstrap.i386-mes-libtcc1-flags.evidence]]

- GIVEN the i386 Mes runtime-layout sibling proof
- WHEN it builds Mes `lib/libtcc1.c`
- THEN `runtime_libtcc1_object` and `runtime_libtcc1_archive` MUST exit 0 without using the prior placeholder object path.

### Requirement: Post-libtcc1 TinyCC 0.9.27 blocker [r[bootstrap.i386-mes-libtcc1-flags.next-blocker]]

Mantle MUST record the next TinyCC handoff blocker after real i386 `libtcc1.a` creation.

#### Scenario: Next blocked step is explicit [r[bootstrap.i386-mes-libtcc1-flags.next-blocker.explicit]]

- GIVEN a real i386 Mes `libtcc1.a` archive exists
- WHEN the TinyCC 0.9.27 object probe fails
- THEN the evidence MUST name the failing step, return code, and stderr excerpt.

<!-- synced from openspec change: repair-i386-tcc27-source-diagnostics -->

### Requirement: i386 TinyCC 0.9.27 source diagnostics [r[bootstrap.i386-tcc27-source-diagnostics.narrowing]]

Mantle MUST keep the i386 TinyCC 0.9.27 handoff diagnostic narrow enough to distinguish predecessor compiler source-emission failures from Mes runtime-library failures.

#### Scenario: Focused diagnostic matrix is preserved [r[bootstrap.i386-tcc27-source-diagnostics.evidence]]

- GIVEN the sibling i386 Mes runtime-layout proof has built `runtime_libtcc1_object` and `runtime_libtcc1_archive` successfully
- WHEN the TinyCC 0.9.27 pass1 object probe fails
- THEN the proof output MUST include diagnostic return codes for no-line preprocessing, line-marker preprocessing, representative unit compiles, and the gating `tcc27_compile_object` step.

### Requirement: i386 TinyCC 0.9.27 pass1 parity patches [r[bootstrap.i386-tcc27-source-diagnostics.pass1-parity]]

Mantle MUST keep the sibling tcc27 pass1 probe aligned with live-bootstrap pass1 source edits and flags when narrowing the handoff blocker.

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
Mantle MUST track the live-bootstrap part `bzip2 1.0.8` as an independent bootstrap change bound to `bootstrap/bzip2-tcc.ncl`.

The part scope MUST include only the source pins, patches, derivation logic, output contract, and evidence needed for `bootstrap/bzip2-tcc.ncl` and direct predecessor compatibility. Broader chain validation MAY depend on this part, but MUST NOT replace this part's own build and smoke evidence.

#### Scenario: Part evidence is isolated [r[bootstrap.part.bzip2.1.0.8.tcc.evidence-isolated]]

- GIVEN implementation work touches `bootstrap/bzip2-tcc.ncl`
- WHEN the part is marked complete
- THEN the change records source-pin audit evidence for `bootstrap/bzip2-tcc.ncl`
- AND it records a successful `mantle build bootstrap/bzip2-tcc.ncl` transcript
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
- **WHEN** the derivation has not been built successfully in Mantle
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
Mantle MUST track the live-bootstrap part `sed 4.0.9` as an independent bootstrap change bound to `bootstrap/sed-tcc.ncl`.
ID: bootstrap.part.sed.4.0.9.tcc

The part scope MUST include only the source pins, patches, derivation logic, output contract, and evidence needed for `bootstrap/sed-tcc.ncl` and direct predecessor compatibility. Broader chain validation MAY depend on this part, but MUST NOT replace this part's own build and smoke evidence.

#### Scenario: Part evidence is isolated

- GIVEN implementation work touches `bootstrap/sed-tcc.ncl`
- WHEN the part is marked complete
- THEN the change records source-pin audit evidence for `bootstrap/sed-tcc.ncl`
- AND it records a successful `mantle build bootstrap/sed-tcc.ncl` transcript
- AND it records a smoke check for the produced output contract

#### Scenario: Downstream blockers stay local

- GIVEN a downstream bootstrap stage fails after `bootstrap/sed-tcc.ncl` succeeds
- WHEN this part's evidence is reviewed
- THEN the downstream failure does not invalidate this part's completed evidence
- AND a separate part change tracks the downstream failure

### Requirement: Live-bootstrap part bzip2 1.0.8 (musl) is independently tracked [r[bootstrap.part.bzip2.1.0.8.musl]]
Mantle MUST track the live-bootstrap part `bzip2 1.0.8` as an independent bootstrap change bound to `bootstrap/bzip2-1.0.8-musl.ncl`.

The part scope MUST include only the source pins, patches, derivation logic, output contract, and evidence needed for `bootstrap/bzip2-1.0.8-musl.ncl` and direct predecessor compatibility. Broader chain validation MAY depend on this part, but MUST NOT replace this part's own build and smoke evidence.

#### Scenario: Part evidence is isolated [r[bootstrap.part.bzip2.1.0.8.musl.evidence-isolated]]

- GIVEN implementation work touches `bootstrap/bzip2-1.0.8-musl.ncl`
- WHEN the part is marked complete
- THEN the change records source-pin audit evidence for `bootstrap/bzip2-1.0.8-musl.ncl`
- AND it records a successful `mantle build bootstrap/bzip2-1.0.8-musl.ncl` transcript
- AND it records a smoke check for the produced output contract

#### Scenario: Downstream blockers stay local [r[bootstrap.part.bzip2.1.0.8.musl.downstream-local]]

- GIVEN a downstream bootstrap stage fails after `bootstrap/bzip2-1.0.8-musl.ncl` succeeds
- WHEN this part's evidence is reviewed
- THEN the downstream failure does not invalidate this part's completed evidence
- AND a separate part change tracks the downstream failure

### Requirement: Binutils-tcc runtime validation completes independently [r[bootstrap.binutils.tcc.runtime-validation]]
The system MUST provide reproducible validation evidence for the binutils-tcc epoch chain when the local build runner requires more than one drain budget window.

#### Scenario: Longer-running Mes prerequisite [r[bootstrap.binutils.tcc.runtime-validation.mes-prerequisite]]
- **GIVEN** the binutils-tcc chain needs to build Mes before the first epoch root
- **WHEN** the validation is run with bubblewrap and writable local state/store directories
- **THEN** the evidence records whether Mes completed, failed, or required a separately cached prerequisite strategy

#### Scenario: Parent evidence set preserved [r[bootstrap.binutils.tcc.runtime-validation.parent-evidence]]
- **GIVEN** the parent V2-V5 validation tasks were deferred
- **WHEN** this follow-up completes
- **THEN** it records epoch builds, no-host leakage, post-musl linkage, and binutils smoke evidence sufficient to close the parent validation gap

### Requirement: gcc-4.0 runtime validation completes after binutils-tcc [r[bootstrap.gcc40.runtime-validation]]
The system MUST validate gcc-4.0.4 only after the binutils-tcc runtime evidence is available or explicitly blocked.

#### Scenario: Build transcript records transition evidence [r[bootstrap.gcc40.runtime-validation.transcript]]
- **GIVEN** the binutils-tcc output is available
- **WHEN** `bootstrap/gcc-4.0.ncl` is built
- **THEN** the transcript records command, provider, exit status, output path or failure class, fallback status, and placeholder rejection

#### Scenario: Compiler smoke tests prove output [r[bootstrap.gcc40.runtime-validation.smoke]]
- **GIVEN** gcc-4.0.4 builds successfully
- **WHEN** C and C++ smoke programs are compiled
- **THEN** the evidence proves the produced compiler output works without host fallback

### Requirement: Source-chain runtime validation completes after transition evidence [r[bootstrap.source.chain.runtime-validation]]
The system MUST keep full-source bootstrap proof incomplete until transition-build, final-provider, and self-build proof evidence is available.

#### Scenario: Transition blockers remain explicit [r[bootstrap.source.chain.runtime-validation.blockers]]
- **GIVEN** binutils-tcc or gcc transition validation is deferred
- **WHEN** source-chain status is reported
- **THEN** the evidence identifies the deferred follow-up changes and does not promote full-source status

#### Scenario: Final provider proof closes umbrella validation [r[bootstrap.source.chain.runtime-validation.final-proof]]
- **GIVEN** transition evidence is complete
- **WHEN** final provider and self-build proof validation runs
- **THEN** the evidence records normalized provider contract fields, fallback-event markers, and proof digests required for promotion

### Requirement: gcc-4.7 runtime validation completes after prerequisite stages [r[bootstrap.gcc47.runtime-validation]]
The system MUST validate gcc-4.7.4 only after binutils-tcc and gcc-4.0 runtime evidence is available or explicitly blocked.

#### Scenario: Build transcript records gcc-4.7 transition evidence [r[bootstrap.gcc47.runtime-validation.transcript]]
- **GIVEN** prerequisite runtime outputs are available
- **WHEN** `bootstrap/gcc-4.7.ncl` is built
- **THEN** the transcript records command, provider, exit status, output path or failure class, fallback status, and placeholder rejection

#### Scenario: C++11 smoke proves transition [r[bootstrap.gcc47.runtime-validation.smoke]]
- **GIVEN** gcc-4.7.4 builds successfully
- **WHEN** C, C++, and minimal C++11 smoke programs are compiled
- **THEN** the evidence proves the produced compiler output works without host fallback

### Requirement: musl 1.1.24 tcc runtime validation waits for prerequisite execution proof [r[bootstrap.part.musl.1.1.24.tcc.runtime-validation]]
The system MUST keep the first musl pass runtime proof incomplete until prerequisite make/tcc execution blockers are resolved and the produced musl output contract is smoke-tested.

#### Scenario: Source-level hardening is not runtime proof [r[bootstrap.part.musl.1.1.24.tcc.runtime-validation.source-hardening]]
- **GIVEN** `bootstrap/musl-1.1.24-tcc.ncl` fails closed on missing startup objects
- **WHEN** the derivation has not been built successfully in Mantle
- **THEN** runtime validation remains incomplete

#### Scenario: First musl contract is proven [r[bootstrap.part.musl.1.1.24.tcc.runtime-validation.output-contract]]
- **GIVEN** prerequisite runtime blockers are resolved
- **WHEN** `bootstrap/musl-1.1.24-tcc.ncl` builds successfully
- **THEN** the evidence proves `libc.a`, installed headers, and a startup object exist without host fallback

### Requirement: grep 2.4 runtime validation remains explicit [r[bootstrap.part.grep.2.4.runtime-validation]]
The system MUST keep grep 2.4 runtime proof incomplete until a completed build transcript, output contract smoke test, and leakage scan are recorded.

#### Scenario: Long-running prerequisite build [r[bootstrap.part.grep.2.4.runtime-validation.long-build]]
- **GIVEN** Mantle needs to build prerequisite bootstrap inputs for `bootstrap/grep-2.4-musl.ncl`
- **WHEN** the validation run exceeds a short drain timeout
- **THEN** the runtime proof remains in this follow-up change rather than being silently treated as complete

#### Scenario: Output contract is proven [r[bootstrap.part.grep.2.4.runtime-validation.output-contract]]
- **GIVEN** `bootstrap/grep-2.4-musl.ncl` builds successfully
- **WHEN** the produced output is smoke-tested
- **THEN** `grep`, `egrep`, and `fgrep` are present and usable without undeclared host fallback

### Requirement: Live-bootstrap part mpc 1.2.1 is independently tracked
Mantle MUST track the live-bootstrap implemented part `mpc-1.2.1` as an independent bootstrap change bound to `bootstrap/mpc-1.2.1.ncl`, while recording that upstream `parts.rst` currently labels the corresponding narrative heading `mpc 3.2.1`.
ID: bootstrap.part.mpc.1.2.1

The part scope MUST include only the source pins, patches, derivation logic, output contract, and evidence needed for `bootstrap/mpc-1.2.1.ncl` and direct predecessor compatibility. Broader chain validation MAY depend on this part, but MUST NOT replace this part's own build and smoke evidence.

#### Scenario: Part evidence is isolated

- GIVEN implementation work touches `bootstrap/mpc-1.2.1.ncl`
- WHEN the part is marked complete
- THEN the change records source-pin audit evidence for `bootstrap/mpc-1.2.1.ncl`
- AND it records a successful `mantle build bootstrap/mpc-1.2.1.ncl` transcript when declared prerequisite providers exist
- AND if declared prerequisite providers are absent, it records fail-closed blocker evidence without substituting host or legacy providers
- AND it records a smoke check for the produced output contract when an output exists

#### Scenario: Downstream blockers stay local

- GIVEN a downstream bootstrap stage fails after `bootstrap/mpc-1.2.1.ncl` succeeds
- WHEN this part's evidence is reviewed
- THEN the downstream failure does not invalidate this part's completed evidence
- AND a separate part change tracks the downstream failure

### Requirement: Live-bootstrap part autoconf 2.61 is independently tracked
Mantle MUST track the live-bootstrap part `autoconf 2.61` as an independent bootstrap change bound to `bootstrap/autoconf-2.61.ncl`.
ID: bootstrap.part.autoconf.2.61

The part scope MUST include only the source pins, patches, derivation logic, output contract, and evidence needed for `bootstrap/autoconf-2.61.ncl` and direct predecessor compatibility. Broader chain validation MAY depend on this part, but MUST NOT replace this part's own build and smoke evidence.

#### Scenario: Part evidence is isolated

- GIVEN implementation work touches `bootstrap/autoconf-2.61.ncl`
- WHEN the part is marked complete
- THEN the change records source-pin audit evidence for `bootstrap/autoconf-2.61.ncl`
- AND it records a successful `mantle build bootstrap/autoconf-2.61.ncl` transcript when declared prerequisite providers exist
- AND if declared prerequisite providers are absent or unvalidated, it records fail-closed blocker evidence without substituting host or legacy providers
- AND it records a smoke check for the produced output contract when an output exists

#### Scenario: Downstream blockers stay local

- GIVEN a downstream bootstrap stage fails after `bootstrap/autoconf-2.61.ncl` succeeds
- WHEN this part's evidence is reviewed
- THEN the downstream failure does not invalidate this part's completed evidence
- AND a separate part change tracks the downstream failure

### Requirement: Live-bootstrap part autoconf 2.52 is independently tracked
Mantle MUST track the live-bootstrap part `autoconf 2.52` as an independent bootstrap change bound to `bootstrap/autoconf-2.52.ncl`.
ID: bootstrap.part.autoconf.2.52

The part scope MUST include only the source pins, patches, derivation logic, output contract, and evidence needed for `bootstrap/autoconf-2.52.ncl` and direct predecessor compatibility. Broader chain validation MAY depend on this part, but MUST NOT replace this part's own build and smoke evidence.

#### Scenario: Part evidence is isolated

- GIVEN implementation work touches `bootstrap/autoconf-2.52.ncl`
- WHEN the part is marked complete
- THEN the change records source-pin audit evidence for `bootstrap/autoconf-2.52.ncl`
- AND it records a successful `mantle build bootstrap/autoconf-2.52.ncl` transcript when declared prerequisite providers exist
- AND if declared prerequisite providers are absent, blocked, or unvalidated, it records fail-closed blocker evidence without substituting host or legacy providers
- AND it records a smoke check for the produced output contract when an output exists

#### Scenario: Downstream blockers stay local

- GIVEN a downstream bootstrap stage fails after `bootstrap/autoconf-2.52.ncl` succeeds
- WHEN this part's evidence is reviewed
- THEN the downstream failure does not invalidate this part's completed evidence
- AND a separate part change tracks the downstream failure

### Requirement: Live-bootstrap part autoconf 2.53 is independently tracked
Mantle MUST track the live-bootstrap part `autoconf 2.53` as an independent bootstrap change bound to `bootstrap/autoconf-2.53.ncl`.
ID: bootstrap.part.autoconf.2.53

The part scope MUST include only the source pins, patches, derivation logic, output contract, and evidence needed for `bootstrap/autoconf-2.53.ncl` and direct predecessor compatibility. Broader chain validation MAY depend on this part, but MUST NOT replace this part's own build and smoke evidence. If declared predecessor outputs are unavailable or unvalidated, the part MUST record prerequisite-gated evidence and MUST NOT claim build, smoke, leakage-clean, or chain-promotion success.

#### Scenario: Part evidence is isolated

- GIVEN implementation work touches `bootstrap/autoconf-2.53.ncl`
- WHEN the part is marked complete
- THEN the change records source-pin audit evidence for `bootstrap/autoconf-2.53.ncl`
- AND it records either a successful `mantle build bootstrap/autoconf-2.53.ncl` transcript or explicit prerequisite-gated build evidence
- AND it records either a smoke check for the produced output contract or explicit evidence that no output path exists yet
- AND it does not substitute host GCC, Nix-provided tools, or legacy compiler outputs for bootstrap proof

#### Scenario: Downstream blockers stay local

- GIVEN a predecessor or downstream bootstrap stage fails before full `bootstrap/autoconf-2.53.ncl` runtime proof exists
- WHEN this part's evidence is reviewed
- THEN the failure does not invalidate this part's completed source-hardening evidence
- AND a separate part change tracks the predecessor or downstream runtime failure

### Requirement: Live-bootstrap part autoconf 2.54 is independently tracked
Mantle MUST track the live-bootstrap part `autoconf 2.54` as an independent bootstrap change bound to `bootstrap/autoconf-2.54.ncl`.
ID: bootstrap.part.autoconf.2.54

The part scope MUST include only the source pins, patches, derivation logic, output contract, and evidence needed for `bootstrap/autoconf-2.54.ncl` and direct predecessor compatibility. Broader chain validation MAY depend on this part, but MUST NOT replace this part's own build and smoke evidence. If declared predecessor outputs are unavailable or unvalidated, the part MUST record prerequisite-gated evidence and MUST NOT claim build, smoke, leakage-clean, or chain-promotion success.

#### Scenario: Part evidence is isolated

- GIVEN implementation work touches `bootstrap/autoconf-2.54.ncl`
- WHEN the part is marked complete
- THEN the change records source-pin audit evidence for `bootstrap/autoconf-2.54.ncl`
- AND it records either a successful `mantle build bootstrap/autoconf-2.54.ncl` transcript or explicit prerequisite-gated build evidence
- AND it records either a smoke check for the produced output contract or explicit evidence that no output path exists yet
- AND it does not substitute host GCC, Nix-provided tools, or legacy compiler outputs for bootstrap proof

#### Scenario: Downstream blockers stay local

- GIVEN a predecessor or downstream bootstrap stage fails before full `bootstrap/autoconf-2.54.ncl` runtime proof exists
- WHEN this part's evidence is reviewed
- THEN the failure does not invalidate this part's completed source-hardening evidence
- AND a separate part change tracks the predecessor or downstream runtime failure

### Requirement: Live-bootstrap part autoconf 2.55 is independently tracked
Mantle MUST track the live-bootstrap part `autoconf 2.55` as an independent bootstrap change bound to `bootstrap/autoconf-2.55.ncl`.
ID: bootstrap.part.autoconf.2.55

The part scope MUST include only the source pins, patches, derivation logic, output contract, and evidence needed for `bootstrap/autoconf-2.55.ncl` and direct predecessor compatibility. Broader chain validation MAY depend on this part, but MUST NOT replace this part's own build and smoke evidence. If declared predecessor outputs are unavailable or unvalidated, the part MUST record prerequisite-gated evidence and MUST NOT claim build, smoke, leakage-clean, or chain-promotion success.

#### Scenario: Part evidence is isolated

- GIVEN implementation work touches `bootstrap/autoconf-2.55.ncl`
- WHEN the part is marked complete
- THEN the change records source-pin audit evidence for `bootstrap/autoconf-2.55.ncl`
- AND it records either a successful `mantle build bootstrap/autoconf-2.55.ncl` transcript or explicit prerequisite-gated build evidence
- AND it records either a smoke check for the produced output contract or explicit evidence that no output path exists yet
- AND it does not substitute host GCC, Nix-provided tools, or legacy compiler outputs for bootstrap proof

#### Scenario: Downstream blockers stay local

- GIVEN a predecessor or downstream bootstrap stage fails before full `bootstrap/autoconf-2.55.ncl` runtime proof exists
- WHEN this part's evidence is reviewed
- THEN the failure does not invalidate this part's completed source-hardening evidence
- AND a separate part change tracks the predecessor or downstream runtime failure

### Requirement: Live-bootstrap part autoconf 2.57 is independently tracked
Mantle MUST track the live-bootstrap part `autoconf 2.57` as an independent bootstrap change bound to `bootstrap/autoconf-2.57.ncl`.
ID: bootstrap.part.autoconf.2.57

The part scope MUST include only the source pins, patches, derivation logic, output contract, and evidence needed for `bootstrap/autoconf-2.57.ncl` and direct predecessor compatibility. Broader chain validation MAY depend on this part, but MUST NOT replace this part's own build and smoke evidence. If declared predecessor outputs are unavailable or unvalidated, the part MUST record prerequisite-gated evidence and MUST NOT claim build, smoke, leakage-clean, or chain-promotion success.

#### Scenario: Part evidence is isolated

- GIVEN implementation work touches `bootstrap/autoconf-2.57.ncl`
- WHEN the part is marked complete
- THEN the change records source-pin audit evidence for `bootstrap/autoconf-2.57.ncl`
- AND it records either a successful `mantle build bootstrap/autoconf-2.57.ncl` transcript or explicit prerequisite-gated build evidence
- AND it records either a smoke check for the produced output contract or explicit evidence that no output path exists yet
- AND it does not substitute host GCC, Nix-provided tools, or legacy compiler outputs for bootstrap proof

#### Scenario: Downstream blockers stay local

- GIVEN a predecessor or downstream bootstrap stage fails before full `bootstrap/autoconf-2.57.ncl` runtime proof exists
- WHEN this part's evidence is reviewed
- THEN the failure does not invalidate this part's completed source-hardening evidence
- AND a separate part change tracks the predecessor or downstream runtime failure

### Requirement: Live-bootstrap part autoconf 2.59 is independently tracked
Mantle MUST track the live-bootstrap part `autoconf 2.59` as an independent bootstrap change bound to `bootstrap/autoconf-2.59.ncl`.
ID: bootstrap.part.autoconf.2.59

The part scope MUST include only the source pins, patches, derivation logic, output contract, and evidence needed for `bootstrap/autoconf-2.59.ncl` and direct predecessor compatibility. Broader chain validation MAY depend on this part, but MUST NOT replace this part's own build and smoke evidence. If declared predecessor outputs are unavailable or unvalidated, the part MUST record prerequisite-gated evidence and MUST NOT claim build, smoke, leakage-clean, or chain-promotion success.

#### Scenario: Part evidence is isolated

- GIVEN implementation work touches `bootstrap/autoconf-2.59.ncl`
- WHEN the part is marked complete
- THEN the change records source-pin audit evidence for `bootstrap/autoconf-2.59.ncl`
- AND it records either a successful `mantle build bootstrap/autoconf-2.59.ncl` transcript or explicit prerequisite-gated build evidence
- AND it records either a smoke check for the produced output contract or explicit evidence that no output path exists yet
- AND it does not substitute host GCC, Nix-provided tools, or legacy compiler outputs for bootstrap proof

#### Scenario: Downstream blockers stay local

- GIVEN a predecessor or downstream bootstrap stage fails before full `bootstrap/autoconf-2.59.ncl` runtime proof exists
- WHEN this part's evidence is reviewed
- THEN the failure does not invalidate this part's completed source-hardening evidence
- AND a separate part change tracks the predecessor or downstream runtime failure

### Requirement: Live-bootstrap part autoconf 2.64 is independently tracked
Mantle MUST track the live-bootstrap part `autoconf 2.64` as an independent bootstrap change bound to `bootstrap/autoconf-2.64.ncl`.
ID: bootstrap.part.autoconf.2.64

The part scope MUST include only the source pins, patches, derivation logic, output contract, and evidence needed for `bootstrap/autoconf-2.64.ncl` and direct predecessor compatibility. Broader chain validation MAY depend on this part, but MUST NOT replace this part's own build and smoke evidence. If declared predecessor outputs are unavailable or unvalidated, the part MUST record prerequisite-gated evidence and MUST NOT claim build, smoke, leakage-clean, or chain-promotion success.

#### Scenario: Part evidence is isolated

- GIVEN implementation work touches `bootstrap/autoconf-2.64.ncl`
- WHEN the part is marked complete
- THEN the change records source-pin audit evidence for `bootstrap/autoconf-2.64.ncl`
- AND it records either a successful `mantle build bootstrap/autoconf-2.64.ncl` transcript or explicit prerequisite-gated build evidence
- AND it records either a smoke check for the produced output contract or explicit evidence that no output path exists yet
- AND it does not substitute host GCC, Nix-provided tools, or legacy compiler outputs for bootstrap proof

#### Scenario: Downstream blockers stay local

- GIVEN a predecessor or downstream bootstrap stage fails before full `bootstrap/autoconf-2.64.ncl` runtime proof exists
- WHEN this part's evidence is reviewed
- THEN the failure does not invalidate this part's completed source-hardening evidence
- AND a separate part change tracks the predecessor or downstream runtime failure

### Requirement: Live-bootstrap part autoconf 2.69 is independently tracked
Mantle MUST track the live-bootstrap part `autoconf 2.69` as an independent bootstrap change bound to `bootstrap/autoconf-2.69.ncl`.
ID: bootstrap.part.autoconf.2.69

The part scope MUST include only the source pins, patches, derivation logic, output contract, and evidence needed for `bootstrap/autoconf-2.69.ncl` and direct predecessor compatibility. Broader chain validation MAY depend on this part, but MUST NOT replace this part's own build and smoke evidence. If declared predecessor outputs are unavailable or unvalidated, the part MUST record prerequisite-gated evidence and MUST NOT claim build, smoke, leakage-clean, or chain-promotion success.

#### Scenario: Part evidence is isolated

- GIVEN implementation work touches `bootstrap/autoconf-2.69.ncl`
- WHEN the part is marked complete
- THEN the change records source-pin audit evidence for `bootstrap/autoconf-2.69.ncl`
- AND it records either a successful `mantle build bootstrap/autoconf-2.69.ncl` transcript or explicit prerequisite-gated build evidence
- AND it records either a smoke check for the produced output contract or explicit evidence that no output path exists yet
- AND it does not substitute host GCC, Nix-provided tools, or legacy compiler outputs for bootstrap proof

#### Scenario: Downstream blockers stay local

- GIVEN a predecessor or downstream bootstrap stage fails before full `bootstrap/autoconf-2.69.ncl` runtime proof exists
- WHEN this part's evidence is reviewed
- THEN the failure does not invalidate this part's completed source-hardening evidence
- AND a separate part change tracks the predecessor or downstream runtime failure

### Requirement: Live-bootstrap part automake 1.10.3 is independently tracked
Mantle MUST track the live-bootstrap part `automake 1.10.3` as an independent bootstrap change bound to `bootstrap/automake-1.10.3.ncl`.
ID: bootstrap.part.automake.1.10.3

The part scope MUST include only the source pins, patches, derivation logic, output contract, and evidence needed for `bootstrap/automake-1.10.3.ncl` and direct predecessor compatibility. Broader chain validation MAY depend on this part, but MUST NOT replace this part's own build and smoke evidence. If declared predecessor outputs are unavailable or unvalidated, the part MUST record prerequisite-gated evidence and MUST NOT claim build, smoke, leakage-clean, or chain-promotion success.

#### Scenario: Part evidence is isolated

- GIVEN implementation work touches `bootstrap/automake-1.10.3.ncl`
- WHEN the part is marked complete
- THEN the change records source-pin audit evidence for `bootstrap/automake-1.10.3.ncl`
- AND it records either a successful `mantle build bootstrap/automake-1.10.3.ncl` transcript or explicit prerequisite-gated build evidence
- AND it records either a smoke check for the produced output contract or explicit evidence that no output path exists yet
- AND it does not substitute host GCC, Nix-provided Automake, or legacy compiler outputs for bootstrap proof

#### Scenario: Downstream blockers stay local

- GIVEN a predecessor or downstream bootstrap stage fails before full `bootstrap/automake-1.10.3.ncl` runtime proof exists
- WHEN this part's evidence is reviewed
- THEN the failure does not invalidate this part's completed source-hardening evidence
- AND a separate part change tracks the predecessor or downstream runtime failure

### Requirement: Live-bootstrap part automake 1.11.2 is independently tracked
Mantle MUST track the live-bootstrap part `automake 1.11.2` as an independent bootstrap change bound to `bootstrap/automake-1.11.2.ncl`.
ID: bootstrap.part.automake.1.11.2

The part scope MUST include only the source pins, patches, derivation logic, output contract, and evidence needed for `bootstrap/automake-1.11.2.ncl` and direct predecessor compatibility. Broader chain validation MAY depend on this part, but MUST NOT replace this part's own build and smoke evidence. If declared predecessor outputs are unavailable or unvalidated, the part MUST record prerequisite-gated evidence and MUST NOT claim build, smoke, leakage-clean, or chain-promotion success.

#### Scenario: Part evidence is isolated

- GIVEN implementation work touches `bootstrap/automake-1.11.2.ncl`
- WHEN the part is marked complete
- THEN the change records source-pin audit evidence for `bootstrap/automake-1.11.2.ncl`
- AND it records either a successful `mantle build bootstrap/automake-1.11.2.ncl` transcript or explicit prerequisite-gated build evidence
- AND it records either a smoke check for the produced output contract or explicit evidence that no output path exists yet
- AND it does not substitute host GCC, Nix-provided Automake, or legacy compiler outputs for bootstrap proof

#### Scenario: Downstream blockers stay local

- GIVEN a predecessor or downstream bootstrap stage fails before full `bootstrap/automake-1.11.2.ncl` runtime proof exists
- WHEN this part's evidence is reviewed
- THEN the failure does not invalidate this part's completed source-hardening evidence
- AND a separate part change tracks the predecessor or downstream runtime failure

### Requirement: Live-bootstrap part automake 1.15.1 is independently tracked
Mantle MUST track the live-bootstrap part `automake 1.15.1` as an independent bootstrap change bound to `bootstrap/automake-1.15.1.ncl`.
ID: bootstrap.part.automake.1.15.1

The part scope MUST include only the source pins, patches, derivation logic, output contract, and evidence needed for `bootstrap/automake-1.15.1.ncl` and direct predecessor compatibility. Broader chain validation MAY depend on this part, but MUST NOT replace this part's own build and smoke evidence. If declared predecessor outputs are unavailable or unvalidated, the part MUST record prerequisite-gated evidence and MUST NOT claim build, smoke, leakage-clean, or chain-promotion success.

#### Scenario: Part evidence is isolated

- GIVEN implementation work touches `bootstrap/automake-1.15.1.ncl`
- WHEN the part is marked complete
- THEN the change records source-pin audit evidence for `bootstrap/automake-1.15.1.ncl`
- AND it records either a successful `mantle build bootstrap/automake-1.15.1.ncl` transcript or explicit prerequisite-gated build evidence
- AND it records either a smoke check for the produced output contract or explicit evidence that no output path exists yet
- AND it does not substitute host GCC, Nix-provided Automake, or legacy compiler outputs for bootstrap proof

#### Scenario: Downstream blockers stay local

- GIVEN a predecessor or downstream bootstrap stage fails before full `bootstrap/automake-1.15.1.ncl` runtime proof exists
- WHEN this part's evidence is reviewed
- THEN the failure does not invalidate this part's completed source-hardening evidence
- AND a separate part change tracks the predecessor or downstream runtime failure

### Requirement: Live-bootstrap part automake 1.6.3 is independently tracked
Mantle MUST track the live-bootstrap part `automake 1.6.3` as an independent bootstrap change bound to `bootstrap/automake-1.6.3.ncl`.
ID: bootstrap.part.automake.1.6.3

The part scope MUST include only the source pins, patches, derivation logic, output contract, and evidence needed for `bootstrap/automake-1.6.3.ncl` and direct predecessor compatibility. Broader chain validation MAY depend on this part, but MUST NOT replace this part's own build and smoke evidence. If declared predecessor outputs are unavailable or unvalidated, the part MUST record prerequisite-gated evidence and MUST NOT claim build, smoke, leakage-clean, or chain-promotion success.

#### Scenario: Part evidence is isolated

- GIVEN implementation work touches `bootstrap/automake-1.6.3.ncl`
- WHEN the part is marked complete
- THEN the change records source-pin audit evidence for `bootstrap/automake-1.6.3.ncl`
- AND it records either a successful `mantle build bootstrap/automake-1.6.3.ncl` transcript or explicit prerequisite-gated build evidence
- AND it records either a smoke check for the produced output contract or explicit evidence that no output path exists yet
- AND it does not substitute host GCC, Nix-provided Automake, or legacy compiler outputs for bootstrap proof

#### Scenario: Downstream blockers stay local

- GIVEN a predecessor or downstream bootstrap stage fails before full `bootstrap/automake-1.6.3.ncl` runtime proof exists
- WHEN this part's evidence is reviewed
- THEN the failure does not invalidate this part's completed source-hardening evidence
- AND a separate part change tracks the predecessor or downstream runtime failure

### Requirement: Live-bootstrap part automake 1.7 is independently tracked
Mantle MUST track the live-bootstrap part `automake 1.7` as an independent bootstrap change bound to `bootstrap/automake-1.7.ncl`.
ID: bootstrap.part.automake.1.7

The part scope MUST include only the source pins, patches, derivation logic, output contract, and evidence needed for `bootstrap/automake-1.7.ncl` and direct predecessor compatibility. Broader chain validation MAY depend on this part, but MUST NOT replace this part's own build and smoke evidence. If declared predecessor outputs are unavailable or unvalidated, the part MUST record prerequisite-gated evidence and MUST NOT claim build, smoke, leakage-clean, or chain-promotion success.

#### Scenario: Part evidence is isolated

- GIVEN implementation work touches `bootstrap/automake-1.7.ncl`
- WHEN the part is marked complete
- THEN the change records source-pin audit evidence for `bootstrap/automake-1.7.ncl`
- AND it records either a successful `mantle build bootstrap/automake-1.7.ncl` transcript or explicit prerequisite-gated build evidence
- AND it records either a smoke check for the produced output contract or explicit evidence that no output path exists yet
- AND it does not substitute host GCC, Nix-provided Automake, or legacy compiler outputs for bootstrap proof

#### Scenario: Downstream blockers stay local

- GIVEN a predecessor or downstream bootstrap stage fails before full `bootstrap/automake-1.7.ncl` runtime proof exists
- WHEN this part's evidence is reviewed
- THEN the failure does not invalidate this part's completed source-hardening evidence
- AND a separate part change tracks the predecessor or downstream runtime failure

### Requirement: Live-bootstrap part automake 1.7.8 is independently tracked
Mantle MUST track the live-bootstrap part `automake 1.7.8` as an independent bootstrap change bound to `bootstrap/automake-1.7.8.ncl`.
ID: bootstrap.part.automake.1.7.8

The part scope MUST include only the source pins, patches, derivation logic, output contract, and evidence needed for `bootstrap/automake-1.7.8.ncl` and direct predecessor compatibility. Broader chain validation MAY depend on this part, but MUST NOT replace this part's own build and smoke evidence. If declared predecessor outputs are unavailable or unvalidated, the part MUST record prerequisite-gated evidence and MUST NOT claim build, smoke, leakage-clean, or chain-promotion success.

#### Scenario: Part evidence is isolated

- GIVEN implementation work touches `bootstrap/automake-1.7.8.ncl`
- WHEN the part is marked complete
- THEN the change records source-pin audit evidence for `bootstrap/automake-1.7.8.ncl`
- AND it records either a successful `mantle build bootstrap/automake-1.7.8.ncl` transcript or explicit prerequisite-gated build evidence
- AND it records either a smoke check for the produced output contract or explicit evidence that no output path exists yet
- AND it does not substitute host GCC, Nix-provided Automake, or legacy compiler outputs for bootstrap proof

#### Scenario: Downstream blockers stay local

- GIVEN a predecessor or downstream bootstrap stage fails before full `bootstrap/automake-1.7.8.ncl` runtime proof exists
- WHEN this part's evidence is reviewed
- THEN the failure does not invalidate this part's completed source-hardening evidence
- AND a separate part change tracks the predecessor or downstream runtime failure

### Requirement: Live-bootstrap part automake 1.8.5 is independently tracked
Mantle MUST track the live-bootstrap part `automake 1.8.5` as an independent bootstrap change bound to `bootstrap/automake-1.8.5.ncl`.
ID: bootstrap.part.automake.1.8.5

The part scope MUST include only the source pins, patches, derivation logic, output contract, and evidence needed for `bootstrap/automake-1.8.5.ncl` and direct predecessor compatibility. Broader chain validation MAY depend on this part, but MUST NOT replace this part's own build and smoke evidence. If declared predecessor outputs are unavailable or unvalidated, the part MUST record prerequisite-gated evidence and MUST NOT claim build, smoke, leakage-clean, or chain-promotion success.

#### Scenario: Part evidence is isolated

- GIVEN implementation work touches `bootstrap/automake-1.8.5.ncl`
- WHEN the part is marked complete
- THEN the change records source-pin audit evidence for `bootstrap/automake-1.8.5.ncl`
- AND it records either a successful `mantle build bootstrap/automake-1.8.5.ncl` transcript or explicit prerequisite-gated build evidence
- AND it records either a smoke check for the produced output contract or explicit evidence that no output path exists yet
- AND it does not substitute host GCC, Nix-provided Automake, or legacy compiler outputs for bootstrap proof

#### Scenario: Downstream blockers stay local

- GIVEN a predecessor or downstream bootstrap stage fails before full `bootstrap/automake-1.8.5.ncl` runtime proof exists
- WHEN this part's evidence is reviewed
- THEN the failure does not invalidate this part's completed source-hardening evidence
- AND a separate part change tracks the predecessor or downstream runtime failure

### Requirement: Live-bootstrap part automake 1.9.6 is independently tracked
Mantle MUST track the live-bootstrap part `automake 1.9.6` as an independent bootstrap change bound to `bootstrap/automake-1.9.6.ncl`.
ID: bootstrap.part.automake.1.9.6

The part scope MUST include only the source pins, patches, derivation logic, output contract, and evidence needed for `bootstrap/automake-1.9.6.ncl` and direct predecessor compatibility. Broader chain validation MAY depend on this part, but MUST NOT replace this part's own build and smoke evidence. If declared predecessor outputs are unavailable or unvalidated, the part MUST record prerequisite-gated evidence and MUST NOT claim build, smoke, leakage-clean, or chain-promotion success.

#### Scenario: Part evidence is isolated

- GIVEN implementation work touches `bootstrap/automake-1.9.6.ncl`
- WHEN the part is marked complete
- THEN the change records source-pin audit evidence for `bootstrap/automake-1.9.6.ncl`
- AND it records either a successful `mantle build bootstrap/automake-1.9.6.ncl` transcript or explicit prerequisite-gated build evidence
- AND it records either a smoke check for the produced output contract or explicit evidence that no output path exists yet
- AND it does not substitute host GCC, Nix-provided Automake, or legacy compiler outputs for bootstrap proof

#### Scenario: Downstream blockers stay local

- GIVEN a predecessor or downstream bootstrap stage fails before full `bootstrap/automake-1.9.6.ncl` runtime proof exists
- WHEN this part's evidence is reviewed
- THEN the failure does not invalidate this part's completed source-hardening evidence
- AND a separate part change tracks the predecessor or downstream runtime failure

### Requirement: Live-bootstrap part bash 2.05b is independently tracked
Mantle MUST track the live-bootstrap part `bash 2.05b` as an independent bootstrap change bound to `bootstrap/bash-2.05b-tcc.ncl`.
ID: bootstrap.part.bash.2.05b

The part scope MUST include only the source pins, patches, derivation logic, output contract, and evidence needed for `bootstrap/bash-2.05b-tcc.ncl` and direct predecessor compatibility. Broader chain validation MAY depend on this part, but MUST NOT replace this part's own build and smoke evidence. If declared predecessor outputs are unavailable or unvalidated, the part MUST record prerequisite-gated evidence and MUST NOT claim build, smoke, leakage-clean, or chain-promotion success.

#### Scenario: Part evidence is isolated

- GIVEN implementation work touches `bootstrap/bash-2.05b-tcc.ncl`
- WHEN the part is marked complete
- THEN the change records source-pin audit evidence for `bootstrap/bash-2.05b-tcc.ncl`
- AND it records either a successful `mantle build bootstrap/bash-2.05b-tcc.ncl` transcript or explicit prerequisite-gated build evidence
- AND it records either a smoke check for the produced output contract or explicit evidence that no output path exists yet
- AND it does not substitute host GCC, Nix-provided Bash, or legacy compiler outputs for bootstrap proof

#### Scenario: Downstream blockers stay local

- GIVEN a predecessor or downstream bootstrap stage fails before full `bootstrap/bash-2.05b-tcc.ncl` runtime proof exists
- WHEN this part's evidence is reviewed
- THEN the failure does not invalidate this part's completed source-hardening evidence
- AND a separate part change tracks the predecessor or downstream runtime failure

### Requirement: Live-bootstrap part binutils 2.30 is independently tracked
Mantle MUST track the live-bootstrap part `binutils 2.30` as an independent bootstrap change bound to `bootstrap/binutils-tcc.ncl`.
ID: bootstrap.part.binutils.2.30

The part scope MUST include only the source pins, patches, derivation logic, output contract, and evidence needed for `bootstrap/binutils-tcc.ncl` and direct predecessor compatibility. Broader chain validation MAY depend on this part, but MUST NOT replace this part's own build and smoke evidence. If declared predecessor outputs are unavailable or unvalidated, the part MUST record prerequisite-gated evidence and MUST NOT claim build, smoke, leakage-clean, or chain-promotion success.

#### Scenario: Part evidence is isolated

- GIVEN implementation work touches `bootstrap/binutils-tcc.ncl`
- WHEN the part is marked complete
- THEN the change records source-pin audit evidence for `bootstrap/binutils-tcc.ncl`
- AND it records either a successful `mantle build bootstrap/binutils-tcc.ncl` transcript or explicit prerequisite-gated build evidence
- AND it records either a smoke check for the produced output contract or explicit evidence that no output path exists yet
- AND it does not substitute host binutils, Nix-provided binutils, or legacy compiler outputs for bootstrap proof

#### Scenario: Downstream blockers stay local

- GIVEN a predecessor or downstream bootstrap stage fails before full `bootstrap/binutils-tcc.ncl` runtime proof exists
- WHEN this part's evidence is reviewed
- THEN the failure does not invalidate this part's completed source-hardening evidence
- AND a separate part change tracks the predecessor or downstream runtime failure

### Requirement: Live-bootstrap part binutils 2.41 is independently tracked
Mantle MUST track the live-bootstrap part `binutils 2.41` as an independent bootstrap change bound to `bootstrap/binutils-full.ncl`.
ID: bootstrap.part.binutils.2.41

The part scope MUST include only the source pins, patches, derivation logic, output contract, and evidence needed for `bootstrap/binutils-full.ncl` and direct predecessor compatibility. Broader chain validation MAY depend on this part, but MUST NOT replace this part's own build and smoke evidence. If declared predecessor outputs are unavailable or unvalidated, the part MUST record prerequisite-gated evidence and MUST NOT claim build, smoke, leakage-clean, or chain-promotion success.

#### Scenario: Part evidence is isolated

- GIVEN implementation work touches `bootstrap/binutils-full.ncl`
- WHEN the part is marked complete
- THEN the change records source-pin audit evidence for `bootstrap/binutils-full.ncl`
- AND it records either a successful `mantle build bootstrap/binutils-full.ncl` transcript or explicit prerequisite-gated build evidence
- AND it records either a smoke check for the produced output contract or explicit evidence that no output path exists yet
- AND it does not substitute host binutils, Nix-provided binutils, or legacy compiler outputs for bootstrap proof

#### Scenario: Downstream blockers stay local

- GIVEN a predecessor or downstream bootstrap stage fails before full `bootstrap/binutils-full.ncl` runtime proof exists
- WHEN this part's evidence is reviewed
- THEN the failure does not invalidate this part's completed source-hardening evidence
- AND a separate part change tracks the predecessor or downstream runtime failure

### Requirement: Live-bootstrap part bison 3.4.1 is independently tracked
Mantle MUST track the live-bootstrap part `bison 3.4.1` as an independent bootstrap change bound to `bootstrap/bison-3.4.1-musl.ncl`.
ID: bootstrap.part.bison.3.4.1

The part scope MUST include only the source pins, patches, derivation logic, output contract, and evidence needed for `bootstrap/bison-3.4.1-musl.ncl` and direct predecessor compatibility. Broader chain validation MAY depend on this part, but MUST NOT replace this part's own build and smoke evidence. If declared predecessor outputs are unavailable or unvalidated, the part MUST record prerequisite-gated evidence and MUST NOT claim build, smoke, leakage-clean, or chain-promotion success.

#### Scenario: Part evidence is isolated

- GIVEN implementation work touches `bootstrap/bison-3.4.1-musl.ncl`
- WHEN the part is marked complete
- THEN the change records source-pin audit evidence for `bootstrap/bison-3.4.1-musl.ncl`
- AND it records either a successful `mantle build bootstrap/bison-3.4.1-musl.ncl` transcript or explicit prerequisite-gated build evidence
- AND it records either a smoke check for the produced output contract or explicit evidence that no output path exists yet
- AND it does not substitute host Bison, Nix-provided Bison, host GCC, or legacy compiler outputs for bootstrap proof

#### Scenario: Downstream blockers stay local

- GIVEN a predecessor or downstream bootstrap stage fails before full `bootstrap/bison-3.4.1-musl.ncl` runtime proof exists
- WHEN this part's evidence is reviewed
- THEN the failure does not invalidate this part's completed source-hardening evidence
- AND a separate part change tracks the predecessor or downstream runtime failure

### Requirement: Live-bootstrap part coreutils 5.0 (musl) is independently tracked
Mantle MUST track the live-bootstrap part `coreutils 5.0` as an independent bootstrap change bound to `bootstrap/coreutils-5.0-musl.ncl`.
ID: bootstrap.part.coreutils.5.0.musl

The part scope MUST include only the source pins, patches, derivation logic, output contract, and evidence needed for `bootstrap/coreutils-5.0-musl.ncl` and direct predecessor compatibility. Broader chain validation MAY depend on this part, but MUST NOT replace this part's own build and smoke evidence. If declared predecessor outputs are unavailable or unvalidated, the part MUST record prerequisite-gated evidence and MUST NOT claim build, smoke, leakage-clean, or chain-promotion success.

#### Scenario: Part evidence is isolated

- GIVEN implementation work touches `bootstrap/coreutils-5.0-musl.ncl`
- WHEN the part is marked complete
- THEN the change records source-pin audit evidence for `bootstrap/coreutils-5.0-musl.ncl`
- AND it records either a successful `mantle build bootstrap/coreutils-5.0-musl.ncl` transcript or explicit prerequisite-gated build evidence
- AND it records either a smoke check for the produced output contract or explicit evidence that no output path exists yet
- AND it does not substitute host coreutils, Nix-provided coreutils, host GCC, or legacy compiler outputs for bootstrap proof

#### Scenario: Downstream blockers stay local

- GIVEN a predecessor or downstream bootstrap stage fails before full `bootstrap/coreutils-5.0-musl.ncl` runtime proof exists
- WHEN this part's evidence is reviewed
- THEN the failure does not invalidate this part's completed source-hardening evidence
- AND a separate part change tracks the predecessor or downstream runtime failure

### Requirement: Live-bootstrap part coreutils 5.0 (tcc) is independently tracked
Mantle MUST track the live-bootstrap part `coreutils 5.0` as an independent bootstrap change bound to `bootstrap/coreutils-5.0-tcc.ncl`.
ID: bootstrap.part.coreutils.5.0.tcc

The part scope MUST include only the source pins, patches, derivation logic, output contract, and evidence needed for `bootstrap/coreutils-5.0-tcc.ncl` and direct predecessor compatibility. Broader chain validation MAY depend on this part, but MUST NOT replace this part's own build and smoke evidence. If declared predecessor outputs are unavailable or unvalidated, the part MUST record prerequisite-gated evidence and MUST NOT claim build, smoke, leakage-clean, or chain-promotion success.

#### Scenario: Part evidence is isolated

- GIVEN implementation work touches `bootstrap/coreutils-5.0-tcc.ncl`
- WHEN the part is marked complete
- THEN the change records source-pin audit evidence for `bootstrap/coreutils-5.0-tcc.ncl`
- AND it records either a successful `mantle build bootstrap/coreutils-5.0-tcc.ncl` transcript or explicit prerequisite-gated build evidence
- AND it records either a smoke check for the produced output contract or explicit evidence that no output path exists yet
- AND it does not substitute host coreutils, Nix-provided coreutils, host GCC, or legacy compiler outputs for bootstrap proof

#### Scenario: Downstream blockers stay local

- GIVEN a predecessor or downstream bootstrap stage fails before full `bootstrap/coreutils-5.0-tcc.ncl` runtime proof exists
- WHEN this part's evidence is reviewed
- THEN the failure does not invalidate this part's completed source-hardening evidence
- AND a separate part change tracks the predecessor or downstream runtime failure

### Requirement: Live-bootstrap part coreutils 6.10 is independently tracked
Mantle MUST track the live-bootstrap part `coreutils 6.10` as an independent bootstrap change bound to `bootstrap/coreutils-6.10-musl.ncl`.
ID: bootstrap.part.coreutils.6.10

The part scope MUST include only the source pins, patches, derivation logic, output contract, and evidence needed for `bootstrap/coreutils-6.10-musl.ncl` and direct predecessor compatibility. Broader chain validation MAY depend on this part, but MUST NOT replace this part's own build and smoke evidence. If declared predecessor outputs are unavailable or unvalidated, the part MUST record prerequisite-gated evidence and MUST NOT claim build, smoke, leakage-clean, or chain-promotion success.

#### Scenario: Part evidence is isolated

- GIVEN implementation work touches `bootstrap/coreutils-6.10-musl.ncl`
- WHEN the part is marked complete
- THEN the change records source-pin audit evidence for `bootstrap/coreutils-6.10-musl.ncl`
- AND it records either a successful `mantle build bootstrap/coreutils-6.10-musl.ncl` transcript or explicit prerequisite-gated build evidence
- AND it records either a smoke check for the produced output contract or explicit evidence that no output path exists yet
- AND it does not substitute host coreutils, Nix-provided coreutils, host GCC, or legacy compiler outputs for bootstrap proof

#### Scenario: Downstream blockers stay local

- GIVEN a predecessor or downstream bootstrap stage fails before full `bootstrap/coreutils-6.10-musl.ncl` runtime proof exists
- WHEN this part's evidence is reviewed
- THEN the failure does not invalidate this part's completed source-hardening evidence
- AND a separate part change tracks the predecessor or downstream runtime failure

### Requirement: Live-bootstrap part diffutils 2.7 is independently tracked
Mantle MUST track the live-bootstrap part `diffutils 2.7` as an independent bootstrap change bound to `bootstrap/diffutils-2.7-musl.ncl`.
ID: bootstrap.part.diffutils.2.7

The part scope MUST include only the source pins, patches, derivation logic, output contract, and evidence needed for `bootstrap/diffutils-2.7-musl.ncl` and direct predecessor compatibility. Broader chain validation MAY depend on this part, but MUST NOT replace this part's own build and smoke evidence. If declared predecessor outputs are unavailable or unvalidated, the part MUST record prerequisite-gated evidence and MUST NOT claim build, smoke, leakage-clean, or chain-promotion success.

#### Scenario: Part evidence is isolated

- GIVEN implementation work touches `bootstrap/diffutils-2.7-musl.ncl`
- WHEN the part is marked complete
- THEN the change records source-pin audit evidence for `bootstrap/diffutils-2.7-musl.ncl`
- AND it records either a successful `mantle build bootstrap/diffutils-2.7-musl.ncl` transcript or explicit prerequisite-gated build evidence
- AND it records either a smoke check for the produced output contract or explicit evidence that no output path exists yet
- AND it does not substitute host diffutils, Nix-provided diffutils, host GCC, or legacy compiler outputs for bootstrap proof

#### Scenario: Downstream blockers stay local

- GIVEN a predecessor or downstream bootstrap stage fails before full `bootstrap/diffutils-2.7-musl.ncl` runtime proof exists
- WHEN this part's evidence is reviewed
- THEN the failure does not invalidate this part's completed source-hardening evidence
- AND a separate part change tracks the predecessor or downstream runtime failure

### Requirement: Live-bootstrap part flex 2.5.11 is independently tracked
Mantle MUST track the live-bootstrap part `flex 2.5.11` as an independent bootstrap change bound to `bootstrap/flex-2.5.11-musl.ncl`.
ID: bootstrap.part.flex.2.5.11

The part scope MUST include only the source pins, patches, derivation logic, output contract, and evidence needed for `bootstrap/flex-2.5.11-musl.ncl` and direct predecessor compatibility. Broader chain validation MAY depend on this part, but MUST NOT replace this part's own build and smoke evidence. If declared predecessor outputs are unavailable or unvalidated, the part MUST record prerequisite-gated evidence and MUST NOT claim build, smoke, leakage-clean, or chain-promotion success.

#### Scenario: Part evidence is isolated

- GIVEN implementation work touches `bootstrap/flex-2.5.11-musl.ncl`
- WHEN the part is marked complete
- THEN the change records source-pin audit evidence for `bootstrap/flex-2.5.11-musl.ncl`
- AND it records either a successful `mantle build bootstrap/flex-2.5.11-musl.ncl` transcript or explicit prerequisite-gated build evidence
- AND it records either a smoke check for the produced output contract or explicit evidence that no output path exists yet
- AND it does not substitute host flex, Nix-provided flex, host GCC, or legacy compiler outputs for bootstrap proof

#### Scenario: Downstream blockers stay local

- GIVEN a predecessor or downstream bootstrap stage fails before full `bootstrap/flex-2.5.11-musl.ncl` runtime proof exists
- WHEN this part's evidence is reviewed
- THEN the failure does not invalidate this part's completed source-hardening evidence
- AND a separate part change tracks the predecessor or downstream runtime failure

### Requirement: Live-bootstrap part flex 2.6.4 is independently tracked
Mantle MUST track the live-bootstrap part `flex 2.6.4` as an independent bootstrap change bound to `bootstrap/flex-2.6.4-musl.ncl`.
ID: bootstrap.part.flex.2.6.4

The part scope MUST include only the source pins, patches, derivation logic, output contract, and evidence needed for `bootstrap/flex-2.6.4-musl.ncl` and direct predecessor compatibility. Broader chain validation MAY depend on this part, but MUST NOT replace this part's own build and smoke evidence. If declared predecessor outputs are unavailable or unvalidated, the part MUST record prerequisite-gated evidence and MUST NOT claim build, smoke, leakage-clean, or chain-promotion success.

#### Scenario: Part evidence is isolated

- GIVEN implementation work touches `bootstrap/flex-2.6.4-musl.ncl`
- WHEN the part is marked complete
- THEN the change records source-pin audit evidence for `bootstrap/flex-2.6.4-musl.ncl`
- AND it records either a successful `mantle build bootstrap/flex-2.6.4-musl.ncl` transcript or explicit prerequisite-gated build evidence
- AND it records either a smoke check for the produced output contract or explicit evidence that no output path exists yet
- AND it does not substitute host flex, Nix-provided flex, host GCC, or legacy compiler outputs for bootstrap proof

#### Scenario: Downstream blockers stay local

- GIVEN a predecessor or downstream bootstrap stage fails before full `bootstrap/flex-2.6.4-musl.ncl` runtime proof exists
- WHEN this part's evidence is reviewed
- THEN the failure does not invalidate this part's completed source-hardening evidence
- AND a separate part change tracks the predecessor or downstream runtime failure

### Requirement: Live-bootstrap part gawk 3.0.4 is independently tracked
Mantle MUST track the live-bootstrap part `gawk 3.0.4` as an independent bootstrap change bound to `bootstrap/gawk-3.0.4-musl.ncl`.
ID: bootstrap.part.gawk.3.0.4

The part scope MUST include only the source pins, patches, derivation logic, output contract, and evidence needed for `bootstrap/gawk-3.0.4-musl.ncl` and direct predecessor compatibility. Broader chain validation MAY depend on this part, but MUST NOT replace this part's own build and smoke evidence. If declared predecessor outputs are unavailable or unvalidated, the part MUST record prerequisite-gated evidence and MUST NOT claim build, smoke, leakage-clean, or chain-promotion success.

#### Scenario: Part evidence is isolated

- GIVEN implementation work touches `bootstrap/gawk-3.0.4-musl.ncl`
- WHEN the part is marked complete
- THEN the change records source-pin audit evidence for `bootstrap/gawk-3.0.4-musl.ncl`
- AND it records either a successful `mantle build bootstrap/gawk-3.0.4-musl.ncl` transcript or explicit prerequisite-gated build evidence
- AND it records either a smoke check for the produced output contract or explicit evidence that no output path exists yet
- AND it does not substitute host awk/gawk, Nix-provided awk/gawk, host GCC, or legacy compiler outputs for bootstrap proof

#### Scenario: Downstream blockers stay local

- GIVEN a predecessor or downstream bootstrap stage fails before full `bootstrap/gawk-3.0.4-musl.ncl` runtime proof exists
- WHEN this part's evidence is reviewed
- THEN the failure does not invalidate this part's completed source-hardening evidence
- AND a separate part change tracks the predecessor or downstream runtime failure

### Requirement: Live-bootstrap part gcc 10.5.0 is independently tracked
Mantle MUST track the live-bootstrap part `gcc 10.5.0` as an independent bootstrap change bound to `bootstrap/gcc-10.ncl`.
ID: bootstrap.part.gcc.10.5.0

The part scope MUST include only the source pins, patches, derivation logic, output contract, and evidence needed for `bootstrap/gcc-10.ncl` and direct predecessor compatibility. Broader chain validation MAY depend on this part, but MUST NOT replace this part's own build and smoke evidence. If declared predecessor outputs are unavailable or unvalidated, the part MUST record prerequisite-gated evidence and MUST NOT claim build, smoke, leakage-clean, final-provider, or chain-promotion success.

#### Scenario: Part evidence is isolated

- GIVEN implementation work touches `bootstrap/gcc-10.ncl`
- WHEN the part is marked complete
- THEN the change records source-pin audit evidence for `bootstrap/gcc-10.ncl`
- AND it records either a successful `mantle build bootstrap/gcc-10.ncl` transcript or explicit prerequisite-gated build evidence
- AND it records either a smoke check for the produced output contract or explicit evidence that no output path exists yet
- AND it does not substitute host GCC, Nix-provided GCC, or legacy compiler outputs for bootstrap proof

#### Scenario: Downstream blockers stay local

- GIVEN a predecessor or downstream bootstrap stage fails before full `bootstrap/gcc-10.ncl` runtime proof exists
- WHEN this part's evidence is reviewed
- THEN the failure does not invalidate this part's completed source-hardening evidence
- AND a separate part change tracks the predecessor or downstream runtime failure

### Requirement: Live-bootstrap part gcc 4.0.4 is independently tracked
Mantle MUST track the live-bootstrap part `gcc 4.0.4` as an independent bootstrap change bound to `bootstrap/gcc-4.0.ncl`.
ID: bootstrap.part.gcc.4.0.4

The part scope MUST include only the source pins, patches, derivation logic, output contract, and evidence needed for `bootstrap/gcc-4.0.ncl` and direct predecessor compatibility. Broader chain validation MAY depend on this part, but MUST NOT replace this part's own build and smoke evidence. If declared predecessor outputs are unavailable or unvalidated, the part MUST record prerequisite-gated evidence and MUST NOT claim build, smoke, leakage-clean, compiler-provider, or chain-promotion success.

#### Scenario: Part evidence is isolated

- GIVEN implementation work touches `bootstrap/gcc-4.0.ncl`
- WHEN the part is marked complete
- THEN the change records source-pin audit evidence for `bootstrap/gcc-4.0.ncl`
- AND it records either a successful `mantle build bootstrap/gcc-4.0.ncl` transcript or explicit prerequisite-gated build evidence
- AND it records either a smoke check for the produced output contract or explicit evidence that no output path exists yet
- AND it does not substitute host GCC, Nix-provided GCC, or legacy compiler outputs for bootstrap proof

#### Scenario: Downstream blockers stay local

- GIVEN a predecessor or downstream bootstrap stage fails before full `bootstrap/gcc-4.0.ncl` runtime proof exists
- WHEN this part's evidence is reviewed
- THEN the failure does not invalidate this part's completed source-hardening evidence
- AND a separate part change tracks the predecessor or downstream runtime failure

### Requirement: Live-bootstrap part gcc 4.7.4 is independently tracked
Mantle MUST track the live-bootstrap part `gcc 4.7.4` as an independent bootstrap change bound to `bootstrap/gcc-4.7.ncl`.
ID: bootstrap.part.gcc.4.7.4

The part scope MUST include only the source pins, patches, derivation logic, output contract, and evidence needed for `bootstrap/gcc-4.7.ncl` and direct predecessor compatibility. Broader chain validation MAY depend on this part, but MUST NOT replace this part's own build and smoke evidence. If declared predecessor outputs are unavailable or unvalidated, the part MUST record prerequisite-gated evidence and MUST NOT claim build, smoke, leakage-clean, compiler-provider, or chain-promotion success.

#### Scenario: Part evidence is isolated

- GIVEN implementation work touches `bootstrap/gcc-4.7.ncl`
- WHEN the part is marked complete
- THEN the change records source-pin audit evidence for `bootstrap/gcc-4.7.ncl`
- AND it records either a successful `mantle build bootstrap/gcc-4.7.ncl` transcript or explicit prerequisite-gated build evidence
- AND it records either a smoke check for the produced output contract or explicit evidence that no output path exists yet
- AND it does not substitute host GCC, Nix-provided GCC, or legacy compiler outputs for bootstrap proof

#### Scenario: Parity report requires checked C++ provider contract

- GIVEN `bootstrap/evidence/gcc-4.7-cxx-provider-contract.json` records the expected C/C++ configure, build, install, and smoke markers
- WHEN `mantle bootstrap parity-report` evaluates the `gcc.4.7` row
- THEN the row remains `partial` until native/full GCC 4.7 correctness is proven
- AND the row does not report an evidence failure while every required marker is present in `bootstrap/gcc-4.7.ncl`
- AND marker drift or a missing receipt is reported as an evidence failure

#### Scenario: Downstream blockers stay local

- GIVEN a predecessor or downstream bootstrap stage fails before full `bootstrap/gcc-4.7.ncl` runtime proof exists
- WHEN this part's evidence is reviewed
- THEN the failure does not invalidate this part's completed source-hardening evidence
- AND a separate part change tracks the predecessor or downstream runtime failure

### Requirement: Live-bootstrap part gmp 6.2.1 is independently tracked
Mantle MUST track the live-bootstrap part `gmp 6.2.1` as an independent bootstrap change bound to `bootstrap/gmp-6.2.1.ncl`.
ID: bootstrap.part.gmp.6.2.1

The part scope MUST include only the source pins, patches, derivation logic, output contract, and evidence needed for `bootstrap/gmp-6.2.1.ncl` and direct predecessor compatibility. Broader chain validation MAY depend on this part, but MUST NOT replace this part's own build and smoke evidence. If declared predecessor outputs are unavailable or unvalidated, the part MUST record prerequisite-gated evidence and MUST NOT claim build, smoke, leakage-clean, library-provider, or chain-promotion success.

#### Scenario: Part evidence is isolated

- GIVEN implementation work touches `bootstrap/gmp-6.2.1.ncl`
- WHEN the part is marked complete
- THEN the change records source-pin audit evidence for `bootstrap/gmp-6.2.1.ncl`
- AND it records either a successful `mantle build bootstrap/gmp-6.2.1.ncl` transcript or explicit prerequisite-gated build evidence
- AND it records either a smoke check for the produced output contract or explicit evidence that no output path exists yet
- AND it does not substitute host GMP, Nix-provided GMP, or legacy library outputs for bootstrap proof

#### Scenario: Downstream blockers stay local

- GIVEN a predecessor or downstream bootstrap stage fails before full `bootstrap/gmp-6.2.1.ncl` runtime proof exists
- WHEN this part's evidence is reviewed
- THEN the failure does not invalidate this part's completed source-hardening evidence
- AND a separate part change tracks the predecessor or downstream runtime failure

### Requirement: Live-bootstrap part gzip 1.2.4 is independently tracked
Mantle MUST track the live-bootstrap part `gzip 1.2.4` as an independent bootstrap change bound to `bootstrap/gzip-tcc.ncl`.
ID: bootstrap.part.gzip.1.2.4

The part scope MUST include only the source pins, patches, derivation logic, output contract, and evidence needed for `bootstrap/gzip-tcc.ncl` and direct predecessor compatibility. Broader chain validation MAY depend on this part, but MUST NOT replace this part's own build and smoke evidence. The change MUST record that upstream `parts.rst` labels the section `gzip 1.2.5` while the implemented step/source is `gzip 1.2.4`. If declared predecessor outputs are unavailable or unvalidated, the part MUST record prerequisite-gated evidence and MUST NOT claim build, smoke, leakage-clean, tool-provider, or chain-promotion success.

#### Scenario: Part evidence is isolated

- GIVEN implementation work touches `bootstrap/gzip-tcc.ncl`
- WHEN the part is marked complete
- THEN the change records source-pin audit evidence for `bootstrap/gzip-tcc.ncl`
- AND it records either a successful `mantle build bootstrap/gzip-tcc.ncl` transcript or explicit prerequisite-gated build evidence
- AND it records either a smoke check for the produced output contract or explicit evidence that no output path exists yet
- AND it does not substitute host gzip, Nix-provided gzip, or legacy tool outputs for bootstrap proof

#### Scenario: Downstream blockers stay local

- GIVEN a predecessor or downstream bootstrap stage fails before full `bootstrap/gzip-tcc.ncl` runtime proof exists
- WHEN this part's evidence is reviewed
- THEN the failure does not invalidate this part's completed source-hardening evidence
- AND a separate part change tracks the predecessor or downstream runtime failure

### Requirement: Live-bootstrap part heirloom devtools is independently tracked
Mantle MUST track the live-bootstrap part `heirloom devtools` as an independent bootstrap change bound to `bootstrap/heirloom-devtools.ncl`.
ID: bootstrap.part.heirloom.devtools

The part scope MUST include only the source pins, patches, derivation logic, output contract, and evidence needed for `bootstrap/heirloom-devtools.ncl` and direct predecessor compatibility. Broader chain validation MAY depend on this part, but MUST NOT replace this part's own build and smoke evidence. If declared predecessor outputs are unavailable or unvalidated, the part MUST record prerequisite-gated evidence and MUST NOT claim build, smoke, leakage-clean, tool-provider, or chain-promotion success.

#### Scenario: Part evidence is isolated

- GIVEN implementation work touches `bootstrap/heirloom-devtools.ncl`
- WHEN the part is marked complete
- THEN the change records source-pin audit evidence for `bootstrap/heirloom-devtools.ncl`
- AND it records either a successful `mantle build bootstrap/heirloom-devtools.ncl` transcript or explicit prerequisite-gated build evidence
- AND it records either a smoke check for the produced output contract or explicit evidence that no output path exists yet
- AND it does not substitute host yacc/lex, Nix-provided yacc/lex, or legacy tool outputs for bootstrap proof

#### Scenario: Downstream blockers stay local

- GIVEN a predecessor or downstream bootstrap stage fails before full `bootstrap/heirloom-devtools.ncl` runtime proof exists
- WHEN this part's evidence is reviewed
- THEN the failure does not invalidate this part's completed source-hardening evidence
- AND a separate part change tracks the predecessor or downstream runtime failure

### Requirement: Live-bootstrap part libtool 2.2.4 is independently tracked
Mantle MUST track the live-bootstrap part `libtool 2.2.4` as an independent bootstrap change bound to `bootstrap/libtool-2.2.4.ncl`.
ID: bootstrap.part.libtool.2.2.4

The part scope MUST include only the source pins, patches, derivation logic, output contract, and evidence needed for `bootstrap/libtool-2.2.4.ncl` and direct predecessor compatibility. Broader chain validation MAY depend on this part, but MUST NOT replace this part's own build and smoke evidence. If declared predecessor outputs are unavailable or unvalidated, the part MUST record prerequisite-gated evidence and MUST NOT claim build, smoke, leakage-clean, tool-provider, or chain-promotion success.

#### Scenario: Part evidence is isolated

- GIVEN implementation work touches `bootstrap/libtool-2.2.4.ncl`
- WHEN the part is marked complete
- THEN the change records source-pin audit evidence for `bootstrap/libtool-2.2.4.ncl`
- AND it records either a successful `mantle build bootstrap/libtool-2.2.4.ncl` transcript or explicit prerequisite-gated build evidence
- AND it records either a smoke check for the produced output contract or explicit evidence that no output path exists yet
- AND it does not substitute host libtool, Nix-provided libtool, or legacy tool outputs for bootstrap proof

#### Scenario: Downstream blockers stay local

- GIVEN a predecessor or downstream bootstrap stage fails before full `bootstrap/libtool-2.2.4.ncl` runtime proof exists
- WHEN this part's evidence is reviewed
- THEN the failure does not invalidate this part's completed source-hardening evidence
- AND a separate part change tracks the predecessor or downstream runtime failure

### Requirement: Live-bootstrap part m4 1.4.7 is independently tracked
Mantle MUST track the live-bootstrap part `m4 1.4.7` as an independent bootstrap change bound to `bootstrap/m4-1.4.7-musl.ncl`.
ID: bootstrap.part.m4.1.4.7

The part scope MUST include only the source pins, patches, derivation logic, output contract, and evidence needed for `bootstrap/m4-1.4.7-musl.ncl` and direct predecessor compatibility. Broader chain validation MAY depend on this part, but MUST NOT replace this part's own build and smoke evidence. If declared predecessor outputs are unavailable or unvalidated, or if the current derivation intentionally uses a bootstrap bridge instead of a full direct GNU m4 binary, the part MUST record gate evidence and MUST NOT claim full toolchain promotion success.

#### Scenario: Part evidence is isolated

- GIVEN implementation work touches `bootstrap/m4-1.4.7-musl.ncl`
- WHEN the part is marked complete
- THEN the change records source-pin audit evidence for `bootstrap/m4-1.4.7-musl.ncl`
- AND it records either a successful `mantle build bootstrap/m4-1.4.7-musl.ncl` transcript or explicit prerequisite/runtime-gated build evidence
- AND it records either a smoke check for the produced output contract or explicit evidence that no full direct-build output path exists yet
- AND it does not substitute host m4, Nix-provided m4, or legacy tool outputs for bootstrap proof

#### Scenario: Downstream blockers stay local

- GIVEN a predecessor, direct GNU m4, or downstream bootstrap stage fails before full `bootstrap/m4-1.4.7-musl.ncl` runtime proof exists
- WHEN this part's evidence is reviewed
- THEN the failure does not invalidate this part's completed source-hardening evidence
- AND a separate part change tracks the predecessor, direct-build, or downstream runtime failure

### Requirement: Live-bootstrap part mpfr 4.1.0 is independently tracked
Mantle MUST track the live-bootstrap part `mpfr 4.1.0` as an independent bootstrap change bound to `bootstrap/mpfr-4.1.0.ncl`.
ID: bootstrap.part.mpfr.4.1.0

The part scope MUST include only the source pins, patches, derivation logic, output contract, and evidence needed for `bootstrap/mpfr-4.1.0.ncl` and direct predecessor compatibility. Broader chain validation MAY depend on this part, but MUST NOT replace this part's own build and smoke evidence. If declared predecessor outputs are unavailable or unvalidated, the part MUST record prerequisite-gated evidence and MUST NOT claim build, smoke, leakage-clean, tool-provider, or chain-promotion success.

#### Scenario: Part evidence is isolated

- GIVEN implementation work touches `bootstrap/mpfr-4.1.0.ncl`
- WHEN the part is marked complete
- THEN the change records source-pin audit evidence for `bootstrap/mpfr-4.1.0.ncl`
- AND it records either a successful `mantle build bootstrap/mpfr-4.1.0.ncl` transcript or explicit prerequisite-gated build evidence
- AND it records either a smoke check for the produced output contract or explicit evidence that no output path exists yet
- AND it does not substitute host MPFR/GMP/GCC, Nix-provided libraries, or legacy tool outputs for bootstrap proof

#### Scenario: Downstream blockers stay local

- GIVEN a predecessor or downstream bootstrap stage fails before full `bootstrap/mpfr-4.1.0.ncl` runtime proof exists
- WHEN this part's evidence is reviewed
- THEN the failure does not invalidate this part's completed source-hardening evidence
- AND a separate part change tracks the predecessor or downstream runtime failure

### Requirement: Live-bootstrap part musl 1.2.5 full is independently tracked
Mantle MUST track the live-bootstrap part `musl 1.2.5` as an independent bootstrap change bound to `bootstrap/musl-full.ncl`.
ID: bootstrap.part.musl.1.2.5.full

The part scope MUST include only the source pins, patches, derivation logic, output contract, and evidence needed for `bootstrap/musl-full.ncl` and direct predecessor compatibility. Broader chain validation MAY depend on this part, but MUST NOT replace this part's own build and smoke evidence. If declared predecessor outputs are unavailable or unvalidated, the part MUST record prerequisite-gated evidence and MUST NOT claim build, smoke, leakage-clean, tool-provider, or chain-promotion success.

#### Scenario: Part evidence is isolated

- GIVEN implementation work touches `bootstrap/musl-full.ncl`
- WHEN the part is marked complete
- THEN the change records source-pin audit evidence for `bootstrap/musl-full.ncl`
- AND it records either a successful `mantle build bootstrap/musl-full.ncl` transcript or explicit prerequisite-gated build evidence
- AND it records either a smoke check for the produced output contract or explicit evidence that no output path exists yet
- AND it does not substitute host musl/GCC, Nix-provided libc objects, or legacy tool outputs for bootstrap proof

#### Scenario: Downstream blockers stay local

- GIVEN a predecessor or downstream bootstrap stage fails before full `bootstrap/musl-full.ncl` runtime proof exists
- WHEN this part's evidence is reviewed
- THEN the failure does not invalidate this part's completed source-hardening evidence
- AND a separate part change tracks the predecessor or downstream runtime failure

### Requirement: Live-bootstrap part oyacc 6.6 is independently tracked
Mantle MUST track the live-bootstrap part `oyacc 6.6` as an independent bootstrap change bound to `bootstrap/oyacc-tcc.ncl`.
ID: bootstrap.part.oyacc.6.6

The part scope MUST include only the source pins, patches, derivation logic, output contract, and evidence needed for `bootstrap/oyacc-tcc.ncl` and direct predecessor compatibility. Broader chain validation MAY depend on this part, but MUST NOT replace this part's own build and smoke evidence. If declared predecessor outputs are unavailable or unvalidated, the part MUST record prerequisite-gated evidence and MUST NOT claim build, smoke, leakage-clean, tool-provider, or chain-promotion success.

#### Scenario: Part evidence is isolated

- GIVEN implementation work touches `bootstrap/oyacc-tcc.ncl`
- WHEN the part is marked complete
- THEN the change records source-pin audit evidence for `bootstrap/oyacc-tcc.ncl`
- AND it records either a successful `mantle build bootstrap/oyacc-tcc.ncl` transcript or explicit prerequisite-gated build evidence
- AND it records either a smoke check for the produced output contract or explicit evidence that no output path exists yet
- AND it does not substitute host yacc, Nix-provided yacc, or legacy parser-generator outputs for bootstrap proof

#### Scenario: Downstream blockers stay local

- GIVEN a predecessor or downstream bootstrap stage fails before full `bootstrap/oyacc-tcc.ncl` runtime proof exists
- WHEN this part's evidence is reviewed
- THEN the failure does not invalidate this part's completed source-hardening evidence
- AND a separate part change tracks the predecessor or downstream runtime failure

### Requirement: Live-bootstrap part perl 5.000 is independently tracked
Mantle MUST track the live-bootstrap part `perl 5.000` as an independent bootstrap change bound to `bootstrap/perl-5.000-musl.ncl`.
ID: bootstrap.part.perl.5.000

The part scope MUST include only the source pins, patches, derivation logic, output contract, and evidence needed for `bootstrap/perl-5.000-musl.ncl` and direct predecessor compatibility. Broader chain validation MAY depend on this part, but MUST NOT replace this part's own build and smoke evidence. If declared predecessor outputs are unavailable or unvalidated, the part MUST record prerequisite-gated evidence and MUST NOT claim build, smoke, leakage-clean, tool-provider, or chain-promotion success.

#### Scenario: Part evidence is isolated

- GIVEN implementation work touches `bootstrap/perl-5.000-musl.ncl`
- WHEN the part is marked complete
- THEN the change records source-pin audit evidence for `bootstrap/perl-5.000-musl.ncl`
- AND it records either a successful `mantle build bootstrap/perl-5.000-musl.ncl` transcript or explicit prerequisite-gated build evidence
- AND it records either a smoke check for the produced output contract or explicit evidence that no output path exists yet
- AND it does not substitute host Perl, Nix-provided Perl, or legacy interpreter outputs for bootstrap proof

#### Scenario: Downstream blockers stay local

- GIVEN a predecessor or downstream bootstrap stage fails before full `bootstrap/perl-5.000-musl.ncl` runtime proof exists
- WHEN this part's evidence is reviewed
- THEN the failure does not invalidate this part's completed source-hardening evidence
- AND a separate part change tracks the predecessor or downstream runtime failure

### Requirement: Live-bootstrap part perl 5.003 is independently tracked
Mantle MUST track the live-bootstrap part `perl 5.003` as an independent bootstrap change bound to `bootstrap/perl-5.003-musl.ncl`.
ID: bootstrap.part.perl.5.003

The part scope MUST include only the source pins, patches, derivation logic, output contract, and evidence needed for `bootstrap/perl-5.003-musl.ncl` and direct predecessor compatibility. Broader chain validation MAY depend on this part, but MUST NOT replace this part's own build and smoke evidence. If declared predecessor outputs are unavailable or unvalidated, the part MUST record prerequisite-gated evidence and MUST NOT claim build, smoke, leakage-clean, tool-provider, or chain-promotion success.

#### Scenario: Part evidence is isolated

- GIVEN implementation work touches `bootstrap/perl-5.003-musl.ncl`
- WHEN the part is marked complete
- THEN the change records source-pin audit evidence for `bootstrap/perl-5.003-musl.ncl`
- AND it records either a successful `mantle build bootstrap/perl-5.003-musl.ncl` transcript or explicit prerequisite-gated build evidence
- AND it records either a smoke check for the produced output contract or explicit evidence that no output path exists yet
- AND it does not substitute host Perl, Nix-provided Perl, or legacy interpreter outputs for bootstrap proof

#### Scenario: Downstream blockers stay local

- GIVEN a predecessor or downstream bootstrap stage fails before full `bootstrap/perl-5.003-musl.ncl` runtime proof exists
- WHEN this part's evidence is reviewed
- THEN the failure does not invalidate this part's completed source-hardening evidence
- AND a separate part change tracks the predecessor or downstream runtime failure

### Requirement: Live-bootstrap part perl 5.004_05 is independently tracked
Mantle MUST track the live-bootstrap part `perl 5.004_05` as an independent bootstrap change bound to `bootstrap/perl-5.004_05-musl.ncl`.
ID: bootstrap.part.perl.5.004.05

The part scope MUST include only the source pins, patches, derivation logic, output contract, and evidence needed for `bootstrap/perl-5.004_05-musl.ncl` and direct predecessor compatibility. Broader chain validation MAY depend on this part, but MUST NOT replace this part's own build and smoke evidence. If declared predecessor outputs are unavailable or unvalidated, the part MUST record prerequisite-gated evidence and MUST NOT claim build, smoke, leakage-clean, tool-provider, or chain-promotion success.

#### Scenario: Part evidence is isolated

- GIVEN implementation work touches `bootstrap/perl-5.004_05-musl.ncl`
- WHEN the part is marked complete
- THEN the change records source-pin audit evidence for `bootstrap/perl-5.004_05-musl.ncl`
- AND it records either a successful `mantle build bootstrap/perl-5.004_05-musl.ncl` transcript or explicit prerequisite-gated build evidence
- AND it records either a smoke check for the produced output contract or explicit evidence that no output path exists yet
- AND it does not substitute host Perl, Nix-provided Perl, or legacy interpreter outputs for bootstrap proof

#### Scenario: Downstream blockers stay local

- GIVEN a predecessor or downstream bootstrap stage fails before full `bootstrap/perl-5.004_05-musl.ncl` runtime proof exists
- WHEN this part's evidence is reviewed
- THEN the failure does not invalidate this part's completed source-hardening evidence
- AND a separate part change tracks the predecessor or downstream runtime failure

### Requirement: Live-bootstrap part perl 5.005_03 is independently tracked
Mantle MUST track the live-bootstrap part `perl 5.005_03` as an independent bootstrap change bound to `bootstrap/perl-5.005_03-musl.ncl`.
ID: bootstrap.part.perl.5.005.03

The part scope MUST include only the source pins, patches, derivation logic, output contract, and evidence needed for `bootstrap/perl-5.005_03-musl.ncl` and direct predecessor compatibility. Broader chain validation MAY depend on this part, but MUST NOT replace this part's own build and smoke evidence. If declared predecessor outputs are unavailable or unvalidated, the part MUST record prerequisite-gated evidence and MUST NOT claim build, smoke, leakage-clean, tool-provider, or chain-promotion success.

#### Scenario: Part evidence is isolated

- GIVEN implementation work touches `bootstrap/perl-5.005_03-musl.ncl`
- WHEN the part is marked complete
- THEN the change records source-pin audit evidence for `bootstrap/perl-5.005_03-musl.ncl`
- AND it records either a successful `mantle build bootstrap/perl-5.005_03-musl.ncl` transcript or explicit prerequisite-gated build evidence
- AND it records either a smoke check for the produced output contract or explicit evidence that no output path exists yet
- AND it does not substitute host Perl, Nix-provided Perl, or legacy interpreter outputs for bootstrap proof

#### Scenario: Downstream blockers stay local

- GIVEN a predecessor or downstream bootstrap stage fails before full `bootstrap/perl-5.005_03-musl.ncl` runtime proof exists
- WHEN this part's evidence is reviewed
- THEN the failure does not invalidate this part's completed source-hardening evidence
- AND a separate part change tracks the predecessor or downstream runtime failure

### Requirement: Live-bootstrap part perl 5.6.2 is independently tracked
Mantle MUST track the live-bootstrap part `perl 5.6.2` as an independent bootstrap change bound to `bootstrap/perl-5.6.2-musl.ncl`.
ID: bootstrap.part.perl.5.6.2

The part scope MUST include only the source pins, patches, derivation logic, output contract, and evidence needed for `bootstrap/perl-5.6.2-musl.ncl` and direct predecessor compatibility. Broader chain validation MAY depend on this part, but MUST NOT replace this part's own build and smoke evidence. If declared predecessor outputs are unavailable or unvalidated, the part MUST record prerequisite-gated evidence and MUST NOT claim build, smoke, leakage-clean, tool-provider, or chain-promotion success.

#### Scenario: Part evidence is isolated

- GIVEN implementation work touches `bootstrap/perl-5.6.2-musl.ncl`
- WHEN the part is marked complete
- THEN the change records source-pin audit evidence for `bootstrap/perl-5.6.2-musl.ncl`
- AND it records either a successful `mantle build bootstrap/perl-5.6.2-musl.ncl` transcript or explicit prerequisite-gated build evidence
- AND it records either a smoke check for the produced output contract or explicit evidence that no output path exists yet
- AND it does not substitute host Perl, Nix-provided Perl, or legacy interpreter outputs for bootstrap proof

#### Scenario: Downstream blockers stay local

- GIVEN a predecessor or downstream bootstrap stage fails before full `bootstrap/perl-5.6.2-musl.ncl` runtime proof exists
- WHEN this part's evidence is reviewed
- THEN the failure does not invalidate this part's completed source-hardening evidence
- AND a separate part change tracks the predecessor or downstream runtime failure

### Requirement: Live-bootstrap part sed 4.0.9 (musl) is independently tracked
Mantle MUST track the live-bootstrap part `sed 4.0.9` as an independent bootstrap change bound to `bootstrap/sed-4.0.9-musl.ncl`.
ID: bootstrap.part.sed.4.0.9.musl

The part scope MUST include only the source pins, patches, derivation logic, output contract, and evidence needed for `bootstrap/sed-4.0.9-musl.ncl` and direct predecessor compatibility. Broader chain validation MAY depend on this part, but MUST NOT replace this part's own build and smoke evidence. Because this derivation currently preserves an explicit `sed-tcc` bridge while the TinyCC/musl source compile boundary is blocked, the part MUST record bridge/gate evidence and MUST NOT claim musl source-build, leakage-clean, tool-provider, or chain-promotion success.

#### Scenario: Part evidence is isolated

- GIVEN implementation work touches `bootstrap/sed-4.0.9-musl.ncl`
- WHEN the part is marked complete
- THEN the change records source-pin audit evidence for `bootstrap/sed-4.0.9-musl.ncl`
- AND it records source-pin audit evidence for `bootstrap/sed-4.0.9-musl.ncl`
- AND it records explicit bridge/gate evidence for the `sed-tcc` runtime copy
- AND it does not substitute the bridge output for musl source-build proof

#### Scenario: Downstream blockers stay local

- GIVEN the TinyCC/musl sed source compile boundary remains blocked or a downstream bootstrap stage fails
- WHEN this part's evidence is reviewed
- THEN the failure does not invalidate this part's completed source-hardening and bridge-gate evidence
- AND a separate part change tracks the compile-boundary or downstream runtime failure

### Requirement: Live-bootstrap part source-built full seed normalization is independently tracked
Mantle MUST track the live-bootstrap part `gcc 10.5.0 through binutils 2.41` as an independent bootstrap change bound to `bootstrap/seed-full.ncl`.
ID: bootstrap.part.seed.full

The part scope MUST include only the source pins, patches, derivation logic, output contract, and evidence needed for `bootstrap/seed-full.ncl` and direct predecessor compatibility. Broader chain validation MAY depend on this part, but MUST NOT replace this part's own build and smoke evidence. If declared predecessor toolchains are unavailable or unvalidated, the part MUST record prerequisite-gated evidence and MUST NOT claim full-source seed promotion, leakage-clean runtime proof, or end-to-end source-built success.

#### Scenario: Part evidence is isolated

- GIVEN implementation work touches `bootstrap/seed-full.ncl`
- WHEN the part is marked complete
- THEN the change records source-pin audit evidence for `bootstrap/seed-full.ncl`
- AND it records the normalized seed output contract and source-chain audit evidence
- AND it records either a successful `mantle build bootstrap/seed-full.ncl` transcript or explicit prerequisite-gated build evidence
- AND it does not promote the full-source seed until GCC 10.5.0, musl 1.2.5, and binutils 2.41 predecessor proofs are trusted

#### Scenario: Downstream blockers stay local

- GIVEN a predecessor or downstream bootstrap stage fails before full `bootstrap/seed-full.ncl` promotion proof exists
- WHEN this part's evidence is reviewed
- THEN the failure does not invalidate this part's completed normalization-contract evidence
- AND a separate part change tracks the predecessor or downstream runtime failure

### Requirement: Live-bootstrap part tar 1.12 is independently tracked
Mantle MUST track the live-bootstrap part `tar 1.12` as an independent bootstrap change bound to `bootstrap/tar-tcc.ncl`.
ID: bootstrap.part.tar.1.12

The part scope MUST include only the source pins, patches, derivation logic, output contract, and evidence needed for `bootstrap/tar-tcc.ncl` and direct predecessor compatibility. Broader chain validation MAY depend on this part, but MUST NOT replace this part's own build and smoke evidence. If declared predecessor outputs are unavailable or unvalidated, the part MUST record prerequisite-gated evidence and MUST NOT claim build, smoke, leakage-clean, tool-provider, or chain-promotion success.

#### Scenario: Part evidence is isolated

- GIVEN implementation work touches `bootstrap/tar-tcc.ncl`
- WHEN the part is marked complete
- THEN the change records source-pin audit evidence for `bootstrap/tar-tcc.ncl`
- AND it records either a successful `mantle build bootstrap/tar-tcc.ncl` transcript or explicit prerequisite-gated build evidence
- AND it records either a smoke check for the produced output contract or explicit evidence that no output path exists yet
- AND it does not substitute host tar, Nix-provided tar, or legacy archive tools for bootstrap proof

#### Scenario: Downstream blockers stay local

- GIVEN a predecessor or downstream bootstrap stage fails before full `bootstrap/tar-tcc.ncl` runtime proof exists
- WHEN this part's evidence is reviewed
- THEN the failure does not invalidate this part's completed source-hardening evidence
- AND a separate part change tracks the predecessor or downstream runtime failure

### Requirement: Bootstrap blocker inventory is deterministic [r[bootstrap.blocker-inventory.deterministic]]

Mantle MUST provide a checked-in bootstrap blocker inventory gate that deterministically derives remaining full-source bootstrap blockers from repository-controlled sources.

The inventory MUST classify at least bridge outputs, placeholder or normalization-only providers, legacy-provider fallback, host-tool fallback, prerequisite-gated evidence, and known compiler/runtime crash boundaries. Each finding MUST include a stable marker class, source path, line or artifact locator when available, and a short explanation of the blocked promotion claim.

#### Scenario: Current gated tree produces an inventory [r[bootstrap.blocker-inventory.deterministic.current-tree]]

- GIVEN the current repository still contains bootstrap bridge and prerequisite-gated markers
- WHEN the blocker inventory gate runs in report mode
- THEN it exits successfully without claiming full-source readiness
- AND it emits JSON and Markdown summaries containing every configured marker class found in the tree
- AND the report identifies `bootstrap/seed-full.ncl` or its provider status as gated rather than promoted

#### Scenario: Inventory is stable for automation [r[bootstrap.blocker-inventory.deterministic.stable-output]]

- GIVEN no blocker-relevant source files changed
- WHEN the blocker inventory gate runs twice
- THEN the JSON report contains stable ordering for marker classes and findings
- AND the Markdown report is suitable for checked-in evidence without timestamps or host-specific paths

#### Scenario: Matcher regressions avoid generic C token noise [r[bootstrap.blocker-inventory.deterministic.matcher-regressions]]

- GIVEN bootstrap sources contain ordinary C tokens such as `signal.h`, `strsignal`, and `static` helper declarations
- WHEN the blocker inventory gate runs its built-in matcher self-tests
- THEN those generic tokens are not classified as compiler/runtime crash boundaries
- AND concrete blocker phrases such as `segfault`, `rc=139`, `exit 139`, `timeout`, `signal-derived`, and `static link` remain classified as compiler/runtime crash boundaries

### Requirement: Full-source promotion claims fail closed while blockers remain [r[bootstrap.blocker-inventory.promotion-drift]]

Mantle MUST fail a bootstrap readiness or promotion check when repository-controlled status text, manifests, reports, or seed-provider metadata claim full-source bootstrap readiness while configured blocker markers remain present.

The failure MUST name the conflicting promotion claim and at least one remaining blocker class. It MUST NOT require running the heavyweight self-hosting proof to reject an inconsistent readiness claim.

#### Scenario: Promotion drift is rejected [r[bootstrap.blocker-inventory.promotion-drift.rejects-conflict]]

- GIVEN a fixture or mutation that marks the full-source provider as promoted
- AND a known bridge, placeholder, fallback, or prerequisite-gated blocker remains
- WHEN the blocker inventory gate runs in enforcement mode
- THEN it exits nonzero
- AND the diagnostic names the promotion claim and remaining blocker class

#### Scenario: Gated status remains allowed [r[bootstrap.blocker-inventory.promotion-drift.allows-gated-status]]

- GIVEN the source tree explicitly labels full-source bootstrap status as gated
- AND blocker markers remain present
- WHEN the blocker inventory gate runs in enforcement mode
- THEN it does not fail merely because blockers exist
- AND it records the blockers as readiness debt rather than promotion evidence

### Requirement: Blocker taxonomy has a retirement workflow [r[bootstrap.blocker-inventory.taxonomy-retirement]]

The blocker inventory gate MUST document how marker classes are added, updated, and retired when a blocker is repaired. Retiring a marker class MUST require positive evidence for the repaired boundary and a negative drift check proving that overclaiming still fails for any remaining classes.

#### Scenario: Repaired blocker can be retired with evidence [r[bootstrap.blocker-inventory.taxonomy-retirement.evidence]]

- GIVEN a bootstrap blocker has been repaired with source-built positive evidence
- WHEN its marker class is removed or narrowed
- THEN the change includes the repair evidence path
- AND the remaining inventory still runs deterministically
- AND the promotion-drift negative fixture still fails for any remaining blocker class

### Requirement: GCC 4.0 Correctness Promotion Plan [r[gcc40-correctness-roadmap]]
Mantle MUST track GCC 4.0 correctness promotion separately from pass1 graph completion, with verifiable milestones for replacing stubs and adding semantic smokes.

#### Scenario: Graph completion caveat remains explicit [r[gcc40-correctness-roadmap.1]]
- GIVEN the GCC 4.0 artifact builds successfully
- WHEN the correctness promotion plan is reviewed
- THEN it distinguishes bridge graph-completion from native/self-hosted/correct GCC behavior

#### Scenario: Milestones are independently verifiable [r[gcc40-correctness-roadmap.2]]
- GIVEN a future implementation task is selected
- WHEN its verification is run
- THEN the task proves a specific semantic or executable behavior rather than broad completion

### Requirement: GCC 4.0 Libgcc Member Semantics [r[gcc40-libgcc-semantic-member]]
Mantle MUST be able to promote individual GCC 4.0 `libgcc.a` members from placeholder bodies to verified semantics without requiring a full native GCC rewrite in the same change.

#### Scenario: One member has non-placeholder semantics [r[gcc40-libgcc-semantic-member.1]]
- GIVEN the GCC 4.0 bootstrap artifact is built
- WHEN the selected `libgcc.a` member is extracted and smoke-tested
- THEN its behavior matches the documented semantics and is not merely `return 0`

#### Scenario: Archive shape remains valid [r[gcc40-libgcc-semantic-member.2]]
- GIVEN the member is promoted
- WHEN host `ar` and `nm` inspect `libgcc.a`
- THEN expected member names and symbols remain visible

### Requirement: GCC 4.0 Muldi3 Libgcc Semantics [r[gcc40-libgcc-muldi3-semantics]]
Mantle MUST be able to promote `_muldi3` from a placeholder body to verified signed 64-bit multiplication semantics without broadening the GCC 4.0 pass1 bridge into a full native GCC rewrite.

#### Scenario: Muldi3 has non-placeholder semantics [r[gcc40-libgcc-muldi3-semantics.1]]
- GIVEN the GCC 4.0 bootstrap artifact is built
- WHEN `_muldi3.o` is extracted from the produced `libgcc.a`
- THEN the symbol table exposes `_muldi3`
- AND a semantic smoke linked against `libgcc.a` proves representative positive, negative, and zero 64-bit products

#### Scenario: Archive shape remains deterministic [r[gcc40-libgcc-muldi3-semantics.2]]
- GIVEN `_muldi3` is promoted
- WHEN the produced `libgcc.a` is inspected
- THEN existing promoted `_negdi2` remains present
- AND the archive continues to use the deterministic hand-written ar(5) member list

### Requirement: GCC 4.0 Lshrdi3 Libgcc Semantics [r[gcc40-libgcc-lshrdi3-semantics]]
Mantle MUST be able to promote `_lshrdi3` from a placeholder body to verified unsigned 64-bit logical-right-shift semantics without broadening the GCC 4.0 pass1 bridge into a full native GCC rewrite.

#### Scenario: Lshrdi3 has non-placeholder semantics [r[gcc40-libgcc-lshrdi3-semantics.1]]
- GIVEN the GCC 4.0 bootstrap artifact is built
- WHEN `_lshrdi3.o` is extracted from the produced `libgcc.a`
- THEN the symbol table exposes `_lshrdi3`
- AND a semantic smoke linked against `libgcc.a` proves representative unsigned 64-bit logical right shifts, including high-bit values, zero shifts, and wider shift counts below 64

#### Scenario: Archive shape remains deterministic [r[gcc40-libgcc-lshrdi3-semantics.2]]
- GIVEN `_lshrdi3` is promoted
- WHEN the produced `libgcc.a` is inspected
- THEN existing promoted `_negdi2` and `_muldi3` remain present
- AND the archive continues to use the deterministic hand-written ar(5) member list

### Requirement: GCC 4.0 Ashldi3 Libgcc Semantics [r[gcc40-libgcc-ashldi3-semantics]]
Mantle MUST be able to promote `_ashldi3` from a placeholder body to verified 64-bit left-shift semantics without broadening the GCC 4.0 pass1 bridge into a full native GCC rewrite.

#### Scenario: Ashldi3 has non-placeholder semantics [r[gcc40-libgcc-ashldi3-semantics.1]]
- GIVEN the GCC 4.0 bootstrap artifact is built
- WHEN `_ashldi3.o` is extracted from the produced `libgcc.a`
- THEN the symbol table exposes `_ashldi3`
- AND a semantic smoke linked against `libgcc.a` proves representative 64-bit left shifts, including zero shifts, low-bit movement, and high-bit-producing counts below 64

#### Scenario: Archive shape remains deterministic [r[gcc40-libgcc-ashldi3-semantics.2]]
- GIVEN `_ashldi3` is promoted
- WHEN the produced `libgcc.a` is inspected
- THEN existing promoted `_negdi2`, `_muldi3`, and `_lshrdi3` remain present
- AND the archive continues to use the deterministic hand-written ar(5) member list

### Requirement: GCC 4.0 Ashrdi3 Libgcc Semantics [r[gcc40-libgcc-ashrdi3-semantics]]
Mantle MUST be able to promote `_ashrdi3` from a placeholder body to verified signed 64-bit arithmetic-right-shift semantics without broadening the GCC 4.0 pass1 bridge into a full native GCC rewrite.

#### Scenario: Ashrdi3 has non-placeholder semantics [r[gcc40-libgcc-ashrdi3-semantics.1]]
- GIVEN the GCC 4.0 bootstrap artifact is built
- WHEN `_ashrdi3.o` is extracted from the produced `libgcc.a`
- THEN the symbol table exposes `_ashrdi3`
- AND a semantic smoke linked against `libgcc.a` proves representative signed 64-bit arithmetic right shifts, including negative sign extension, positive values, zero shifts, and wider shift counts below 64

#### Scenario: Archive shape remains deterministic [r[gcc40-libgcc-ashrdi3-semantics.2]]
- GIVEN `_ashrdi3` is promoted
- WHEN the produced `libgcc.a` is inspected
- THEN existing promoted `_negdi2`, `_muldi3`, `_lshrdi3`, and `_ashldi3` remain present
- AND the archive continues to use the deterministic hand-written ar(5) member list

### Requirement: GCC 4.0 Cmpdi2 Libgcc Semantics [r[gcc40-libgcc-cmpdi2-semantics]]
Mantle MUST be able to promote `_cmpdi2` from a placeholder body to verified signed 64-bit comparison semantics without broadening the GCC 4.0 pass1 bridge into a full native GCC rewrite.

#### Scenario: Cmpdi2 has non-placeholder semantics [r[gcc40-libgcc-cmpdi2-semantics.1]]
- GIVEN the GCC 4.0 bootstrap artifact is built
- WHEN `_cmpdi2.o` is extracted from the produced `libgcc.a`
- THEN the symbol table exposes `_cmpdi2`
- AND a semantic smoke linked against `libgcc.a` proves the GCC libgcc signed comparison contract: 0 for less-than, 1 for equality, and 2 for greater-than across representative negative, zero, and positive 64-bit values

#### Scenario: Archive shape remains deterministic [r[gcc40-libgcc-cmpdi2-semantics.2]]
- GIVEN `_cmpdi2` is promoted
- WHEN the produced `libgcc.a` is inspected
- THEN existing promoted `_negdi2`, `_muldi3`, `_lshrdi3`, `_ashldi3`, and `_ashrdi3` remain present
- AND the archive continues to use the deterministic hand-written ar(5) member list

### Requirement: GCC 4.0 Ucmpdi2 Libgcc Semantics [r[gcc40-libgcc-ucmpdi2-semantics]]
Mantle MUST be able to promote `_ucmpdi2` from a placeholder body to verified unsigned 64-bit comparison semantics without broadening the GCC 4.0 pass1 bridge into a full native GCC rewrite.

#### Scenario: Ucmpdi2 has non-placeholder semantics [r[gcc40-libgcc-ucmpdi2-semantics.1]]
- GIVEN the GCC 4.0 bootstrap artifact is built
- WHEN `_ucmpdi2.o` is extracted from the produced `libgcc.a`
- THEN the symbol table exposes `_ucmpdi2`
- AND a semantic smoke linked against `libgcc.a` proves the GCC libgcc unsigned comparison contract: 0 for less-than, 1 for equality, and 2 for greater-than across representative low, high-bit, and maximum 64-bit values

#### Scenario: Archive shape remains deterministic [r[gcc40-libgcc-ucmpdi2-semantics.2]]
- GIVEN `_ucmpdi2` is promoted
- WHEN the produced `libgcc.a` is inspected
- THEN existing promoted `_negdi2`, `_muldi3`, `_lshrdi3`, `_ashldi3`, `_ashrdi3`, and `_cmpdi2` remain present
- AND the archive continues to use the deterministic hand-written ar(5) member list

### Requirement: Whole-bootstrap parity map

Mantle MUST maintain a canonical whole-bootstrap parity map that covers the complete bootstrap ladder required to claim parity with the reference source-bootstrap ecosystems being used as inputs: live-bootstrap for the concrete seed-to-modern-toolchain stage order, Guix for full-source bootstrap claim semantics and trust-root disclosure, and StageX for the no-quorum audited-seed lineage profile.
ID: bootstrap.parity.map

The map MUST enumerate every planned or implemented bootstrap stage from the audited seed through the normalized seed provider and final self-build proof, including stage0/hex0 material, M0/M1/hex2/kaem-style transition tools, Mes, TinyCC, musl, make, patch, grep, sed, bzip2, gzip, tar, coreutils, diffutils, gawk, bison, flex, m4, libtool, autoconf/automake versions, Perl versions, GMP, MPFR, MPC, binutils generations, GCC 4.0, GCC 4.7, GCC 10, full musl/binutils handoff, seed-full, selftest, integration-test, and the Mantle self-build stages. Each entry MUST name the reference source lineage (`live-bootstrap`, `guix`, `stagex`, or a documented Mantle-specific bridge), source artifact identity, patch set, provider inputs and outputs, implementation derivation, placeholder status, runtime-smoke evidence, proof transcript path or digest, and known deviations.

#### Scenario: Parity map names all reference axes

- GIVEN the bootstrap parity map exists
- WHEN it is validated
- THEN it lists live-bootstrap, Guix full-source bootstrap, and StageX no-quorum axes separately
- AND each axis has explicit completion criteria and evidence fields

#### Scenario: Missing mapped stage blocks parity claim

- GIVEN any mapped stage lacks an implementation derivation, replacement rationale, or proof transcript
- WHEN bootstrap parity is reported
- THEN parity status is `incomplete`
- AND the report names the missing stage and reference axis

#### Scenario: Placeholder stage blocks parity claim

- GIVEN a mapped stage still emits placeholder, bridge-only, or stub output
- WHEN bootstrap parity is reported
- THEN the stage is counted as incomplete unless the map records an accepted replacement rationale and equivalent proof evidence
- AND full live-bootstrap/Guix/StageX parity claims remain disabled

### Requirement: Bootstrap parity gap report

The bootstrap parity report MUST classify intentional GCC 4.0 pass1 bridge markers using a checked placeholder inventory receipt. The receipt MUST use schema `crunch-gcc40-placeholder-inventory-v1`, MUST name `bootstrap/gcc-4.0.ncl`, MUST mark the inventory as `inventory-only`, and MUST enumerate the exact standalone placeholder-marker occurrences currently present in the derivation. The report MUST fail closed when the receipt is missing or when the recomputed marker set differs from the receipt. A matching inventory MAY classify `gcc.4.0` as evidence-backed `partial`, but MUST NOT mark it `complete` or unblock live-bootstrap/Guix parity without native compiler correctness evidence.

#### Scenario: GCC 4.0 placeholder inventory matches

- GIVEN `bootstrap/gcc-4.0.ncl` contains intentional pass1 bridge markers
- AND `bootstrap/evidence/gcc-4.0-placeholder-inventory.json` enumerates the exact marker set
- WHEN the parity report evaluates `gcc.4.0`
- THEN the row reports `partial` rather than unclassified `placeholder`
- AND the row remains a live-bootstrap and Guix blocker

#### Scenario: GCC 4.0 placeholder inventory drifts

- GIVEN a standalone marker is added, removed, moved, or renamed without updating the receipt
- WHEN the parity report evaluates `gcc.4.0`
- THEN the evidence check fails
- AND the row remains a blocker with a drift note

### Requirement: Bootstrap parity map rejects unevidenced binutils bridges [r[bootstrap.parity.binutils-tcc-evidence]]

The parity report MUST fail closed for `binutils.tcc` when the derivation contains explicit standalone placeholder markers, bridge-only notes, missing smoke transcripts, or unchecked evidence references. When `bootstrap/evidence/binutils-tcc-tool-smoke.json` is present, the report MUST accept it only if it is produced from `bootstrap/binutils-tcc.ncl`, names the output path, records provider kind, records no host fallback, and contains successful smoke entries for `as`, `ld`, `ar`, `ranlib`, `nm`, and `objcopy`. Because `binutils.tcc` remains `expected_complete=false` until full native/source correctness is proven, a marker-free derivation plus checked transcript MUST report `partial` rather than `complete`.

#### Scenario: Require checked evidence for promotion [r[bootstrap.parity.binutils-tcc-evidence.require-checked]]

- GIVEN `bootstrap/binutils-tcc.ncl` exists but no checked binutils tool transcript is present
- WHEN `mantle bootstrap parity-report --require live-bootstrap` or `--require guix` runs
- THEN the command fails and identifies `binutils.tcc` as a blocker

#### Scenario: Promote only after tool smokes [r[bootstrap.parity.binutils-tcc-evidence.tool-smokes]]

- GIVEN a checked transcript proves the binutils-tcc output tools
- AND the derivation has no explicit standalone placeholder marker
- WHEN the parity report loads that evidence
- THEN the `binutils.tcc` row reports `partial`
- AND the report does not imply downstream GCC correctness or full-source parity

#### Scenario: Transcript is generated from complete logical closure [r[bootstrap.parity.binutils-tcc-evidence.logical-closure]]

- GIVEN the binutils-tcc output depends on logical `/crunch/store` paths
- WHEN the evidence producer smokes the tools
- THEN `/crunch/store` is bound to the complete scratch store closure used by the build
- AND the transcript records the scratch store root and smoke command exit statuses

### Requirement: Bootstrap parity claim gating

Mantle MUST fail closed on any operator-facing claim that Mantle has reached full live-bootstrap, Guix full-source bootstrap, or StageX no-quorum parity unless the parity map and gap report show every required stage complete with the required provider/proof evidence.
ID: bootstrap.parity.claim-gating

A parity claim MUST be scoped to the exact axis satisfied. Live-bootstrap parity MUST require the complete mapped stage ladder or accepted Mantle-specific replacements. Guix full-source parity MUST require source-built inputs, trust-root documentation, and final source proof comparable to Guix's full-source bootstrap claim semantics. StageX parity MUST require the audited hex0 seed lineage, no prebuilt compiler/tool root, protected execution audit when used, and `stagex-lineage` self-build proof.

GCC 4.0 MUST remain evidence-backed partial unless native compiler correctness is proven, and its checked evidence MUST include a native-frontier receipt that names remaining non-native blockers. The libiberty demangle frontier MUST expose a checked bounded-demangle semantic marker for the supported Itanium zero-argument function slice rather than a disabled-demangle marker.

#### Scenario: GCC 4.0 native frontier remains partial
- GIVEN `bootstrap/evidence/gcc-4.0-native-boundary.json` has `status=boundary-only`
- AND it contains a non-empty `native_frontier.blockers` array with derivation markers for remaining non-native GCC 4.0 seams
- WHEN `mantle bootstrap parity-report` evaluates `gcc.4.0`
- THEN the row remains `partial`
- AND the evidence check passes only if each frontier marker is present in `bootstrap/gcc-4.0.ncl`
- AND the report does not mark live-bootstrap or Guix parity complete for GCC 4.0

#### Scenario: GCC 4.0 libiberty demangle has bounded semantics
- GIVEN `bootstrap/gcc-4.0.ncl` writes the libiberty demangle bridge
- WHEN the derivation-local smoke calls `cplus_demangle` and `cplus_demangle_v3`
- THEN `_Z3foov` demangles to `foo()`
- AND malformed or unsupported names return null
- AND parity checks require `gcc40_cplus_demangle_bounded_itanium_v0_boundary`
- AND parity checks require `gcc40_cp_demangle_bounded_itanium_v0_boundary`
- AND parity checks reject `gcc40_cp_demangle_disabled_boundary`
- AND `gcc.4.0` remains `partial` until native compiler correctness is proven

### Requirement: Bootstrap parity report gates provider rows by axis-specific evidence

Mantle MUST report normalized seed provider parity with enough granularity to avoid treating Guix source-root provider evidence as StageX-class lineage evidence.
ID: bootstrap.parity.provider.axis.evidence

The parity report MUST keep the `seed-full` row scoped to Guix/source-root provider contract evidence. It MUST report StageX-class normalized seed provider evidence as a separate row that remains blocked until lineage proof evidence is present. Completing the Guix `seed-full` row MUST NOT cause `--require stagex` to pass while StageX lineage, self-build, or other StageX blockers remain unresolved.

#### Scenario: Guix seed-full contract completes without StageX overclaim

- GIVEN `bootstrap/seed-full.ncl` exposes a normalized provider contract without legacy fetched-provider metadata
- WHEN `mantle bootstrap parity-report --json` runs
- THEN the `seed-full` row reports complete source-root provider evidence for the Guix axis
- AND a separate StageX seed-provider row remains blocking the StageX axis
- AND `mantle bootstrap parity-report --require stagex` exits non-zero while that StageX row is blocked

#### Scenario: Legacy seed-full metadata remains blocked

- GIVEN the seed-full derivation contains legacy fetched-provider raw metadata or omits normalized provider metadata
- WHEN `mantle bootstrap parity-report --json` runs
- THEN the Guix `seed-full` row blocks parity
- AND the diagnostic notes the missing or legacy provider contract evidence

### Requirement: Scalable Clankers Root Vendor Closure

Mantle MUST represent the root `clankers` Cargo source and vendor closure as a fixed, reproducible input without requiring a monolithic large artifact to be committed to git.

#### Scenario: root closure is larger than small-rung package artifacts
- **GIVEN** offline `cargo vendor --locked --offline --versioned-dirs` for `/home/brittonr/git/clankers`
- **WHEN** the root `clankers` binary derivation is prepared
- **THEN** the implementation records the vendor crate count and byte size
- **AND** the implementation chooses a scalable fixed-input representation before adding `packages/clankers/clankers.ncl`

### Requirement: Root Clankers Binary Build Evidence

Mantle MUST build the root `clankers` binary from the fixed source/vendor closure with offline Cargo and record the output binary and smoke result.

#### Scenario: root binary builds without network
- **GIVEN** the scalable source/vendor closure representation exists
- **WHEN** Mantle builds `packages/clankers/clankers.ncl`
- **THEN** Cargo runs with `--locked --offline`
- **AND** `CARGO_HOME` is sandbox-local
- **AND** `CARGO_TARGET_DIR` is deterministic
- **AND** `$out/bin/clankers` exists
- **AND** a network-free `$out/bin/clankers --help` or `$out/bin/clankers --version` smoke output is recorded

### Requirement: External Rust workspace builds use fixed offline source closures

Mantle MUST support an external Rust workspace build pattern where every workspace source, path dependency, git dependency, registry crate, and tool input is represented by an explicit fixed source closure before derivation execution.
ID: bootstrap.external-rust-workspace.offline-source-closure

The build MUST run Cargo with network disabled, a writable sandbox-local `CARGO_HOME`, a deterministic `CARGO_TARGET_DIR`, and a checked `.cargo/config.toml` or equivalent source replacement that points only at fixed inputs. Host checkout-relative paths MAY be used only to construct the fixed source closure outside the derivation; derivation execution MUST NOT read undeclared sibling checkouts, live git remotes, or ambient Cargo caches.

#### Scenario: Offline source closure builds a simple crate

- GIVEN a fixed source closure for an external Rust workspace
- AND the closure includes registry crates, git dependencies, path dependencies, and workspace sources required by a selected package
- WHEN Mantle builds the package derivation with `CARGO_NET_OFFLINE=true` and `cargo build --locked --offline -p <package>`
- THEN Cargo does not access the network or ambient Cargo caches
- AND the selected package builds successfully from declared inputs only

#### Scenario: Missing git dependency is rejected before success is claimed

- GIVEN a selected package depends on a git source that is not present in the fixed source closure
- WHEN the package derivation or its preflight validation runs
- THEN the build fails before any success receipt is written
- AND the diagnostic names the missing git dependency or source replacement

#### Scenario: Ambient sibling checkout is not accepted as derivation input

- GIVEN a workspace package has a path dependency such as `../subwayrat` or `../ratcore`
- WHEN the Mantle derivation executes
- THEN it reads the dependency from a declared fixed input
- AND it does not read the live sibling checkout path from the host filesystem

### Requirement: Clankers build ladder starts with low-dependency crates

Mantle MUST build `../../clankers/` through an ordered Clankers build ladder that proves small workspace packages before attempting the root `clankers` binary.
ID: bootstrap.external-rust-workspace.clankers-ladder

The first rung MUST target a low-dependency package such as `clanker-message`, using `bootstrap/rust.ncl`, a fixed Clankers source closure, and offline Cargo. Each later rung MUST add only the source closure entries and native build tools required by that rung. The root `clankers` binary success claim MUST require an installed binary under `$out/bin/clankers` plus a non-network smoke check such as `clankers --help` or `clankers --version`. Full NixOS VM checks, plugin bundle builds, source-built Rust proof, and optional heavyweight runtime integrations are separate follow-up claims unless they are required for the root binary to compile.

#### Scenario: First rung builds clanker-message

- GIVEN a Mantle derivation for the Clankers `clanker-message` package
- AND a fixed source/vendor closure sufficient for that package
- WHEN `mantle build` runs the derivation with `cargo build --locked --offline -p clanker-message`
- THEN the derivation succeeds
- AND the output records the package name, Cargo command, source closure digest, vendor closure digest, and built artifact path

#### Scenario: Native build-script tool is introduced only when needed

- GIVEN a later Clankers rung fails because a package build script requires a native tool such as `cmake`, `go`, `pkg-config`, a C compiler, or onnxruntime headers/libraries
- WHEN the next derivation revision addresses the failure
- THEN it adds the smallest explicit Mantle input required by that build script
- AND earlier rungs remain buildable without that new input unless Cargo's dependency graph requires it

#### Scenario: Root clankers binary claim requires executable smoke

- GIVEN the Clankers root binary derivation succeeds
- WHEN the output is installed
- THEN `$out/bin/clankers` exists and is executable
- AND a sandbox-local smoke check runs without network access and records the observed `--help` or `--version` output

### Requirement: External fixed bundle proofs expose BLAKE3 final proof hashes

Mantle MUST record a BLAKE3 final proof hash for external fixed bundle proofs even when the fetcher or upstream tooling requires SHA-256 compatibility hashes.
ID: bootstrap.external-fixed-bundle.final-proof-blake3

The final proof hash MUST be a Mantle-owned BLAKE3 digest over canonical proof metadata that binds the fixed source bundle identity, compatibility hashes, build recipe, output artifact identity, and smoke evidence. SHA-256 SRI values MAY remain as fetcher or Cargo compatibility hashes, but they MUST NOT be the only recorded proof digest.

#### Scenario: Clankers root proof carries BLAKE3 digest

- GIVEN `packages/clankers/clankers-root-bundle.json` records SHA-256 SRI compatibility hashes
- WHEN the Clankers root bundle proof is inspected
- THEN it records `final_proof_hash_blake3`
- AND the referenced proof file records the same final BLAKE3 proof hash
- AND the proof also records the BLAKE3 digest of the external bundle archive or output artifact.

### Requirement: Clankers root rebuild reproducibility proof

Mantle MUST record whether the pinned Clankers root derivation rebuilds to the same output binary BLAKE3 digest in fresh Mantle stores.
ID: bootstrap.external-fixed-bundle.clankers-rebuild-reproducibility

The proof MUST run `packages/clankers/clankers.ncl` from committed source/bundle metadata, compute BLAKE3 for rebuilt `$out/bin/clankers`, compare at least two fresh `--state-dir`/`--store` rebuilds, and record a machine-readable receipt with commands, store paths, rebuilt digests, previous mismatch evidence when applicable, and verdict. A mismatch MUST be recorded as a failed reproducibility proof rather than silently updating the final proof hash.

#### Scenario: Fresh rebuilds match the stable binary digest

- GIVEN the committed Clankers root bundle and derivation
- AND the proof records stable binary BLAKE3 `2a0fb9daba5445529141aa734de798b5748e65d18e86db5b6f4a776d1700c2ef`
- WHEN Mantle rebuilds `packages/clankers/clankers.ncl` in two fresh stores
- THEN both rebuilt `$out/bin/clankers` BLAKE3 values equal the stable binary BLAKE3
- AND the receipt records verdict `match`

#### Scenario: Fresh rebuild mismatch fails closed

- GIVEN a fresh rebuild produces a different `$out/bin/clankers` BLAKE3
- WHEN the reproducibility receipt is generated
- THEN the receipt records verdict `mismatch`
- AND the final proof hash is not updated to hide the mismatch

### Requirement: Clankers root Cargo paths are reproducibility-stabilized

Mantle MUST normalize or eliminate nondeterministic Cargo build-script output paths from the Clankers root output proof before claiming the Clankers root binary is byte-reproducible.
ID: bootstrap.external-fixed-bundle.clankers-cargo-path-stability

The Clankers derivation MUST pass deterministic Rust path-remapping controls for sandbox-local build roots, run fresh-store rebuilds after the controls are applied, compare rebuilt output binary BLAKE3 digests, and record either a matching stable digest or a fail-closed mismatch receipt. A successful stability proof MUST update final proof metadata to the stable binary digest produced by the remapped derivation.

#### Scenario: Fresh remapped rebuilds match

- GIVEN the Clankers derivation applies deterministic Rust path remapping
- WHEN two fresh-store Mantle rebuilds complete
- THEN both rebuilt `$out/bin/clankers` files have the same BLAKE3 digest
- AND proof metadata records that digest as the stable binary proof input

#### Scenario: Remaining path mismatch is recorded fail-closed

- GIVEN the remapped derivation still produces differing binary BLAKE3 digests
- WHEN the rebuild receipt is generated
- THEN the receipt records verdict `mismatch`
- AND it records the first bounded observed difference
- AND final proof metadata is not updated to claim reproducibility

