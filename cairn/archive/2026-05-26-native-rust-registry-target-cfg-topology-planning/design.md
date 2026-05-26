# Design: Native Rust registry target-cfg topology planning

## Current state

Mantle has explicit native receipt rails for vendored registry source facts and bounded topology execution, including transitive registry chains and feature-selected optional dependencies. Native package parsing currently focuses on normal dependency tables plus bounded feature activation. Target-specific dependency tables are not yet represented as Mantle-owned facts.

## Goals

1. Add native target-cfg dependency facts for a bounded supported manifest shape.
2. Decide supported cfg predicates against Mantle's active target triple without invoking Cargo as planner.
3. Feed selected target-cfg registry dependencies into native package/unit/topology evidence only when their registry source facts are ready.
4. Block before `rustc` when cfg selection would require unsupported Cargo/platform behavior.
5. Prove the seam with CLI JSON fixtures.

## Native fact shape

Implementation should extend the existing native package planning evidence with enough deterministic material to review target-cfg selection. A minimal acceptable shape is:

- package id and manifest path;
- cfg expression string;
- decision (`selected`, `not-selected`, or `blocked`);
- active target triple used for evaluation;
- dependency key/package/version/source fact selected when applicable;
- blocker class/message for unsupported cfg surfaces.

If the implementation can express the same material through existing `selected_features`, `path_dependencies`, blockers, and source facts without a new top-level receipt section, that is acceptable only if CLI JSON assertions still prove the target-cfg decision and selected/unselected edge behavior.

## Supported cfg subset

Start narrowly:

- `cfg(unix)` for Unix targets;
- `cfg(target_os = "linux")` for Linux targets;
- optionally direct `cfg(target_arch = "x86_64")` if the active target triple is available deterministically.

Everything else should block with an explicit class such as `unsupported-target-cfg-surface` before affected `rustc` execution.

## Graph behavior

- Selected target-cfg dependencies become native dependency edges and participate in producer-first topology ordering.
- Unselected target-cfg dependencies must not execute and must not appear as required `rustc --extern` material.
- Selected vendored registry packages must still pass native registry source planning gates.

## Verification

Positive CLI fixture:

- local app depends on vendored registry package A;
- package A declares a target-cfg dependency on vendored registry package B using a supported Linux/Unix cfg;
- topology execution proves B -> A -> app order and dependency artifact digest binding.

Negative CLI fixture:

- same shape but cfg expression uses an unsupported surface such as `cfg(any(target_os = "linux", target_os = "macos"))` or target-specific feature behavior;
- receipt reports deterministic target-cfg blocker and zero affected execution before `rustc`.
