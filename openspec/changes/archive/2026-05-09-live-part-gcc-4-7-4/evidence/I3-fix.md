# I3 source hardening evidence

Task-ID: I3
Covers: bootstrap.part.gcc.4.7.4

- Added source provenance and first-consumer comments beside the `gcc47_src` fixed-output fetch.
- Removed the C-only configure fallback so this part cannot silently lose its C++ provider contract.
- Removed suppressed `all-gcc`, `all-target-libgcc`, `install-gcc`, and `install-target-libgcc` failures and manual partial-install copy fallback.
- Added fail-closed output checks for `cc`, `c++`, `cc1`, `cc1plus`, and `libgcc.a`.
- Converted C, C++, and C++11 compile smokes from warnings into fail-closed checks.
