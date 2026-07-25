## Why

`bootstrap/perl-5.005_03-gcc.ncl` is the generator boundary consumed by the GCC-built Perl 5.6.2 and downstream autotools ladder, but its committed runtime success is not independently proven. Three untracked one-variable diagnostics test plausible causes—optimization sensitivity, missing LP64 configuration, and predecessor-compiler sensitivity—without durable evidence or a reviewed promotion rule.

Leaving those variants as loose files risks either losing useful causal evidence or promoting a coincidental workaround. Mantle needs a bounded comparison that reaches the Perl builder, preserves the first exact failure for every candidate, and changes the canonical derivation only when positive execution and malformed-source rejection both pass.

## What Changes

- Capture the canonical Perl 5.005_03 GCC build baseline and distinguish precursor/store-identity failures from Perl runtime failures.
- Compare the canonical construction against `-O0`, explicit `LONGSIZE=8`, and final GCC 10 variants while holding other declared inputs constant.
- Promote only the smallest evidence-backed correction into `bootstrap/perl-5.005_03-gcc.ncl`; delete temporary diagnostic variants after their outcomes are recorded.
- Add a deterministic regression rail covering version output, arithmetic execution, malformed-source rejection, and ELF shape.
- Preserve explicit non-claims about compiler correctness, provider admission, whole-chain reproducibility, and downstream Perl 5.6.2 success.

## Impact

- **Files**: `bootstrap/perl-5.005_03-gcc.ncl`, temporary diagnostic variants under `bootstrap/`, focused validation code or fixtures if needed, and lifecycle evidence under this change.
- **Testing**: canonical pre-change build, bounded diagnostic matrix, positive runtime smoke, negative malformed-source rejection, source-pin/evaluation checks, Cairn validation/gates, and focused downstream Perl 5.6.2 evaluation or an exact blocker.
