## MODIFIED Requirements

### Requirement: Binutils-TCC chain implementation [r[bootstrap.binutils.tcc.chain]]

Crunch MUST build `bootstrap/binutils-tcc.ncl` from chain-internal TinyCC-era and post-musl derivations without using host compiler, host libc, host shell tools, or the legacy musl.cc provider. The binutils-tcc stage MUST NOT satisfy live-bootstrap or Guix parity until reproducible evidence proves the produced assembler/linker/archive tools and records absence of host fallback.

#### Scenario: Placeholder is replaced with evidence [r[bootstrap.binutils.tcc.chain.evidence-promotion]]

- GIVEN `crunch bootstrap parity-report` evaluates the `binutils.tcc` row
- WHEN the row is considered for live-bootstrap or Guix parity
- THEN it remains `placeholder` or `partial` unless a checked transcript proves `as`, `ld`, `ar`, `ranlib`, `nm`, and `objcopy` from `bootstrap/binutils-tcc.ncl`
- AND the transcript records the build command, output path, provider kind, fallback markers, and smoke command exit statuses

### Requirement: Bootstrap parity map rejects unevidenced binutils bridges [r[bootstrap.parity.binutils-tcc-evidence]]

The parity report MUST fail closed for `binutils.tcc` when the derivation contains placeholder markers, bridge-only notes, missing smoke transcripts, or unchecked evidence references.

#### Scenario: Require checked evidence for promotion [r[bootstrap.parity.binutils-tcc-evidence.require-checked]]

- GIVEN `bootstrap/binutils-tcc.ncl` exists but no checked binutils tool transcript is present
- WHEN `crunch bootstrap parity-report --require live-bootstrap` or `--require guix` runs
- THEN the command fails and identifies `binutils.tcc` as a blocker

#### Scenario: Promote only after tool smokes [r[bootstrap.parity.binutils-tcc-evidence.tool-smokes]]

- GIVEN a checked transcript proves the binutils-tcc output tools
- WHEN the parity report loads that evidence
- THEN the `binutils.tcc` row may advance only to the status justified by the transcript and must not imply downstream GCC correctness
