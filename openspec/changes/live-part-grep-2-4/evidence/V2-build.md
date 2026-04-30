Task-ID: V2
Covers: bootstrap.part.grep.2.4

Status: deferred

## Attempted command

`nix shell nixpkgs#bubblewrap -c ./target/debug/crunch build --no-substitute --store .crunch-drain/grep-store --state-dir .crunch-drain/grep-state bootstrap/grep-2.4-musl.ncl`

## Result

The build was re-run after creating writable local state/store directories. It entered Crunch startup and repaired prior local redb state, then exceeded the local 300 second command budget before producing either an output path or a failure class.

This is a runtime-budget blocker, not proof that the hardened grep derivation is incorrect. Runtime proof is deferred to OpenSpec change `live-part-grep-2-4-runtime-validation`.

Full attempted transcript: `evidence/V2-build-full.log`

Verified: 2026-04-30T22:12:07Z
