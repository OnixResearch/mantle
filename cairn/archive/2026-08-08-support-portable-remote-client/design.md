# Design: Portable remote-first Mantle client

## Context

`crunch-pipeline` rejects local builds on non-Linux targets. The route planner already models local executor capability and remote builder eligibility. Remote clients can submit concrete derivation facts and admit returned outputs through signed PathInfo and castore checks.

The root Mantle binary also contains Linux-only worker, seccomp, sandbox, publication, bootstrap, and proof code. Platform `cfg` fallbacks prevent some calls, but there is no complete support matrix or dependency boundary that proves the portable client cannot enter those paths.

## Decisions

### Decision: Extend the typed operator inventory with platform profiles

**Choice:** Extend the operator inventory from `stabilize-operator-command-contract` with profiles keyed by client platform and command capability. Each row records support state, local effects, network effects, required remote capability, required trust, and the stable unsupported reason. Canonical command identity and generated command documentation continue to come from the shared operator catalog.

The initial Darwin profile supports doctor, project check and refresh, Nickel evaluation, build planning, native remote build submission and status operations, bounded logs and cancellation, output admission and optional local materialization, and portable evidence inspection. Worker, server, local build, bootstrap, self-build, and Linux proof commands remain unsupported.

**Rationale:** Compilation success alone does not define a usable product surface.

### Decision: Separate client core from Linux execution

**Choice:** Extract remote request planning, route decisions, upload classification, response admission inputs, and client report construction into a portable pure core. A thin portable shell owns files, network transport, credential handles, local castore, and output writes.

Linux worker, bwrap, FUSE, seccomp, cgroup, protected-exec, bootstrap, and proof dependencies remain outside the portable client dependency closure. Build-time guards reject accidental imports across the boundary.

**Rationale:** Runtime unsupported branches still compile and increase coupling. A dependency boundary makes the client support claim reviewable.

### Decision: Treat local execution as one route capability

**Choice:** Build planning always evaluates local cache, trusted import, remote, and local execution eligibility from explicit facts. On a non-Linux client, local execution is ineligible with a stable reason. It is not a process-wide fatal error when another route is eligible.

A non-Linux build selects remote execution only after concrete inputs, source readiness, upload policy, worker capability, credentials, and output trust pass. If no route passes, the command returns the ordered route blockers.

**Rationale:** The client host does not need to implement the target executor.

### Decision: Keep evaluation on the client

**Choice:** The client evaluates Nickel, project selectors, locks, and frontend input locally under existing import and source rules. It sends concrete frontend-neutral build requests and immutable object refs.

The remote side does not receive raw Nickel, Onix module data, flakes, Nix expressions, or package-manager resolution authority. The request binds target system independently from client system.

**Rationale:** Remote evaluation would enlarge authority and couple workers to frontends.

### Decision: Use explicit portable storage locations

**Choice:** Client state defaults to the platform XDG or native application-state location selected by policy. Logical store paths remain `/mantle/store` or an explicit compatibility prefix. Physical local output materialization uses an explicit unprivileged directory and never requires `/nix`.

A client can choose report-only completion or admitted local materialization. Materialization imports verified objects and PathInfo through the ordinary store admission path before writing the physical output.

**Rationale:** Logical identity and physical placement are already separate. macOS must not inherit Nix volume assumptions.

### Decision: Preserve credential and trust separation

**Choice:** The portable shell reads credentials only from explicit caller-owned files or supported secure platform handles. It never places bearer material in plans, reports, logs, process arguments, or store objects.

Execution authorization, upload authority, log access, cancellation, output signer trust, and publication remain separate. A valid remote ticket cannot admit an output without configured signer and content checks.

**Rationale:** Portability must not weaken the remote security model.

### Decision: Fail before Linux-only effects

**Choice:** Unsupported commands fail during command-matrix admission before store mutation, network connection, worker launch, or Linux tool discovery. Remote route fixtures install panic or sentinel implementations for local executor, bwrap, FUSE, seccomp, and worker-server seams.

**Rationale:** An error after looking for bwrap is not remote-first behavior.

### Decision: Prove support on native Darwin and Linux

**Choice:** Validation includes native `aarch64-darwin` and `x86_64-darwin` compile and command fixtures plus Linux parity. Cross-target checks are supplemental because they cannot prove native filesystem, process, TLS, keychain, or signal behavior.

The release support matrix identifies checked, blocked, and untested command classes per platform.

**Rationale:** Platform claims need execution evidence on the named platform.

## Validation

Positive fixtures cover Darwin doctor, project check, evaluation, route plan, remote build, bounded logs, cancellation, report-only completion, local materialization, explicit physical store, and Linux local-build parity.

Negative fixtures cover no eligible route, raw frontend payload, target/client confusion, local executor sentinel, bwrap or FUSE discovery sentinel, unsafe physical store, missing output trust, wrong signer, corrupt CAS object, stale fence, oversized upload, credential leakage, unsupported command, and native platform drift.

## Risks / Trade-offs

- Extracting a portable dependency closure can expose hidden Linux assumptions across many root modules.
- Native Darwin validation requires maintained runners and cannot be replaced by Linux cross-compilation.
- Client-side Nickel evaluation can still need evaluator resource budgets from the separate change.
- Remote-first use depends on reachable trusted workers and explicit source upload policy.
- Some evidence publication operations remain Linux-only because their atomic no-replace guarantees differ by platform.
