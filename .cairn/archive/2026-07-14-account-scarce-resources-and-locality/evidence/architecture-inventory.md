# Resource and locality architecture inventory

## Portfolio-search success contract

- **Goal:** extend Mantle's existing lazy scheduler and remote coordinator with bounded quantified resources, receiver-verified locality, and durable fenced resource leases without creating a second scheduler or weakening existing hard policy.
- **Completion evidence:** focused pre/post tests; pure-core positive and negative cases; production coordinator concurrent/restart fixtures; Rust/Nickel parity; bounded report assertions; format/lint; Cairn validate and proposal/design/tasks gate receipts using the checked-in generated policy.
- **False completion excluded:** ordinal classes without derivation; in-memory-only reservations; capacity inferred from concurrency; provider-response ordering; locality copied from worker advertisements; zero-transfer claims without verified missing-set facts; fake known-graph pressure; task checkboxes without current evidence.
- **Audit risks:** arithmetic overflow/underflow, duplicate resource names, stale-fence mutation, lease loss or double allocation across restart, registration capacity shrink under active leases, semantic accelerator drift, stale/wrong-manifest locality, unbounded evidence, output-trust conflation, and accidental provider order.
- **Budget:** repository source and checked-in Cairn policy are authoritative; two correlated advisory model passes; one architecture synthesis round; one post-implementation adversarial audit; focused retrieval first with follow-up only for missing coordinator/persistence/report seams; deterministic repository checks remain authoritative.
- **Allowed outcomes:** validated, blocked with exact evidence, exhausted, or user-decision-required. Plausible design prose and model agreement are not validation.

## Current seams and compatibility treatment

| Concern | Existing seam | Extension treatment |
|---|---|---|
| Lazy ready-goal ordering | `crates/crunch-build/src/scheduling.rs`: `normalize_eligible_preference`, `rank_ready_goals`, `SchedulingPolicy` | Keep comparator and starvation ordering authoritative. Derive concrete classes before preference. Do not synthesize graph pressure for worker placement. |
| Remote hard gates | `crates/crunch-build/src/distributed.rs`: `RemoteRouteEligibilityChecks`, `normalize_remote_route_scheduling_facts` | Resource admission remains a hard gate. Locality remains preference-only. |
| Durable attempt identity | `crates/crunch-build/src/distributed/remote_attempt.rs`: job/attempt/fence state and report authorization | Bind every resource lease to the same current job, attempt, fence, and worker. Do not put locks, clocks, or persistence in the core. |
| Receiver missing set | `crates/crunch-build/src/distributed/remote_transfer.rs`: canonical manifest, policy digest, receiver facts, `plan_remote_transfer_demand` | Recompute locality from the canonical manifest and receiver facts. Never accept claimed saved bytes or ordinal locality from a worker hint. |
| Worker registration | `src/remote_build.rs`: `RemoteWorkerRegistration`, `apply_worker_registration` | Add explicit worker generation and optional quantified inventory. Missing legacy inventory remains unknown and cannot satisfy quantified requests; concurrency is never converted into CPU or memory capacity. |
| Request admission | `src/remote_build.rs`: `RemoteCoordinatorBuildRequest`, `normalized_remote_build_key` | Add optional quantified scheduling requirements and locality scope. Scheduling quantities remain outside action identity; semantic accelerator classes are explicit identity inputs. |
| Atomic dispatch | `src/remote_build.rs`: clone candidate, persist, then publish state in `admit_coordinator_dispatch_with_nonce` | Reserve and persist the lease in the same candidate state before returning `Dispatch`. |
| Reassignment/reporting | `reassign_coordinator_attempt_with_nonce`, `apply_coordinator_attempt_report` | Replace/release leases only after current-fence validation, in the same durable candidate mutation. |
| Restart | `load_coordinator_state`, `migrate_legacy_coordinator_state`, coordinator state validation | Reconcile active leases against current attempts and inventories. Invalid aggregate state fails closed; superseded/terminal leases are explicitly released. |
| Typed config | `lib/remote-builders.ncl`, `src/remote_farm_config.rs` | Add bounded resource inventory/token fields with Rust/Nickel parity. Config never supplies locality observations. |
| Reports | coordinator status, build observability, scheduler priority evidence | Emit bounded quantities/digests/classes/reasons and explicit non-claims; omit object lists, credentials, and license material. |

Compatibility is explicit: absent resource inventories or requirements deserialize as `None`, produce `unknown` resource evidence, and do not claim quantified admission. A quantified request cannot use an unquantified worker. Existing unquantified routes remain compatibility behavior without fabricated capacity or locality.

## Approach registry

| Family | Mechanism | State | Evidence / blocker |
|---|---|---|---|
| Pure resource algebra | New pure distributed-core module canonicalizes vectors, uses checked arithmetic, derives availability/fit, and plans fenced lease reserve/release/resize/recovery | active | Matches FCIS and existing clone-persist shell. |
| Attempt-core lock | Put mutexes, clocks, or mutable counters inside `remote_attempt.rs` | falsified | Violates the pure attempt-core contract and cannot provide process-restart durability. One advisory pass suggested this and was rejected. |
| Inferred capacity | Treat concurrency or provider ordering as CPU/memory/token capacity | falsified | Would fabricate capacity and violate the change's explicit non-claim boundary. |
| Verified transfer derivation | Canonical transfer manifest + policy digest + receiver-probed facts produce demanded/present/missing counts and bytes | active | Reuses production missing-set semantics and checked arithmetic. |
| Self-reported/weighted locality | Trust an ordinal worker hint or divide claimed transfer by a locality factor | falsified | Fabricates savings and permits stale/adversarial claims. One advisory pass suggested this and was rejected. |
| Fake graph reuse | Create zero-pressure graph facts solely to pass worker candidates through ready-goal ranking | falsified | Would fabricate graph evidence. Worker placement instead compares only admitted resource/locality fields in configured order and stable worker identity; goal starvation remains in the existing scheduler. |

## Surviving architecture

1. `remote_resources.rs` is the pure functional core for canonical resource vectors, semantic classification, checked admission, lease identities, stale-fence authorization, mutation, recovery, locality normalization, and provider-neutral placement ordering.
2. `remote_build.rs` remains the imperative shell for worker probes, mutation locking, durable JSON state, registration, lease commit/release, restart reconciliation, and rendering.
3. Quantified requests require quantified worker inventory. The lease is durably committed before dispatch is returned.
4. Receiver-verified locality summaries are produced only by a shell probe calling the transfer core and locality core. Worker hints are not stored as verified summaries.
5. Scheduling-only counts are excluded from normalized action identity. Semantic accelerator classes are validated against requested accelerator classes and included in the normalized key.
6. Worker placement uses hard gates first, then configured resource/locality precedence and stable endpoint identity. It never consumes provider response order or invented known-graph facts.

## Advisory-pass limitations

Both model passes were correlated advisory reviews and contained invalid suggestions (mutable locks in the pure core, invented state variants, and arithmetic locality factors). They are retained only as adversarial counterexamples. Repository invariants and deterministic tests are authoritative.
