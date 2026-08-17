# Native target-cfg optional dependency scope

## Summary

Scope selected target-cfg dependency facts so optional target dependencies that are not selected by explicit native feature facts do not require vendored registry sources before normal build topology execution.

## Motivation

Mantle self planning can now evaluate common target-cfg predicates, but normal package planning still reports `unsupported-target-cfg-dependency` for optional target dependencies such as assembly, cookie, font, and platform backends that are not present in Cargo's locked build graph. Treating every selected cfg table dependency as required over-claims optional feature surfaces and blocks before the remaining real build/host topology gaps.

## Scope

- Reuse native selected feature facts when deciding target-cfg dependencies.
- Keep selected non-optional target dependencies bound to existing path or declared registry source facts.
- Record unselected optional target dependencies explicitly without a manifest path.
- Preserve fail-closed blockers for selected non-optional target dependencies that lack a bounded source.

## Non-goals

- Implementing Cargo's full feature resolver.
- Selecting optional target dependencies from transitive feature expressions not already represented as native selected features.
- Widening normal topology execution semantics.
