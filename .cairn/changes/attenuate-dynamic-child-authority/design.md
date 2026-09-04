# Design: Attenuate dynamic child authority

## Context

Mantle has two dynamic graph-growth lanes. Native `mantle-plan-v1` outputs declare inherited policy, while traditional compatibility outputs carry Nix `.drv` bytes.

The native wire policy says `inherit`, but Worker registration currently assigns a new native compatibility profile. The traditional path does the same after staged parsing and identity admission.

`plan_network_policy` recognizes `builtin:fetchurl` before it evaluates ordinary profile network policy. A generated child can therefore gain fixed-output network authority even when its producer ran offline.

Nix now has an experimental `builder-rpc-v0` system feature. Mantle does not implement that builder-facing service and must not treat the Nix protocol as its native domain model.

## Decisions

### Decision: Use one Mantle-owned attenuation core

Add a pure `dynamic_authority` module under `crunch-build`. The module will use Mantle-owned types and will not import derivation, daemon, Varlink, filesystem, process, network, clock, or async-runtime types.

The core receives normalized values:

- the producer identity and execution-profile identity;
- the producer's effective effect authority;
- the child execution-profile request;
- child effects derived from its action kind;
- inherited substitution, store-prefix, and host-path policy;
- named collection and field bounds.

The effect model covers current authority-bearing fields. These fields include network acquisition, declared build-time network, environment mode, shell provision, setid behavior, syscall exceptions, writable prefixes, host paths, substitutions, and logical store scope.

Resource demand remains separate from effect authority. Existing resource-limit validation continues to reject unsupported or excessive demands.

The core returns either `AdmittedDynamicChildAuthority` or ordered `DynamicAuthorityBlocker` values. An accepted decision includes a domain-separated BLAKE3 identity over the parent ceiling, child request, and policy version.

### Decision: Derive effective effects before attenuation

The Worker adapter will project each parsed derivation or native unit into generic effect facts. Fixed-output fetch behavior is an effect request even when the execution profile says `network_mode = deny`.

The attenuation decision therefore runs before `plan_network_policy` can authorize network use. Child effects must be a subset of the producer ceiling.

An offline producer cannot admit a fixed-output fetch child. A producer with admitted network authority can admit only the matching network class. No child shape grants authority by itself.

### Decision: Bind admission to registry state

Dynamic registry insertion will require the admitted child profile and the attenuation-decision identity. Dynamic paths will not call registration helpers that silently select `ExecutionProfile::native_compatibility()`.

The Worker will recheck the parent profile identity before it applies the registration plan. A missing, changed, or conflicting parent profile fails before goal, waiter, registry, fetch, or scheduler mutation.

Native `mantle-plan-v1` policy value `inherit` will use the actual parent profile. This change does not alter the version-one wire shape.

Traditional `.drv` discovery remains a labeled compatibility path. It receives no exception from attenuation and does not define Mantle's native dynamic API.

### Decision: Classify Nix system features at the adapter edge

The Nix producer adapter will inspect required system features from supported concrete ATerm and derivation-JSON inputs. The adapter will use a closed supported-feature table.

`builder-rpc-v0` is unsupported. Malformed declarations and unknown mandatory features also fail with typed diagnostics before successful graph or package-index publication.

The diagnostic may retain the bounded source feature name. The core receives only Mantle-owned supported or unsupported execution facts.

This change does not add a builder-RPC transport. Cache-only use of derivations that require unsupported builder services remains deferred.

### Decision: Keep builder protocols outside Mantle core

Mantle owns generic object admission, execution-unit admission, declared-result binding, and child-authority attenuation. A future Nix adapter may map protocol operations onto these capabilities.

No Mantle core command, port, result, or receipt type will expose Varlink, `SCM_RIGHTS`, Nix daemon messages, or `builder-rpc-v0` types. Architecture checks will reject those dependencies outside declared adapters and fixtures.

The existing Nix remote-service gateway remains a separate client-facing adapter. It does not grant builder-sandbox authority and does not satisfy this change.

## Effect Flow

```text
producer registry entry
  -> Worker observes profile and effective action policy
  -> adapter projects ParentAuthorityCeiling
  -> dynamic bytes or native plan pass existing structural admission
  -> adapter projects DynamicChildAuthorityRequest
  -> pure attenuation decision
  -> explicit profile-bound registry plan
  -> Worker rechecks parent and decision identities
  -> registry and scheduler mutation
```

A rejection stops before `FetchBuildService`, sandbox dispatch, registry insertion, waiter creation, or goal creation.

## Compatibility and Migration

Accepted native version-one plans keep their wire bytes and canonical plan identity. Their `inherit` fields gain the required runtime meaning.

Traditional generated `.drv` children remain compatible when their effective authority is equal to or narrower than the parent ceiling. Wider children now fail with a stable `dynamic-child-authority-widening` class.

Nix producer inputs without required system features keep their existing projection. Inputs that require unsupported builder services now fail instead of reaching ordinary planning.

## Validation

Positive tests cover equal authority, narrower writable scope, inherited offline policy, stable decision identity, and unchanged native plan bytes.

Negative tests cover fixed-output network escalation, build-time network escalation, wider writable scope, shell provision, setid, syscall exceptions, host paths, substitutions, wrong store scope, stale parent identity, and decision tampering.

Nix fixtures cover supported empty requirements, `builder-rpc-v0`, unknown mandatory features, malformed feature values, and structured-attribute variants.

A Worker integration test will use a counting fetch adapter. The escalation case must leave its call count at zero and preserve registry and scheduler state.

An architecture guard will reject Nix, Varlink, descriptor-passing, transport, and effectful imports from the attenuation core.

## Risks and Trade-offs

- Some compatibility children will now fail. This is an intentional fail-closed change.
- Effective authority projection can drift from execution policy. Shared projection tests and identity rechecks limit this risk.
- The Nix feature table needs maintenance as Nix changes. This maintenance remains confined to the adapter.
- The change does not provide cache-only handling for builder-RPC derivations. That support needs a separate consumer-driven change.

## References

- <https://github.com/NixOS/nix/pull/13768>
- <https://github.com/NixOS/nix/pull/13768#issuecomment-5540182998>
- <https://github.com/NixOS/nix/pull/15793>
- ADR 0011, native dynamic plans
- ADR 0077, Nix derivation compatibility boundary
- ADR 0119, builder protocols at the Nix adapter edge
