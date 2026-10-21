# ADR 0118: Keep REAPI transport separate from Mantle output authority

- **Status:** Proposed
- **Date:** 2026-10-18

## Context

[Issue #22](https://github.com/OnixResearch/mantle/issues/22) asks for an opt-in
Bazel REAPI execution/cache adapter without replacing native stdio/SSH/P2P or
weakening Mantle trust. REAPI CAS identifies bytes; its action cache identifies
one mutable ActionResult by an encoded Action digest. Neither provides Mantle's
signed PathInfo, native action receipts, producer policy or current-attempt
fencing. TLS, Operation names and worker metadata are not output provenance.

The [compatibility investigation](../docs/reapi-compatibility.md) maps the exact
v2.3.0 protocol at remote-apis commit
`e95641649b5b4d3c582c89daabfaabeb8189dd77`, and ByteStream/Operations at
googleapis commit `9415ba048aa587b1b2df2b96fc00aa009c831597`. This version's
digest enum does not include BLAKE3. REAPI does not standardize Mantle's
logical store prefix, resource leases, network isolation or signing producer.
Even one Execute can perform duplicate executions; cancellation is best effort
and general Operations support is optional.

Existing configuration and available handoffs supply no explicitly authorized
compatible REAPI origin/instance, credential handle, allowed upload closure or
Mantle-aware signed producer. The native loopback rail is not such a service.
This ADR proposes constraints for future implementation; it does not claim an
adapter, service provisioning or milestone-2 acceptance.

## Decision drivers

- Preserve native semantic identities, lazy scheduling and trust/admission.
- Reuse infrastructure only where its observed contract is compatible.
- Separate byte presence, advisory action-result discovery, authenticated
  producer evidence and execution authority, as required by ADR 0024.
- Keep resource and attempt ownership fenced, as required by ADR 0025.
- Require explicit endpoint, data-upload and credential authorization.
- Reject incompatible semantics rather than invent evidence or weaken claims.

## Decision

Keep REAPI in a separate opt-in host adapter. Do not replace Mantle's identity,
CAS encodings, action-result index or native transport protocols with REAPI.
Maintain explicit associations between native action ref, normalized build key,
per-dispatch plan, exact REAPI Action digest and current local job/attempt/fence;
these are distinct identities, not aliases. Start with SHA-256 wire digests and
uncompressed ByteStream, independently verifying native BLAKE3 and NAR facts.
Lock complete protobuf/tool dependencies during implementation; the reviewed
source pins alone do not provision a generated client.

The bounded initial execution fragment is one concrete Linux x86-64 action,
one known input-addressed output, immutable declared inputs and network-none.
Use a pinned worker-side Mantle-aware producer to unpack explicitly approved
input NARs and actually execute the action under the required native policy.
Transport NARs as regular-file CAS blobs rather than pretending REAPI Directory
or Tree encodings equal castore or NAR. Require a separately authorized,
immutable worker/runtime/bridge profile and enforced resource limits. A generic
REAPI service without compatible producer and isolation semantics is rejected.
Do not rewrite arbitrary absolute store paths or advertise general Bazel/Nix
compatibility.

The producer returns bounded regular-file NAR, PathInfo, signed native result
record and complete referenced evidence closure. Its signing capability is
separately authorized and isolated from the untrusted build sandbox; private
keys and service credentials never enter CAS. A client must not manufacture
provenance by signing downloaded ActionResult metadata. Finalize and version
the bridge/envelope format with its real implementation before enabling it.

Treat a server cache hit as one advisory locator. Independently hash all bytes;
verify complete native content, exact requested output/prefix, full public-key
signatures, action receipt/input linkage, reference scans and current
producer/sandbox/network/publication policy before ordinary native admission.
Signature presence or signer-name equality is not cryptographic verification.
Reuse existing immutable native signed records and conflict policy rather than
turning the mutable REAPI action cache into native authority. No automatic
client UpdateActionResult publication is enabled in the initial fragment.

Require an explicit service authorization bundle before any service RPC:
permitted TLS origins and instance, operation permissions, data-retention and
tenant policy, exact allowed upload closure/budgets, explicit external
credential handle/provider, and separately trusted producer keys/policy.
Source upload is default-deny. A native ticket, reachable DNS name, binary-cache
URL or service capability advertisement supplies none of those permissions.
Do not provision credentials, discover alternate endpoints or fall back to an
ambient provider. Keep bearer material out of argv, environment, durable
operation state and diagnostics, consistent with ADR 0061.

Freeze canonical Action bytes across reconnects. Once a named operation exists,
reconcile it with WaitExecution instead of blindly resubmitting Execute.
Persist only credential-free operation associations. A disconnect before the
first operation name has unknown execution outcome, not proof that nothing
ran. Explicit reassignment invalidates the old local attempt; current-fence
checks precede authoritative admission/completion. A reusable signed native
cache record may satisfy a new authorized admission attempt, but an old
attempt's completion message cannot complete it. Cancellation invalidates local
acceptance authority before the best-effort remote request; never claim remote
termination from a transport close or cancellation acknowledgement.

Reject unsupported hashes, malformed/incomplete trees, unexpected output paths,
unverified evidence, unknown enforced profiles, unsupported resource contracts,
dynamic/CA outputs, accelerators and persistent workspaces in this initial
fragment. Do not drop a constraint to make an endpoint appear compatible.
Native transports remain available without implying any automatic fallback
from trust/admission failures.

## Consequences

A compatible real service requires more than ordinary CAS/ActionCache/Execution:
it also needs an explicitly authorized producer/evidence and enforcement path.
This reduces immediate endpoint compatibility but avoids turning infrastructure
access into output trust. Canonical translation, byte verification and operation
bookkeeping add adapter cost without changing the pure semantic core.

Milestone 1 can be reviewed independently. Milestone 2 remains open until the
operator supplies the missing authorization/service/producer/fixture bundle,
issue #15 supplies the immutable source/executable baseline, and a real bounded
adapter demonstrates every issue #22 acceptance scenario. No public endpoint,
fake fixture, documentary claim or historical native binary can substitute.
This ADR does not authorize deployment, source upload, secret provisioning or
issue closure. The detailed investigation records the required bundle,
source-derived native regression commands and next implementation boundary.
