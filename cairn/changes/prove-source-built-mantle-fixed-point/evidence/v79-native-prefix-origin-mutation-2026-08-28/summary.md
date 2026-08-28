# V79 native-prefix origin mutation blocker

## Outcome

V79 stopped before Rust-provider construction. The imported V61 native provider no longer matched its independently authenticated output BLAKE3.

Expected provider BLAKE3:

`63d9bc23cfcc232726527c141132ea35bc7ef9adf4ed7a952b5baa62bd466ed9`

Observed provider BLAKE3:

`a5c6a9749a93dcea577784a8eace2eda6e0f7a4cf1889140ec9255d10336a118`

The fail-closed admission rejected the provider. No Rust stage ran and no checkpoint was published.

## Root cause

Native-prefix reuse returned the imported provider path directly to downstream Rust construction. The Rust bootstrap made six musl inputs owner-writable:

- `Scrt1.o`
- `crt1.o`
- `crti.o`
- `crtn.o`
- `libc.a`
- `rcrt1.o`

A later failed-root cleanup could not treat that imported prefix as immutable authority. The six mode changes were enough to change the provider-tree identity. `v79-provider-writable-files.txt` records the exact bounded set.

## Repair

Native-prefix reuse now performs these operations in order:

1. Revalidate the imported origin against the expected provider and source-closure identities.
2. Copy the provider through the bounded no-follow tree copier into the current staging root.
3. Verify that the copied tree has the same provider identity.
4. Revalidate the copied provider and bind downstream execution to that copy.
5. Rehash the origin and fail if materialization changed it.
6. Write an explicit materialization report with origin and copied validation paths.

The positive test proves identity parity and different inodes. It then changes the copied payload and proves that the origin bytes and mode do not change. The negative test proves that a nonempty destination fails closed without changing either tree.

ADR 0084 records this boundary.

## Derived V61 repair

The historical V61 origin was not rewritten. A compact derived prefix was created by ordinary byte copying. Only the exact six writable musl files had owner-write removed. Full provider admission then restored the previously admitted BLAKE3 `63d9bc23…66ed9` and the source-closure BLAKE3 `7a97f836…28778`.

The derived prefix is:

`/home/brittonr/mantle-runs/receipt-fix-v31/source-built-native-prefix-v61-repaired-20260828`

This derived prefix is eligible as V80 input after the code repair is committed. It is not a provider reconstruction or new authority claim.

## Validation

The pre-change fixed-point shell suite passed 30 tests, with one expected ignored test.

After the repair, the combined fixed-point shell suite passed 36 tests, with one expected ignored test. `cargo check -p mantle --bin mantle`, edition-2024 rustfmt, and `git diff --check` passed.

## Non-claims

V79 does not prove Rust-provider construction, checkpoint publication, fixed-point equality, final receipt verification, or complete action trust. A fresh detached proof must establish those results.
