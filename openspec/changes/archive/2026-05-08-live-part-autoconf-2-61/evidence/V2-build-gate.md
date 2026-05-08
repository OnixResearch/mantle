# V2 autoconf 2.61 build gate evidence

Task-ID: V2
Covers: bootstrap.part.autoconf.2.61

## Result

`bootstrap/autoconf-2.61.ncl` was inspected after source hardening instead of launching an expensive build against known-unarchived predecessor parts. The derivation directly imports and declares `automake-1.8.5.ncl` as `prev_am`; `openspec list` still reports `live-part-automake-1-8-5` active with 0/8 tasks, so the predecessor part has not produced scoped build/smoke/leakage evidence yet.

The build is therefore closed for this drain as prerequisite-gated, not as a runtime promotion. Future positive evidence must build with declared predecessor outputs only.

Required transcript field status:

| Field | Status |
|---|---|
| command | deferred: target build gated on unarchived predecessor `automake-1.8.5` and its transitive toolchain |
| prerequisite | blocked: `live-part-automake-1-8-5` remains active/unvalidated |
| exit status | not run; prerequisite evidence is incomplete before target build |
| output path | none |
| failure class | prerequisite part not independently validated yet |
| fallback status | no host Autoconf/Automake/Perl/Make or Nix substitute accepted |

No `autoconf-2.61` build success is claimed here.
