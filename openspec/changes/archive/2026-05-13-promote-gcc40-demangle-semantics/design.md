## Context

The previous increment replaced a generic cp-demangle stub marker with a checked disabled-demangle boundary. The actual `cplus_demangle`/`cplus_demangle_v3` entrypoints still returned null.

## Decision

Implement only a bounded Itanium zero-argument function slice: `_Z3foov` becomes `foo()` and malformed or unsupported names return null. This is useful semantic progress, easy to smoke, and avoids overclaiming full libiberty demangler correctness.

## Non-Goals

- Full C++ ABI demangling.
- Native GCC cp-demangle correctness.
- Changing `gcc.4.0` parity classification away from partial.

## Validation

The derivation must compile and run a small libiberty smoke that checks a positive `_Z3foov` case and negative malformed inputs. Parity tests must require the bounded marker and reject the previous disabled marker.
