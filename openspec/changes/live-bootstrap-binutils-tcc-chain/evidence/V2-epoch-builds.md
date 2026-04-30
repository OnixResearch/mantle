Task-ID: V2
Covers: bootstrap.binutils.tcc.chain

Status: deferred

## Attempted command

`nix shell nixpkgs#bubblewrap -c ./target/debug/crunch build --no-substitute --state-dir /home/brittonr/git/crunch/crunch/.crunch-drain/binutils-tcc-state --store /home/brittonr/git/crunch/crunch/.crunch-drain/store bootstrap/bzip2-tcc.ncl`

## Result

The prerequisite environment issue from the original task note was resolved: `crunch doctor --profile build --store .crunch-drain/store --state-dir .crunch-drain/binutils-tcc-state` passed when `bubblewrap` was supplied through `nix shell nixpkgs#bubblewrap`.

The first epoch build then exceeded the local drain budget. At ~30 minutes it was still compiling the Mes prerequisite for `bzip2-tcc` inside bubblewrap:

- outer root: `bootstrap/bzip2-tcc.ncl`
- active dependency: `mes`
- active child observed: `/crunch/store/...-mes/bin/mes-m2 --no-auto-compile ... -c .../lib/stub/realpath.c`
- transcript: `openspec/changes/live-bootstrap-binutils-tcc-chain/evidence/logs/V2-1-bootstrap_bzip2-tcc.ncl.log`

This is not a correctness failure; it is a runtime/validation-budget problem. The process was terminated and the validation work is deferred to scoped follow-up change `live-bootstrap-binutils-tcc-chain-runtime-validation`.

## Required follow-up

The follow-up must resume with a longer-running validation strategy, preserve the `nix shell nixpkgs#bubblewrap` prerequisite, and either complete V2-V5 or split the Mes prerequisite build into separately cached/proven roots before reattempting the full epoch chain.

Verified: 2026-04-30T21:55:51Z
