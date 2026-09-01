# Native observation v12: provider identity

## Result

Fresh proof v12 completed the StageX transition, published the StageX provider, and built the full native toolchain root.
It then failed closed because the command supplied the StageX normalized provider digest as the expected final native-provider digest.

The rejected comparison was:

```text
expected=e1039a3c844d709f51586f7afa2aacdbbe92aa224f1e20a778ea80573603ada3
observed=9f47f75ab2677afe2e789eeb8486da7abeffa20427130e3d7403bf9d200759dd
```

The first digest identifies the bounded StageX intermediate provider.
The second digest identifies the complete `full-source-seed-toolchain` tree.
These are different authority stages and must not share one expected identity.

## Independent admission

Pueue task `7555` reran the full-source provider admission command over the preserved v12 output.
It admitted BLAKE3 `9f47f75ab2677afe2e789eeb8486da7abeffa20427130e3d7403bf9d200759dd`.
The report records 53 source-closure records, 1,173 observed entries, 18 required tools, 10 required runtime files, and all 25 bounded smoke steps.

Fresh proof v13 now uses this independently checked native-provider identity.

## Non-claim

This observation proves the final native-provider identity and bounded admission rails for one fresh build.
It does not prove the Rust provider, either Mantle stage, fixed-point equality, compiler correctness, or release eligibility.
