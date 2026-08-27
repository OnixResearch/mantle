# V74 rustc proxy mode and target linker

## Result

V74 failed before stage-2 standard-library compilation. The proxy source rewrite removed execute permission, so Cargo could not run `rustc_proxy.sh -vV`.

Reconciliation recorded 67,360 observed events, 67,356 matches, four denials, and 109 promotions. Its BLAKE3 is `9def5e81e7f5f3490d8755b37985d70083ac332123d08238cbe35ff8cdc66aba`.

## Cause

The V73 proxy rewrite used a temporary regular file and copied it over the executable. The replacement did not restore execute permission.

The rewrite also added the explicit linker only to the bootstrap branch. The target branch still permitted rustc to probe two nonexistent sysroot `cc` paths twice before using the protected linker wrapper.

## Repair

The proxy rewrite now:

- rewrites both `PROXY_RUSTC` and `PROXY_MRUSTC` calls with the explicit protected linker;
- restores execute permission after publication;
- accepts already-protected target and bootstrap forms;
- rejects missing or unknown forms before execution.

The final smoke artifact execution remains absolute from the V73 repair.

## Validation

The current validation transcript is `post-repair-validation.log`.

- Rust-provider tests: 73 passed, zero failed.
- `cargo check --bin mantle`: passed.
- Edition-2024 rustfmt check: passed.
- `git diff --check`: passed.
