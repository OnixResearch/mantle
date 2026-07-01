# Proposal: Project input fetch policy

## Summary

Add explicit fetch policy to project inputs so Mantle can distinguish values needed before evaluation or input generation, ordinary build-time fetch actions, and offline/imported source-state requirements.

## Motivation

Nixtamal exposes `fetch-time = eval | build` to make source acquisition timing visible. Mantle should adopt the same clarity without importing Nix builtins semantics. Project manifests need to say whether an input is resolved during project refresh/input generation, realized as a build action through Mantle's fetch services, or required to already exist in imported source state for offline builds.

Explicit policy prevents accidental network access during builds, makes source bundle planning easier, and lets diagnostics explain why an input was fetched early, deferred, or rejected.

## Scope

- Define project input fetch policies for generation-time material, build-time fetch actions, and offline/imported-only source material.
- Thread policy through refresh, generated `.mantle/inputs.ncl`, build planning, source bundle planning, and offline preflight.
- Reject policy combinations that cannot be honored, such as patched inputs in a mode that cannot apply patches.
- Report network-required, source-state-required, and build-fetch-required classes deterministically.
- Keep fetch policy as build-tool data, not a frontend module-layer semantic.

## Non-goals

- No Nix `builtins.fetch*` compatibility requirement.
- No implicit policy inference from URL hostnames or forge-specific shortcuts.
- No network fetch during offline preflight or source-bundle import.

## Target Spec Domains

- `project-workflows` for manifest policy and project command behavior.
- `build-tool-boundary` remains relevant for ensuring policies stay frontend-neutral build data.
