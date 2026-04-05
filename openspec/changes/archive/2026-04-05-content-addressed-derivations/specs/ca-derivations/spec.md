# Content-Addressed Derivations Specification

## Purpose

Defines how crunch computes output paths from build output content
rather than from derivation inputs, enabling deduplication of
identical outputs and avoiding unnecessary rebuild cascades.

## Requirements

### Requirement: CA output path computation

For content-addressed derivations, the output store path MUST be
computed after the build completes, as the BLAKE3 hash of the NAR
serialization of the output. The path format is:

```
/nix/store/<nixbase32(compress(blake3(nar(output))))>-<name>
```

This uses `build_ca_path` with `CAHash::Nar(NixHash::Sha256(nar_hash))`
— except crunch uses BLAKE3 for the outer fingerprint hash per the
defaults spec. The inner content hash (the NAR hash) also uses BLAKE3.

#### Scenario: Identical outputs from different inputs

- GIVEN derivation A built with gcc-13 producing output bytes X
- AND derivation B built with gcc-14 producing identical bytes X
- WHEN both builds complete
- THEN both output paths are identical

#### Scenario: Different outputs get different paths

- GIVEN derivation A producing output bytes X
- AND derivation B producing output bytes Y (X ≠ Y)
- WHEN both builds complete
- THEN the output paths differ

### Requirement: Provisional paths and placeholders

Before a CA build starts, the system MUST assign provisional output
paths using `hash_placeholder(output_name)`. These placeholders are
set in the build environment so the builder can reference `$out`.

After the build, the provisional paths are replaced with the final
content-addressed paths.

#### Scenario: Builder sees $out

- GIVEN a CA derivation with output "out"
- WHEN the build script runs
- THEN `$out` contains a placeholder path that the builder can
  write to

#### Scenario: Placeholder is replaced post-build

- GIVEN a build that writes files to `$out`
- WHEN the build completes
- THEN the output is moved from the placeholder path to the
  final content-addressed path

### Requirement: Self-reference rewriting

If a CA build output contains references to its own provisional
path (e.g., a binary with an embedded RPATH, or a script with
a hardcoded store path), the system MUST rewrite those references
to the final content-addressed path.

The rewriting MUST be byte-level: scan the output for occurrences
of the provisional path string and replace with the final path
string. Both strings MUST have the same length (they do — store
paths are fixed-width).

#### Scenario: Binary with embedded store path

- GIVEN a CA derivation that compiles a binary
- AND the binary contains the provisional path in its RPATH
- WHEN the build completes and the content hash is computed
- THEN the provisional path is rewritten to the final path
  in the output

#### Scenario: No self-references

- GIVEN a CA derivation producing a plain text file
- WHEN the build completes
- THEN no rewriting occurs (nothing to replace)

### Requirement: Input reference rewriting

If a CA derivation's inputs are themselves CA derivations whose
final paths differ from their provisional paths, the system MUST
rewrite references to input provisional paths in the output.

This is necessary for transitive correctness: if `libfoo`'s final
CA path differs from its provisional path, and `myapp` embeds
`libfoo`'s path in its binary, the embedded path must be the
final one.

#### Scenario: Transitive CA reference

- GIVEN CA derivation `libfoo` with provisional path P and
  final path F (P ≠ F)
- AND CA derivation `myapp` that depends on `libfoo`
- WHEN `myapp`'s output contains string P
- THEN P is rewritten to F in `myapp`'s output

### Requirement: KnownPaths two-phase resolution

`KnownPaths` MUST support entries where output paths are initially
`None` (not yet built) and are resolved to final paths after the
build.

The structure MUST provide:

- `insert_provisional(aterm_hash, drv_path, hdm, derivation)` —
  register a CA derivation before build, with `None` output paths
- `resolve_output(drv_path, output_name, final_path, path_info)` —
  set the final path after build completes
- `get_output_path(drv_path, output_name) -> Option<StorePath>` —
  returns `None` if not yet built, `Some(path)` if resolved

Downstream derivations that reference a CA derivation's output
MUST block until the output path is resolved.

#### Scenario: Sequential build resolves paths

- GIVEN CA derivation A (not yet built) and derivation B
  depending on A
- WHEN A is built and its output path resolved
- THEN B's build can proceed with A's final output path

### Requirement: Addressing mode in Nickel

The `Derivation` contract MUST include an `addressing_mode` field:

```nickel
addressing_mode | [| 'input-addressed, 'content-addressed |]
               | default = 'content-addressed
```

- `'content-addressed` — output path computed from content
  (new default)
- `'input-addressed` — output path computed from inputs
  (v0 behavior, needed for compatibility)

#### Scenario: Default is content-addressed

- GIVEN a derivation without an explicit `addressing_mode`
- WHEN evaluated
- THEN `addressing_mode` is `'content-addressed`

#### Scenario: Opt into input-addressed

- GIVEN a derivation with `addressing_mode = 'input-addressed`
- WHEN built
- THEN output paths are computed from inputs (v0 behavior)

### Requirement: FODs are unaffected

Fixed-output derivations MUST continue to compute output paths
from the declared hash, regardless of `addressing_mode`. FODs
are content-addressed by definition.

#### Scenario: FOD with content-addressed mode

- GIVEN a FOD with `addressing_mode = 'content-addressed` and
  `fixed_output = { hash = "sha256-...", ... }`
- WHEN built
- THEN the output path is computed from the declared hash
  (same as v0 FOD behavior)

### Requirement: Multi-output CA derivations

For derivations with multiple outputs, each output MUST get its
own content-addressed path computed independently from its own
NAR hash. Different outputs of the same derivation MAY have
different final paths.

#### Scenario: Multi-output CA

- GIVEN a CA derivation with outputs "out" and "lib"
- WHEN built, "out" produces bytes X and "lib" produces bytes Y
- THEN "out" path = hash(X), "lib" path = hash(Y)

### Requirement: Cache behavior with CA

The cache check for CA derivations MUST use a mapping from
derivation identity to final output path (stored in
`PathInfoService`). A cache hit means: this derivation was
previously built, and the output at the final CA path still
exists.

The system MUST NOT use the provisional path for cache checks.

#### Scenario: Rebuilt derivation with same output

- GIVEN a CA derivation previously built with output path P
- WHEN the derivation is rebuilt (same inputs)
- THEN the build produces the same content, computes the same
  CA path P, and reports a cache hit (or deduplicates)

### Requirement: Determinism property

For deterministic builds, CA derivations MUST be idempotent:
building the same derivation twice MUST produce the same output
path. For non-deterministic builds, different runs MAY produce
different output paths (different content → different hash).

#### Scenario: Deterministic build

- GIVEN a CA derivation with a deterministic build script
- WHEN built twice
- THEN both builds produce the same output path

#### Scenario: Non-deterministic build

- GIVEN a CA derivation that embeds a timestamp
- WHEN built twice at different times
- THEN the output paths MAY differ
