# Current native source closure v8

## Result

The current `bootstrap/seed-full-toolchain.ncl` source closure contains 47 records and `285,740,854` payload bytes.
Its manifest BLAKE3 is:

```text
0d5e81a6741391c9e3acb4329b3290e777eac2a3fcf9014c357771e3dded05d2
```

The bundle path is:

```text
/home/brittonr/.cargo-target/mantle-full-source-closure-v8-current-native-20260801.json
```

## Construction

Pueue task `7893` imported and pinned closure v7 into a fresh source state.
Pueue task `7895` evaluated the current native root and exported its exact closure with explicit missing-source acquisition.
The fetcher fixed-output-validated the current `patch-2.5.9-src` record after one transient HTTP 502 retry.

Pueue task `7900` independently verified the finished bundle.
It reported:

```text
readiness=Ready missing=0 stale=0 unsupported=0 untrusted=0
```

## Non-claim

The closure proves source availability and identity for the evaluated native root.
It does not prove a build, provider admission, compiler correctness, or release eligibility.
