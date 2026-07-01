# Proposal: P2P remote builders

## Summary

Add a Mantle-native P2P remote builder capability inspired by `adeci/drv-thru` and `Mic92/tribuchet`: operators can run a builder service, optionally route requests through a coordinator/hub, issue bounded tickets or configure trusted clients, and clients can offload concrete Mantle build work without creating system accounts on the builder. Remote outputs must return through Mantle's signed PathInfo, artifact attestation, and substitution machinery rather than through raw file import.

## Motivation

Mantle already has local scheduling, signed store metadata, binary-cache sharing, and delta/full substitution paths. What is missing is a first-class way for one Mantle host to temporarily spend another host's build resources while preserving Mantle's trust model. The useful `drv-thru` pattern is the access-control and transport shape: tickets grant builder resource access, while signed outputs remain a separate trust decision. Tribuchet adds the stronger service architecture lesson: a coordinator can schedule capability-matched workers that dial in from NATed hosts, deduplicate identical submissions, negotiate only missing inputs, stream bounded logs, and survive reloads without giving the coordinator output-trust authority. Mantle should adapt those patterns to its own CAS, store prefix, build reports, and attestation contracts instead of inheriting Nix-specific `nix-store`, external-builders, nix-daemon, identical scratch-path, or `/nix/store` behavior.

## Scope

- Define a versioned Mantle remote-build protocol over pluggable transport bindings: loopback and stdio/SSH-stdio for deterministic validation, with a NAT-friendly P2P transport such as Iroh as the production network binding.
- Define ticketed and trusted-client access with expiry, use count, build time limits, upload limits, optional endpoint binding, revocation, and status-safe inspection.
- Define an optional coordinator/hub role where workers initiate registration, advertise systems/features/network/sandbox capabilities, concurrency, output signing keys, and resumable job summaries; clients submit normalized concrete requests; duplicate requests attach to in-flight work; and conflicting requests are rejected.
- Preserve a hard separation between permission to use builder resources and permission to import builder outputs.
- Require local evaluation or frontend lowering before remote dispatch; the builder executes concrete Mantle derivations/action specs, not arbitrary Nickel or frontend module logic.
- Negotiate and upload only missing Mantle inputs using PathInfo, source-input manifests, and CAS blob/directory/object refs with bounded chunks.
- Return outputs through signed PathInfo, artifact attestations, and delta-first/full-fallback substitution where the client verifies trust before accepting results.
- Expose server queue/status information that is useful for operators without leaking ticket bearer secrets.
- Require protocol-mode stdout/stderr separation, cheap initial handshakes, phase-based failure classification, stable session identity, and session-scoped leases or roots for live remote-build state.

## Non-goals

- No dependency on `nix-store`, `nix copy`, Nix trusted users, NixOS modules, or `/nix/store` as a remote-build protocol requirement.
- No commitment to SSH as the only or final production transport; SSH-stdio is a validation and operator bootstrap binding for the same protocol, not a separate remote-shell feature.
- No remote evaluation of arbitrary Nickel source or frontend module semantics on the builder.
- No weakening of Mantle's signed PathInfo, artifact attestation, store prefix, or substitution trust policies.
- No general remote shell, remote command execution, account provisioning, or persistent deploy agent.
- No output import based only on possession of a ticket.

## Reference

`adeci/drv-thru` is prior art for the user workflow and threat-model split: P2P builder sessions, one-time tickets, trusted clients, queue/status reporting, missing-input upload, and signed binary-cache output import. Mantle should adapt those ideas while replacing Nix-specific subprocesses and cache assumptions with Mantle-native build graph, CAS, PathInfo, delta substitution, and attestation primitives.

Kolu's oRPC-over-SSH work is prior art for the transport seam and production hardening lessons: keep one remote boundary, treat stdout as the protocol in stdio mode, make the first handshake cheap, classify network/config/build/trust failures by phase, and pin live remote artifacts only for the active session.

`Mic92/tribuchet` is prior art for hub/worker scheduling, worker-dialed registration, capability-matched queues, in-flight dedupe, missing-input negotiation, worker-signed outputs, bounded log replay, and restart/reload survival. Mantle should adapt those architecture properties while replacing Tribuchet's Nix external-builder shim, nix-daemon imports, hardcoded `/nix/store` assumptions, TOML-first configuration, and identical-scratch-path output trick with Mantle-native CAS, PathInfo, attestation, store-prefix, and Nickel-owned policy surfaces.

## Target Spec Domains

- `remote-builds` for the Mantle-native remote builder protocol, access model, input sync, output import, and status requirements.
