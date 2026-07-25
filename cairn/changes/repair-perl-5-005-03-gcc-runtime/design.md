## Context

The canonical derivation uses the source-built GCC 4.0 C/C++ provider with `-O2`, declares `INTSIZE=4`, and does not declare `LONGSIZE`. The three diagnostic files differ by one intended mechanism: compile the same source at `-O0`, declare `LONGSIZE=8`, or compile with the final GCC 10 provider. All retain the same generated-header path, bounded miniperl source list, runtime arithmetic smoke, malformed-source rejection, and ELF64 inspection.

A first baseline using the current `/mantle/store` identity stopped at a fresh `stage0-posix` dependency and therefore provides no Perl evidence. The comparison must use a precursor identity/state that reaches the target builder, or record that exact precursor blocker without misclassifying it as a Perl result.

## Decisions

### Decision: compare mechanisms, not filenames

**Choice:** Treat optimization mode, LP64 configuration, and compiler generation as separate approach families. Hold all other source, generator, libc, binutils, and smoke behavior constant during the first comparison round.

**Rationale:** Simultaneous changes could produce a passing binary without identifying the causal requirement and would make the canonical correction unauditable.

### Decision: require execution and rejection evidence

**Choice:** A candidate is viable only when it builds an ELF64 interpreter, reports Perl 5.005_03, evaluates the arithmetic smoke correctly, and rejects malformed Perl source with nonzero status and no stdout.

**Rationale:** Compilation, installation, `perl -v`, or a zero exit from the build command alone can hide a broken parser or runtime.

### Decision: prefer the smallest semantic correction

**Choice:** Prefer a target-ABI configuration correction over disabling optimization, and prefer preserving the source-built GCC 4.0 predecessor over switching to GCC 10, when evidence proves the smaller correction is sufficient. A compiler switch is acceptable only if the predecessor is demonstrated unsound for this source and the closure/claim change is explicit.

**Rationale:** `LONGSIZE=8` describes the selected x86_64 LP64 ABI. `-O0` can mask compiler defects, while switching compiler generations changes the bootstrap dependency claim.

### Decision: keep diagnostic artifacts temporary

**Choice:** Record exact commands, reports, logs, and outcomes under lifecycle evidence, then delete the three hidden Nickel variants before closeout.

**Rationale:** Untracked near-duplicate derivations are not a durable test matrix and can drift from the canonical source.

## Risks / Trade-offs

- Existing local action results are keyed by store prefix; a fresh prefix can fail in precursor construction before Perl. Such runs are environment/precursor evidence only.
- A candidate can pass the narrow runtime smoke yet remain unsuitable as a generator for Perl 5.6.2. The focused downstream evaluation/build remains a separate check.
- Reusing local authenticated action results accelerates causal isolation but is not an independent clean-room rebuild or compiler-correctness proof.
