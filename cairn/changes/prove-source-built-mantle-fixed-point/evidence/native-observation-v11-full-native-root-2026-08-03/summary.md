# Native observation v11: full native root

## Result

The GCC pass4 relocation repair moved the preserved strict offline state through the complete native toolchain root.

Pueue task `7504` rebuilt the conventional compiler chain and published the final GCC output:

```text
/mantle/store/8prxdlzabmg165kqq1xhplvahhwv7rzl-gcc-10.5.0-musl-final-v1
```

Pueue task `7520` then built `bootstrap/seed-full-toolchain.ncl`.
It published:

```text
/mantle/store/ig0l0rv4gvvma3f3s64ngpnbf4ll0h4v-binutils-2.41-gcc10-v1
/mantle/store/ly1hax2fpd8mb8xbn7y7yqcf88b92lg8-full-source-seed-toolchain
```

The final provider metadata records no state-pinned inputs, no legacy members, and no release-generated sources.
It records passed builder runtime checks for C, C++, assembly, archive, object inspection, and malformed-input rejection.

## Validation

Both diagnostic reports record strict hermeticity, no hermeticity audit events, one built root, no cache hit, and no root failure.
The full-source provider root also records its ordinary bounded non-claims.

## Non-claim

This observation proves one bounded full native toolchain build in the preserved strict offline state.
It does not prove the Rust provider, stage1 Mantle, stage2 Mantle, fixed-point equality, compiler correctness, or release eligibility.
