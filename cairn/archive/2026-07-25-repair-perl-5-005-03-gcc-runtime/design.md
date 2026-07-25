## Context

The canonical derivation uses the source-built GCC 4.0 C/C++ provider with `-O2`, declares `INTSIZE=4`, and does not declare `LONGSIZE`. The three diagnostic files differed by one intended mechanism: compile the same source at `-O0`, declare `LONGSIZE=8`, or compile with the final GCC 10 provider. All retained the same generated-header path, bounded miniperl source list, runtime arithmetic smoke, malformed-source rejection, and ELF64 inspection.

Initial runs stopped before Perl 5.005_03: first at host sandbox prerequisites, then at unmaterialized GCC and predecessor Perl outputs. The first source defect was in Perl 5.000, which imported the normalized `gcc-generator-base-v4` through Bison's inputs but searched for the base's hidden raw GCC/musl/binutils paths. Those runs provide no target optimization, ABI, compiler-generation, or runtime evidence.

## Decisions

### Decision: reach the target before varying target mechanisms

**Choice:** Classify and repair predecessor/input-closure failures before executing target-local optimization, LP64 configuration, or compiler-generation variants. Run those variants only if the canonical target fails after its declared predecessors succeed.

**Rationale:** A target-local workaround cannot explain a failure that occurs before the target builder runs. Once the canonical target passes the full behavioral contract, further variants add no causal value and risk promoting unnecessary semantic changes.

### Decision: require execution and rejection evidence

**Choice:** A candidate is viable only when it builds an ELF64 interpreter, reports Perl 5.005_03, evaluates the arithmetic smoke correctly, and rejects malformed Perl source with nonzero status and no stdout.

**Rationale:** Compilation, installation, `perl -v`, or a zero exit from the build command alone can hide a broken parser or runtime.

### Decision: preserve the normalized generator-base boundary

**Choice:** Make Perl 5.000 consume compiler/runtime tools from `gcc-generator-base-v4` and invoke its relocated GCC through explicit `-B`, libc, libm, and CRT seams. Do not flatten the base's raw input closure or change compiler lineage.

**Rationale:** The generator base exists to provide a linear direct-input DAG and a relocated toolchain contract. Searching for its hidden predecessors violated that contract; flattening them again would reintroduce the dependency shape the base was designed to remove.

### Decision: keep diagnostic artifacts temporary

**Choice:** Record exact commands, reports, logs, and outcomes under lifecycle evidence, then delete the three hidden Nickel variants before closeout.

**Rationale:** Untracked near-duplicate derivations are not a durable test matrix and can drift from the canonical source.

## Outcome

The normalized Perl 5.000 handoff is the only compiler/runtime construction change. With that repair, Perl 5.000, 5.003, and 5.004_05 built in order, and canonical Perl 5.005_03 passed under GCC 4.0.4 and `-O2` without `LONGSIZE`. The three target-local diagnostics were therefore classified as unnecessary and removed without execution. `bootstrap/perl-5.005_03-gcc.ncl` changes only its stale malformed-source error label.

## Risks / Trade-offs

- Existing local action results are keyed by store prefix; a fresh prefix can rebuild a long precursor chain before Perl. Such runtime cost is not a target failure.
- A candidate can pass the narrow runtime smoke yet remain unsuitable as a generator for Perl 5.6.2. The focused downstream build is separate evidence.
- Reusing local authenticated action results accelerates causal isolation but is not an independent clean-room rebuild or compiler-correctness proof.
