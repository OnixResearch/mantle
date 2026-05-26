# Design: Native build oracle target scope

## Problem

`cargo metadata` includes targets that are outside a normal build topology, including examples and benches. The existing native package/target oracle comparison failed closed on any unsupported Cargo target kind attached to a package, even if Cargo's build unit graph would not build that target for the requested normal build invocation.

## Approach

During package/target oracle comparison:

1. Keep matching package identity and supported native target facts.
2. Recognize build-relevant Cargo target kinds as the bounded comparison surface.
3. Ignore Cargo target kinds that are outside normal build execution, rather than producing `unsupported-cargo-oracle-target-kind` blockers.
4. Leave explicit future example/bench rails responsible for any execution claims for those targets.

## Safety

This does not add execution support for examples or benches. It only prevents out-of-scope Cargo metadata targets from blocking normal build-mode package facts. Unsupported dependency, feature, cfg, and source surfaces remain fail-closed through their existing blockers.
