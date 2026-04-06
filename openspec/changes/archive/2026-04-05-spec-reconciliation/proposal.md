## Why

The audit found 7 places where specs contradict each other or contradict
the code. The code made correct engineering trade-offs but the specs were
never updated. This creates confusion — a new contributor reads the spec,
implements what it says, and produces wrong code.

Contradictions:

1. **nickel-eval spec** says "MUST NOT use JSON as intermediate format."
   Code uses JSON because `Expr::to_serde()` fails on nested Nickel enum
   tags. The spec was never updated.

2. **nickel-stdlib spec** says Derivation contract is "closed." Code uses
   an open contract (`..`) to support mkDerivation's extra fields. (This
   should re-close after extract-stdlib-builders lands, but the spec needs
   to document why it was opened and the planned fix.)

3. **ca-derivations spec** says "MUST assign provisional output paths
   using hash_placeholder(output_name)." Code uses input-addressed
   provisional paths and blake3-derived markers because hash_placeholder
   produces a different-length string. AGENTS.md documents the actual
   approach but the spec doesn't.

4. **persistent-pathinfo spec** says cache hit requires "output path exists
   on the filesystem." castore-store spec (newer) says the opposite: "NOT
   filesystem existence checks." Code follows castore-store. The older
   spec is stale.

5. **nickel-stdlib spec** says sandbox default is `'wasm`. Portability
   spec says v0 must use `'native` because WASI lacks process spawning.
   Code uses `'native`.

6. **fetchers spec** says `--fix` should "continue the build." Code
   returns an error telling the user to re-run. Re-run is correct because
   the derivation path changes after hash rewrite.

7. **architecture spec** crate table doesn't list `crunch-build`. The
   crate was extracted after the spec was written.

## What Changes

Update each contradicted spec to match the actual design decisions.
No code changes — this is a documentation-only change.

## Capabilities

### Modified Capabilities
- Specs match code and each other
- New contributors can trust the specs

## Impact

- **Files**: modified specs in `openspec/specs/`
- **APIs**: none
- **Dependencies**: none
- **Testing**: none
