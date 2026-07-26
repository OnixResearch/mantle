# Canonical early-native GCC 4.0 parity root.
#
# The selected derivation performs the complete same-version bootstrap cycle:
# TCC/native-musl handoff -> GCC C passes -> regenerated GCC 4.0.4 C/C++.
# Row admission remains external and requires the independently validated,
# BLAKE3-bound receipt in bootstrap/evidence/early-native-gcc40-row-v2.json.

import "gcc-4.0-musl-cxx.ncl"
