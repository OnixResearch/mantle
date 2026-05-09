# V3 automake 1.10.3 smoke gate evidence

Task-ID: V3
Covers: bootstrap.part.automake.1.10.3

The smoke contract for this part is conditional on a produced `automake-1.10.3` output. Because V2 has no output path while prerequisites remain blocked or gated, no `automake`, `automake-1.10`, `aclocal`, or `aclocal-1.10` smoke success is claimed.

The hardened derivation now fails closed unless those entrypoints and `share/automake-1.10` are installed.
