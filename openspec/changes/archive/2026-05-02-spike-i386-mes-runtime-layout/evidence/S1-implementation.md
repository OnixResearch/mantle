# S1 implementation

Task-ID: S1
Covers: bootstrap.i386-mes-runtime-layout.spike

Added `bootstrap/spike-i386-mes-runtime-layout.ncl`, a sibling diagnostic derivation that imports the proven `spike-i386-tinycc26-cross-smoke.ncl` predecessor. The derivation creates an i386 Mes header/CRT layout from Mes 0.27.1 sources, then probes the TinyCC 0.9.27 handoff without mutating production `bootstrap/make-tcc.ncl`.
