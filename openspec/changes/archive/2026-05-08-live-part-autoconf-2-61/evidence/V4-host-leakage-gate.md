# V4 autoconf 2.61 host-leakage gate

Task-ID: V4
Covers: bootstrap.part.autoconf.2.61

A target-specific host-leakage scan cannot be completed until V2 produces an Autoconf build transcript. This evidence preserves the invariant instead:

- `bootstrap/autoconf-2.61.ncl` must continue to resolve Make, Sed, M4, Perl, Coreutils, Gawk, Grep, Diffutils, Bash, Stage0, predecessor Automake, and source inputs through declared derivation inputs;
- the unvalidated predecessor state must not be replaced with host Autoconf/Automake/Perl/Make, Nix packages, or undeclared tools;
- future positive evidence must scan the actual Autoconf 2.61 build transcript before promotion.

No no-host-leakage success over an Autoconf build transcript is claimed here, because no such transcript exists yet.
