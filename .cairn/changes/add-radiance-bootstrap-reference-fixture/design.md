## Context

Radiance is a self-hosted compiler that targets RV64. Its repository contains a checked seed and fixed-point workflow.

Radiance.s0 is a separate C99 implementation that can produce the self-hosted compiler. The emulator runs the resulting RV64 programs.

All three reviewed repositories use Git SHA-256 objects and MIT licenses.

## Completion Contract

Completion requires two independently started routes to reach admitted route-local fixed points under one explicit Mantle proof profile.

Every source, tool, predecessor, execution, and output must have an exact identity. Equality remains evidence of convergence only.

A live fetch, ambient compiler, undeclared executable, substituted predecessor, stale receipt, or fixed-point correctness claim is false completion.

## Decisions

### Decision: Keep the fixture optional and external

**Choice:** Place the fixture under the examples or bootstrap-reference surface. Do not add it to ordinary builds or default release acceptance.

**Rationale:** The fixture tests proof machinery. It is not required to build Mantle.

### Decision: Bind a three-repository source cohort

**Choice:** The source profile names exact tagged Git SHA-256 objects, source-tree BLAKE3 values, repository roles, MIT licenses, projection rules, and snapshot profiles.

**Rationale:** A branch name or portal URL cannot identify the reviewed source bytes.

### Decision: Use authenticated offline source bundles

**Choice:** Connected preparation acquires and verifies exact sources. The proof imports one bundle, pins it, and forbids live source acquisition.

**Rationale:** The run must expose complete source closure and remove network fallback.

### Decision: Separate the two route roots

**Choice:** Route `seed` starts from the admitted Radiance RV64 seed. Route `c99` starts from an admitted host C compiler that builds Radiance.s0.

Both routes build or use the admitted emulator and then compile the same Radiance source projection.

**Rationale:** Different starting roots provide useful diversity without pretending that either root is trusted.

### Decision: Enforce immediate predecessor lineage

**Choice:** Each stage must execute the compiler produced by its declared immediate predecessor. StageX or the current protected execution mechanism records every admitted executable.

**Rationale:** Output equality is weak if an ambient compiler can replace an intermediate stage.

### Decision: Compare route-local and cross-route outputs

**Choice:** Each route compares stage one with stage two by exact BLAKE3 and byte length. A separate comparison records whether the two route fixed points match.

**Rationale:** Route-local convergence and cross-route convergence are different observations. Neither proves semantic correctness.

### Decision: Export bounded reusable artifacts

**Choice:** The receipt can publish exact RV64 programs and compiler artifacts for later differential-execution consumers. Publication uses immutable identities and no sibling path.

**Rationale:** Reuse must occur through published artifacts, not the active Mantle worktree.

## Functional Core and Imperative Shell

Pure cores own source-cohort admission, stage graph validation, predecessor checks, convergence classification, receipt admission, and non-claim enforcement.

Shells own source acquisition, source-bundle I/O, C compilation, emulator execution, protected process launch, artifact measurement, and publication.

## Proof Graph

```text
seed route:
  admitted RV64 seed
    -> compile Radiance source
    -> route stage one
    -> compile Radiance source
    -> route stage two

c99 route:
  admitted host C compiler
    -> build Radiance.s0
    -> compile Radiance source
    -> route stage one
    -> compile Radiance source
    -> route stage two
```

Each arrow records source, predecessor executable, argv, environment policy, result, and output identity.

## Evidence

The receipt binds:

- three external repository identities and licenses;
- source-bundle and source-state identities;
- host C compiler and emulator build identities;
- every stage executable and immediate predecessor;
- protected execution audit;
- route-local fixed-point outcomes;
- cross-route comparison;
- zero live fetch, substitution, fallback, and ambient discovery counts;
- explicit non-claims.

## Positive and Negative Verification

Positive fixtures cover source admission, graph construction, two route-local fixed points, matching and differing cross-route outputs, replay, and publication.

Negative fixtures cover source drift, wrong Git format, missing license, seed drift, compiler substitution, ambient tools, live fetch, wrong predecessor, output mutation, and overclaims.

## Risks and Trade-offs

- The host C compiler remains an explicit route root and trust input.
- External projects can change or disappear. Offline bundles preserve the reviewed source bytes.
- The full run can be costly. Default checks validate contracts and small frozen fixtures only.

## Claim Boundary

A passing receipt proves bounded source, lineage, execution, and convergence facts for the exact cohort. It does not prove compiler correctness or seed trust.
