# V3 automake 1.15.1 smoke gate evidence

Task-ID: V3
Covers: bootstrap.part.automake.1.15.1

The smoke contract for this part is conditional on a produced `automake-1.15.1` output. Because V2 has no output path while prerequisites remain blocked or gated, no `automake`, `automake-1.15`, `aclocal`, or `aclocal-1.15` smoke success is claimed.

The hardened derivation now fails closed unless those entrypoints and `share/automake-1.15` are installed.
