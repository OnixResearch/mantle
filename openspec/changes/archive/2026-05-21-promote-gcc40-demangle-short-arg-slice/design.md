# Design

Add a fourth bounded argument kind for Itanium type code `s` (`short`) to the existing static-buffer demangler. This is deliberately adjacent to existing `i`, `c`, and `l` handling and preserves the same rejection behavior for unsupported type codes and deeper nested names.

The receipt advances to `mantle-gcc40-native-demangle-slice-v6`, uses `selected_shape=single-short-arg-itanium-v6`, records v5 markers as forbidden stale markers, and keeps the bounded non-claim text explicit.

Validation updates require the new accepted input, transcript entries, new source markers, and digest recomputation. Existing v0-v5 stale markers must fail closed if reintroduced.
