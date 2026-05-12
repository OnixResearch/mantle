## MODIFIED Requirements

### Requirement: Bootstrap parity map rejects unevidenced binutils bridges [r[bootstrap.parity.binutils-tcc-evidence]]

The parity report MUST fail closed for `binutils.tcc` when the derivation contains placeholder markers, bridge-only notes, missing smoke transcripts, or unchecked evidence references. When `bootstrap/evidence/binutils-tcc-tool-smoke.json` is present, the report MUST accept it only if it is produced from `bootstrap/binutils-tcc.ncl`, names the output path, records provider kind, records no host fallback, and contains successful smoke entries for `as`, `ld`, `ar`, `ranlib`, `nm`, and `objcopy`.

#### Scenario: Require checked evidence for promotion [r[bootstrap.parity.binutils-tcc-evidence.require-checked]]

- GIVEN `bootstrap/binutils-tcc.ncl` exists but no checked binutils tool transcript is present
- WHEN `crunch bootstrap parity-report --require live-bootstrap` or `--require guix` runs
- THEN the command fails and identifies `binutils.tcc` as a blocker

#### Scenario: Promote only after tool smokes [r[bootstrap.parity.binutils-tcc-evidence.tool-smokes]]

- GIVEN a checked transcript proves the binutils-tcc output tools
- WHEN the parity report loads that evidence
- THEN the `binutils.tcc` row may advance only to the status justified by the transcript and must not imply downstream GCC correctness

#### Scenario: Transcript is generated from complete logical closure [r[bootstrap.parity.binutils-tcc-evidence.logical-closure]]

- GIVEN the binutils-tcc output depends on logical `/crunch/store` paths
- WHEN the evidence producer smokes the tools
- THEN `/crunch/store` is bound to the complete scratch store closure used by the build
- AND the transcript records the scratch store root and smoke command exit statuses
