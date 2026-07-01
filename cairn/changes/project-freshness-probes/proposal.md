# Proposal: Project freshness probes

## Summary

Add Mantle-native freshness probes for project inputs. Operators should be able to define how an input's upstream value is discovered, compare that value against the lockfile, and refresh only stale inputs without coupling the pure project core to shell execution or ambient network state.

## Motivation

Nixtamal's `fresh-cmd` captures a useful idea: “fresh” is contextual and is not always the newest tag or branch head. Mantle already has `refresh` and `list-stale`, but it should let projects encode freshness policy for Git refs, HTTP resources, local directories, model files, release feeds, and carefully bounded commands.

A Mantle design should keep the command/network side in the adapter shell and pass deterministic observations into the core. This keeps project refresh logic testable, auditable, and compatible with proof-before-claim expectations.

## Scope

- Define a versioned freshness probe model for project inputs and patches.
- Support built-in probe kinds for Git refs, HTTP text or JSON extraction, local file or directory state, and explicit command probes with bounded execution policy.
- Record freshness observations in refresh/list-stale reports and lock update evidence.
- Allow templated source URLs or references to use the observed freshness value after validation.
- Keep probe execution out of the no-std functional core.
- Preserve offline and no-network modes by reporting network-required probes instead of silently executing them.

## Non-goals

- No arbitrary shell execution in core logic.
- No claim that a freshness observation proves source integrity; hashes and trust checks remain separate.
- No background refresh daemon.
- No use of freshness probes during ordinary build execution unless an operator explicitly asks for refresh/preflight behavior.

## Target Spec Domains

- `project-workflows` for project input freshness semantics, refresh/list-stale behavior, and lockfile updates.
- `operator-diagnostics` may later receive rendering requirements if probe diagnostics need additional CLI shaping.
