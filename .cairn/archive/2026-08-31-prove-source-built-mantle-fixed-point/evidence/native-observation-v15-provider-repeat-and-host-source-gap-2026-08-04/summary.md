# Native observation v15: provider repeat and host-source gap

Task-ID: I3
Covers: bootstrap_inventory.source_built_mantle_fixed_point

## Result

Two fresh runs from profile v27 built the same native provider.
Both build reports recorded `built_total = 1`, `cached_total = 0`, and no failure.

The normalized provider BLAKE3 was:

```text
82b08dcd3ce1769d37888a0378b756bdda0529b2070fbe013e64b07225f49373
```

The provider metadata BLAKE3 was:

```text
57a0afb811c9bd05c676ece50b2985fe383e49369b57454b61593cfe7cb0ae15
```

The independent admission report recorded 53 source records, 1,173 entries,
18 required tools, 10 required runtime files, and all 25 smoke steps.
The second run admitted the same provider before it continued.

## Next blocker

The second run failed closed before it built the full-source Rust provider.
The Make 4.4.1 build required this missing fixed source:

```text
fixed-url-809778e55f2ea5bb42c28a2acbaff93b5a34e24b5d71420a3c570b5639946859
```

The durable attempt status recorded `network-required` under offline preflight.
No live fetch, substitution, cache completion, or fallback occurred.

## Repair

A connected producer exported all six Rust host-tool build roots.
The resulting source union added six materialized fetch records to profile v28.
This includes Make, Linux headers, BusyBox, CMake, Python, and Perl source needs.

## Evidence paths

- Profile v27: `/home/brittonr/.cargo-target/mantle-source-built-fixed-point-profile-v27-current-source-20260804.json`
- Observation A root: `/home/brittonr/.cargo-target/mantle-source-built-fixed-point-runs-v15a/`
- Independent admission: `/home/brittonr/.cargo-target/mantle-source-built-fixed-point-v15a-native-admission-20260804.json`
- Observation B root: `/home/brittonr/.cargo-target/mantle-source-built-fixed-point-runs-v15b/`
- Host-source union: `/home/brittonr/.cargo-target/mantle-rust-host-source-union-v1-20260804.json`

## Non-claim

This evidence proves two matching fresh native-provider observations for profile
v27 and the bounded admission checks. It does not prove the Rust provider,
either Mantle stage, fixed-point equality, compiler correctness, or release
eligibility.
