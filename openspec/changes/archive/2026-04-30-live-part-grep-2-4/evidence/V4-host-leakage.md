Task-ID: V4
Covers: bootstrap.part.grep.2.4

Status: deferred

A source-level hardening pass removed suppressed per-source compile failures from `bootstrap/grep-2.4-musl.ncl` and added fail-closed object/output checks. Full host-tool/path/environment leakage validation still depends on the completed V2 runtime transcript, which was not available within the local drain command budget.

Deferred to OpenSpec change `live-part-grep-2-4-runtime-validation`.

Verified: 2026-04-30T22:12:07Z
