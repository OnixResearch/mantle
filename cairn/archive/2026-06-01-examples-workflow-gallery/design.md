## Context

Users need examples that answer both "how do I write a derivation?" and "how do I trust what Mantle built?" Existing examples cover simple derivations, fetchers, package sets, projects, and bootstrap, but the path is not curated and some examples are placeholders or generated-input dependent.

## Approach

1. Organize the gallery into progressive lanes: beginner, fetcher cookbook, package composition, project workflow, trust/provenance, and advanced bootstrap.
2. Add small examples before large ones. Beginner and composition examples should be local and fast; advanced examples can require generated seed material or explicit heavyweight commands.
3. Add project workflow examples that show default package, named package, checks, and expected output inspection.
4. Add provenance/trust examples only when the commands can produce deterministic local evidence. If full release/witness workflows are too heavy, keep them as command skeletons with non-claim language and link to real proof docs.
5. Keep examples frontend-neutral. They can show Mantle receiving concrete derivations, project data, or opaque evaluated data, but they must not reintroduce Onix/NixOS-style module-layer semantics into Mantle examples.

## Risks

- Too many examples can become noise unless the catalog and README preserve the learning path.
- Provenance examples can overstate trust if they use fake evidence; use explicit non-claim wording for skeletons.
- Advanced examples can break on fresh checkouts without generated seed material; catalog their prerequisites.

## Validation

- The gallery README orders examples by lane and capability.
- Fast lanes evaluate/build/execute under deterministic tests.
- Trust/provenance examples have either executable local smoke coverage or explicit non-claim documentation.
- Boundary tests continue rejecting module-layer examples under `examples/`.
