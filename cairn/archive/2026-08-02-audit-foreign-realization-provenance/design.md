# Design: Castore-backed foreign output provenance audit

## Context

Foreign graph compilation checks declared fields before execution. Real builders can still emit bytes that contain original foreign paths or unexpected executable content.

Mantle PathInfo and castore services already identify output roots and closure references. `src/build_correctness.rs` validates reference observations supplied by a caller. The missing component is a production observation generator.

## Decisions

### Decision 1: Scan admitted castore objects

**Choice:** The audit shell will walk signed PathInfo and castore nodes. It will not scan ambient host paths or trust exported filesystem placement.

Every visited root, directory, blob, and symlink will use bounded worklists and duplicate suppression by canonical node identity.

**Rationale:** Castore is the content authority admitted by Mantle. Host paths can be replaced or incomplete.

### Decision 2: Keep classification pure

**Choice:** A pure core will classify in-memory metadata and bounded byte prefixes. The shell will retrieve bytes and metadata from castore.

The core will return typed observations and diagnostics. It will not read files, open stores, print, or execute payloads.

**Rationale:** Classification and policy need deterministic positive and negative tests without store mocks.

### Decision 3: Use explicit payload classes

**Choice:** Classify regular data, executable ELF, executable script, symlink, supported archive, supported initrd, and unsupported executable payloads.

Mode bits, magic bytes, shebang syntax, archive format, and configured limits will drive classification. A heuristic text label alone cannot authorize executable content.

**Rationale:** Unknown executable bytes require review. Silent data classification would hide risk.

### Decision 4: Resolve executable provenance against closure facts

**Choice:** Resolve script shebang interpreters and recognized executable store references against admitted closure paths and exact foreign-to-target path maps.

The scanner will reject unresolved target paths, foreign store paths, path escapes, malformed shebangs, and reference targets outside the admitted closure.

**Rationale:** Classification without target resolution does not establish translated executable provenance.

### Decision 5: Inspect containers with bounded recursion

**Choice:** Supported archive and initrd formats will use format-specific bounded readers. Nested entry count, expanded bytes, path depth, and recursion depth will have named policy limits.

Unsupported compressed or executable containers will produce an unclassified finding. The scanner will not invoke host archive tools.

**Rationale:** Executable content can hide inside containers. Unbounded extraction is unsafe.

### Decision 6: Reuse build-correctness validation

**Choice:** Convert scanner observations into the existing build-correctness reference model where it applies. Add new typed observations only for payload classes that model cannot represent.

**Rationale:** One reference policy avoids competing output-trust rules.

### Decision 7: Separate realized and audited states

**Choice:** A completed build remains realized even when audit fails. The realization receipt will reference the audit result and report the strongest completed state.

Only a passing audit can report `provenance-audited`. That state proves bounded path and classification facts only.

**Rationale:** Audit failure does not erase valid build observations. It blocks stronger provenance claims.

### Decision 8: Emit deterministic audit evidence

**Choice:** Emit `mantle-foreign-provenance-audit-v1`. It will bind root and closure identities, path-map digest, profile digest, scanner policy, limits, observation digest, findings, disposition, and non-claims.

BLAKE3 will identify Mantle-owned policy, observation, and receipt artifacts.

**Rationale:** Reviewers need exact scan scope and limits, not a Boolean result.

## Failure Semantics

- Missing or invalid PathInfo fails before content scanning.
- Missing castore content produces an incomplete-closure finding and a failed audit.
- Any untranslated foreign store path fails the affected audit.
- Any unclassified executable or container payload fails the affected audit.
- Malformed ELF, shebang, symlink, archive, or initrd data produces a stable typed finding.
- Limit exhaustion fails closed and records the exact exhausted limit.
- The scanner never executes, repairs, deletes, or rewrites output content.

## Risks / Trade-offs

- Bounded byte inspection can require format-specific parsers.
- Deep archive inspection adds runtime and memory cost.
- Static reference scanning cannot prove runtime behavior or dynamic code generation.
- A strict unknown-executable policy can block unusual but valid packages until classification support is added.
