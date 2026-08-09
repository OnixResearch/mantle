# Support a portable remote-first Mantle client

## Why

Mantle can plan remote-builder routes and owns concrete remote build, transfer, PathInfo, attestation, and output-admission protocols. Local realization still fails on non-Linux hosts because the build pipeline requires Linux and bubblewrap.

The current unsupported-platform diagnostic points operators toward Linux execution tools even when a compatible remote route can perform the build. Mantle also lacks one checked command matrix that separates portable client operations from Linux worker, sandbox, server, and proof operations.

Mantle needs a client profile for macOS and other supported non-Linux hosts that evaluates and plans locally, sends only concrete frontend-neutral build inputs, and realizes through trusted Linux workers without a fixed `/nix` volume.

## What Changes

- Extend the accepted operator command inventory with portable-client platform profiles for `x86_64-darwin`, `aarch64-darwin`, and supported Linux clients.
- Separate portable remote-client core and shell dependencies from Linux worker and local sandbox dependencies.
- Make local executor capability an explicit route fact instead of a process-wide build prerequisite.
- Let non-Linux `mantle build` planning select an eligible remote route and reject only when no admitted route exists.
- Keep Nickel and project evaluation on the client and send concrete Mantle build inputs, source and object refs, policy identities, and bounded upload plans.
- Materialize admitted remote outputs into a configurable unprivileged local physical store when requested.
- Preserve explicit credential files, output trust, CAS verification, PathInfo, attestation, and privacy checks.
- Add native Darwin, cross-target, protocol, no-local-exec, trust, transfer, and unsupported-command fixtures.

## Dependencies

- `stabilize-operator-command-contract` owns canonical command identity, support tiers, effects, and generated command documentation. This change extends that accepted inventory with platform profiles rather than creating a second command catalog.
- Existing realization-routing, remote-build, resumable transfer, and output-admission requirements remain authoritative.
- Public remote endpoints depend on archived `harden-remote-credential-boundary` evidence.
- The Nix compatibility gateway is optional. Portable Mantle clients use the stronger native protocol or Build API without changing the internal model.

## Non-Goals

- Native macOS build execution, an APFS `/nix` volume, or a macOS sandbox backend.
- Remote evaluation of raw Nickel, Onix modules, Nix expressions, flakes, or package-manager manifests.
- Running worker, server, bootstrap, self-build, or Linux proof commands on Darwin.
- Treating remote authorization as output trust.

## Impact

- **Affected specs:** `realization-routing`
- **Planned files:** typed platform matrix, portable client core, CLI feature boundaries, route planner, remote client shell, local materialization adapter, diagnostics, CI fixtures, and docs
- **Compatibility:** Linux local execution remains available; non-Linux builds require an eligible remote route and explicit output trust
- **Testing:** Darwin command matrix, remote route selection, no local executor launch, upload privacy, credentials, output materialization, unsupported commands, and Cairn gates
