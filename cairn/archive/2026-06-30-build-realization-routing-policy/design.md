# Design: Build realization routing policy

## Architecture

Realization routing is a pure planning layer above concrete transports. It consumes facts gathered by the shell and returns a deterministic route plan. The shell owns filesystem/store queries, network configuration reads, builder discovery, trust config loading, clocks, and command output. The core owns route classification, priority/eligibility decisions, blocker construction, non-claim wording, and redaction decisions over in-memory facts.

## Inputs

The route planner should receive bounded facts:

- requested roots, output names, target platform/profile, hermeticity mode, network policy, and requested evidence strength;
- local PathInfo/castore/artifact-attestation availability;
- source-bundle/offline source-state readiness and missing-source diagnostics;
- store archive candidates and their trusted compatibility metadata;
- substituter metadata availability, trust roots, and delta/full transfer capability summaries;
- remote builder profiles, endpoint identities, ticket/trusted-client authorization class, capabilities, queue/resource limits, and output signing keys;
- local executor capability facts and preflight blockers.

No raw secret bytes, bearer ticket strings, private-key paths, full environment dumps, or unbounded log payloads belong in planner inputs or outputs.

## Route classes

Initial route classes should include:

- `cached-local`: every requested output is already accepted locally;
- `trusted-substitute`: trusted remote cache metadata can satisfy missing outputs;
- `archive-import-candidate`: a store archive can satisfy missing outputs after import/verify;
- `source-bundle-required`: missing source/input material must be imported before build or remote dispatch;
- `p2p-remote-build-candidate`: a concrete build can be offloaded to a compatible builder and the client has an output-trust path;
- `local-build-candidate`: local execution preflight passed;
- `preflight-error`: no route satisfies policy, trust, input readiness, and capability constraints.

The planner may rank routes, but the ranking must be explicit and stable. Rejected routes should include reason codes such as `missing-source`, `untrusted-output-key`, `offline-network-required`, `unsupported-platform`, `archive-prefix-mismatch`, `builder-capability-mismatch`, and `local-executor-unavailable`. If route facts are equal, named tie-breakers must decide the route; latency, discovery order, map iteration order, or random availability races cannot silently choose a different route.

## Offline mode

Offline mode is stricter than low-network preference. If selected, any route that would require live network, remote cache lookup, remote builder connection, VCS fetch, language package-manager access, source download, or undeclared build/cache directory access is ineligible unless all required bytes are already available through local state, imported source bundles, or local store archives. Diagnostics must name the first missing class and preserve the full bounded rejected-route list in JSON.

## Claim strength

Route planning must consider requested evidence strength. Practical builds may select routes that produce ordinary signed PathInfo and artifact attestation evidence. Strong action-correctness or release-facing routes require matching action refs, source/input refs, sandbox and network policy evidence, reference-scan evidence, and trusted receipts before reuse or remote/offline acceptance. A route that can build practically but cannot produce the requested strong evidence must be downgraded or rejected with a deterministic claim-strength blocker.

## Remote builder eligibility

A P2P remote-build route is eligible only when:

- the client already has concrete evaluated build inputs or frontend-neutral action specs;
- builder capabilities match platform/profile/hermeticity requirements;
- input sync is possible from local store/source state within upload/resource limits;
- the client has a configured trust path for returned output PathInfo/attestations;
- the route does not require remote evaluation or frontend module semantics;
- the planned upload set is privacy-safe under configured policy, with source/blob/path classes, byte counts, and secret descriptors summarized before dispatch.

The planner must never include bearer tickets, private key paths, decrypted secret bytes, or raw environment values in upload summaries. Operators should be able to distinguish "will upload source inputs", "will upload store closures", "will upload proof inputs", and "will upload secret descriptors only" before a remote route is selected.

Ticket validity may allow resource access, but it must not satisfy output trust.

## Reporting

Human and JSON route reports should identify selected route, rejected alternatives, input/source gaps, trust blockers, network requirements, and non-claims. A route plan can say what Mantle intends to try; it must not say the build succeeded, the output is trusted, or the archive/builder/cache was accepted until the downstream command produces evidence.

## Validation strategy

- Pure positive tests for stable route selection across local cache, trusted substitute, archive import, remote build, source-bundle-required, and local build cases.
- Pure negative tests for offline network requirements, missing source state, untrusted output keys, archive prefix mismatch, builder capability mismatch, and local executor preflight errors.
- CLI/build-plan tests proving JSON reports remain deterministic and secret-safe.
- Integration tests can use fake fact providers before real transports exist, but claims must stay bounded to planner behavior.
