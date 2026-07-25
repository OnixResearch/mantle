## Why

`bootstrap/perl-5.005_03-gcc.ncl` is the generator boundary consumed by the GCC-built Perl 5.6.2 and downstream autotools ladder, but its committed runtime success was not independently proven. Three untracked one-variable diagnostics suggested optimization sensitivity, missing LP64 configuration, or predecessor-compiler sensitivity, while the first reachable failure actually occurred earlier: Perl 5.000 searched for raw compiler inputs that `gcc-generator-base-v4` intentionally encapsulates.

Leaving the predecessor boundary broken while promoting a target-local workaround would misclassify a closure failure as Perl behavior. Mantle needs a bounded comparison that first reaches the Perl builder, preserves exact precursor failures, repairs the smallest causal boundary, and accepts the canonical target only when positive execution and malformed-source rejection both pass.

## What Changes

- Capture the canonical Perl 5.005_03 GCC build baseline and distinguish environment, store-identity, and predecessor failures from target runtime behavior.
- Repair Perl 5.000's normalized generator-base handoff without flattening hidden raw inputs or changing compiler lineage.
- Establish that canonical Perl 5.005_03 succeeds at GCC 4.0.4 and `-O2`; reject `-O0`, explicit `LONGSIZE=8`, and GCC 10 as unnecessary target mutations, then delete the temporary variants.
- Add deterministic positive and negative regression coverage for the predecessor handoff and target runtime contract.
- Preserve explicit non-claims about compiler correctness, provider admission, whole-chain reproducibility, and downstream behavior beyond the separately executed focused Perl 5.6.2 build.

## Impact

- **Files**: `bootstrap/perl-5.000-gcc.ncl`, `bootstrap/perl-5.005_03-gcc.ncl`, `tests/bootstrap_eval.rs`, removal of temporary diagnostic variants, and lifecycle evidence under this change.
- **Testing**: precursor isolation, canonical default-prefix build, positive runtime smoke, negative malformed-source rejection, source-pin/evaluation checks, focused downstream Perl 5.6.2 build, and Cairn validation/gates.
