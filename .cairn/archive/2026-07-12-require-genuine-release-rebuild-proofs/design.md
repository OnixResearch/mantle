## Context

The real-release script creates `rebuild-stage2-mantle-copy.sh` inside the release bundle and invokes it through BusyBox. The deterministic sandbox mounts the complete bundle read-only and the helper copies `binaries/01-stage2-mantle` to the output root. The current proof receipt records source/proof digests and a path-and-argv command string, but it does not bind recipe or executable bytes and does not distinguish a rebuild from a copy of the comparison target.

Two isolated stores and matching output digests are useful only after the proof establishes that each output was produced without read authority over the published output bytes.

## Decisions

### Separate rebuild authority from comparison authority

The comparison shell retains the published target artifacts and measures their BLAKE3 identities. It passes only target names and expected identities to the pure planner. Each proof sandbox receives a separately materialized, explicit read-only rebuild input closure containing reviewed source, recipe, toolchain, and policy artifacts; the complete release bundle is not mounted into the rebuild sandbox.

The planner rejects any candidate rebuild input whose measured content identity equals a published target identity, including alternate paths in the proof bundle. Execution also denies direct paths, symlink aliases, hardlink aliases, prior run outputs, and the ordinary reproduce output root. Published bytes become available only to the comparison shell after each sandbox exits.

### Bind the executable rebuild contract by content

A versioned rebuild descriptor commits to:

- canonical recipe bytes and BLAKE3;
- executable and tool object identities, byte digests, and roles;
- ordered argument encoding;
- source and vendor/input closure identities;
- selected provider and target artifact identities;
- sandbox, effect, and determinism-normalization policy identities; and
- the approved read roots and fresh write roots for each run.

Path strings and version output may remain diagnostic fields, but they cannot substitute for content identities. The descriptor digest participates in every run receipt and in the final deterministic proof receipt.

### Make stronger admission explicit

The deterministic proof schema is versioned so release verification can distinguish content-bound genuine rebuild evidence from legacy path-bound evidence. A promoting verdict requires every run to cite the same accepted rebuild descriptor, use distinct fresh output/store roots, have no target-authority or undeclared-input observations, and produce the exact selected output set.

Legacy receipts remain parseable for diagnostics when practical, but they produce a non-promoting disposition. The standalone receipt checker and summary scripts apply the same admission rule as `mantle release verify`.

### Preserve functional core and imperative shell boundaries

The pure core receives normalized target identities, candidate input observations, rebuild descriptors, run observations, and policy. It plans allowed roots, classifies blockers, canonicalizes identities, and decides eligibility without filesystem, environment, process, clock, or output effects.

The shell measures bytes, opens capability roots, materializes the approved closure, launches the sandbox, captures observations, compares output bytes, writes receipts, and renders diagnostics. The production script orchestrates the shell but cannot mint eligibility by choosing a matching output digest alone.

### Validate the actual production rail

A positive fixture builds a small selected artifact twice from declared source through the production rebuild interface. Negative fixtures try the current direct copy, an alternate content-identical proof-bundle path, a symlink/hardlink alias, modified recipe bytes at the same path, modified tool bytes, an undeclared source, and a prior output root. Each negative fixture must fail before a promoting receipt or summary is emitted.

## Risks / Trade-offs

- Removing the complete bundle mount requires an explicit rebuild closure and may expose missing toolchain/source declarations.
- Content identity does not prove compiler or recipe semantics; the claim remains bounded to execution of the reviewed, identified recipe without target-byte authority.
- A receipt schema revision requires compatibility diagnostics for existing evidence, but accepting legacy evidence for promotion would retain the defect.
