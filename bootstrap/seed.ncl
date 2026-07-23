# Selected bootstrap seed provider.
#
# The normalized full-source provider passed committed-source construction,
# runtime admission, complete source-closure authentication, and independent
# BLAKE3 binding before this selector changed. Its own derivation still fails
# closed if any required compiler, C++, binutils, CRT, libc, libgcc, shared
# runtime, relocation, or rejection surface is absent.
#
# The legacy musl.cc provider remains available only through the explicit
# `mantle bootstrap --fetch` compatibility path; it is not a selector fallback.

import "seed-full.ncl"
