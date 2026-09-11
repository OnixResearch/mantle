# Proposal: Add machine-wide compile slots

## Why

Mantle's Worker bounds parallelism per build, but nothing bounds machine-wide
compile concurrency across builds. A proof session with several parallel
derivations, each running its own parallel compile, oversubscribes the host;
the recorded long-proof sessions run for hours with contention and pueue
co-existence. The reviewed external reference solves this in its cache
daemon: compilers start only when the machine-wide slot authority grants a
slot, and the slot authority is the same daemon that serves the compile
cache (`evidence/repkgs-review.md`).

In Mantle this composes with `extend-compile-cache-to-cc`: the driver seam is
already the single choke point for bootstrap compiler invocations, so it can
request a slot before running a compile. The trust framing is identical to
the cache: a scheduling device, never evidence, with no influence on outputs.

## What Changes

- Define a machine-wide slot authority: a per-machine service that grants
  bounded compile slots to requesting builds, with an explicit policy of
  total slots, per-build reservation bounds, and queue limits.
  r[mantle.compile_slots.slot_authority]
- Guarantee build degradation: a build without the authority, or when the
  authority is unavailable, MUST run with its own scheduling unchanged and
  MUST NOT fail. r[mantle.compile_slots.degrade_without_authority]
- Bound fairness: slot grants MUST have bounded wait, starvation controls,
  and per-build fairness so one large build cannot monopolize the machine.
  r[mantle.compile_slots.bounded_fairness]
- Keep outputs uninfluenced: slots affect scheduling only; outputs, receipts,
  and derivation identities MUST be identical with and without slot
  enforcement. r[mantle.compile_slots.no_output_influence]

## Impact

- **Immediate consumer**: long source-built proof sessions and the
  `extend-compile-cache-to-cc` driver seam; any host running parallel Mantle
  builds.
- **Immediate outcome**: parallel bootstrap derivations share the machine's
  compile capacity deliberately instead of oversubscribing.
- **Durable capability**: a machine-local scheduling boundary that later
  serves non-compiler resources if proven.
- **Maintenance owner**: Mantle cache and scheduling owner, covering the
  daemon extension and the driver seam.
- **Repeatability evidence**: slot-grant receipts, contention fixtures,
  degradation fixtures, and output-identity runs with slots on and off.
- **Compatibility**: builds never require the authority; absence is today's
  behavior.

## Scope

The change covers the slot authority service, the driver integration, the
policy, receipts, and positive and negative fixtures.

## Out of Scope

- Distributed or multi-machine scheduling (remote-build admission owns its
  domain).
- Slot enforcement for non-compiler processes.
- Any claim that slots improve correctness or reproducibility.

## Success Criteria

- Under concurrent builds, total concurrent compiles never exceed the
  declared slot count, with grants recorded.
- Killing the authority mid-build leaves the build running and successful
  with its own scheduling.
- Outputs are byte-identical with slots on and off.
- A starving build eventually receives slots under declared fairness policy.
