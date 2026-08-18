## Context

`bootstrap native-toolchain-closure` now builds a claiming manifest only after required members exist and are hashed. It still needs a stronger shell boundary: root metadata should constrain whether a directory may satisfy host or target roles before member paths are inspected.

## Design

### Functional core

Add a pure capability classifier for native-root metadata. The classifier takes parsed metadata data plus the requested role (`host` or `target`) and expected triple. It returns either a bounded capability identity or a deterministic mismatch diagnostic.

The accepted metadata shapes are intentionally narrow:

- host roots: `mantle-native-toolchain` metadata whose advertised target/host triple matches the Rust provider host triple and whose capability includes `host-native`;
- target roots: either `mantle-native-toolchain` metadata for the requested target triple with target capability, or existing source-root metadata for `x86_64-linux-musl`.

### Imperative shell

The shell reads `provider.json`, parses the small fields needed for classification, computes the metadata BLAKE3 digest, and only then collects executable/sysroot/runtime candidates. It must return before file collection when a root is target-only or has no source-built capability.

### Claim boundary

Capability validation is not a source-built closure proof by itself. It only prevents the wrong provider kind from entering a manifest. The fixed-point proof with enforced observed inputs remains the only path that can retire `not-source-built-toolchain-closure`.
