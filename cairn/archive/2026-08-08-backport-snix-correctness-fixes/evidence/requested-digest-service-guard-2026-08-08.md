# Requested-digest service guard

## Implemented boundary

`vendor/snix-store/src/pathinfoservice/nix_http.rs` now compares the requested fixed-width digest with the parsed narinfo store-path digest.

Both `get()` and `get_references()` reject a mismatch immediately after narinfo parsing and signature validation. The guard runs before the NAR request, NAR ingestion, or PathInfo return.

The stable error class is:

```text
narinfo store path digest does not match the requested digest
```

## Positive and negative coverage

The pure comparator test accepts an exact digest match and rejects a different digest with both values retained in the typed error.

Two HTTP-service tests serve narinfo for a different path. Both tests assert one narinfo request, zero NAR requests, and a mismatch error.

The existing matching-path tests continue to cover metadata-only reference lookup and ordinary full-get NAR fetch behavior.

## Validation

Command:

```text
SNIX_BUILD_SANDBOX_SHELL=/bin/sh nix develop -c cargo test -p snix-store --lib nix_http
```

Result:

```text
test result: ok. 18 passed; 0 failed; 0 ignored; 0 measured; 71 filtered out; finished in 0.04s
```

Command:

```text
SNIX_BUILD_SANDBOX_SHELL=/bin/sh nix develop -c cargo test -p snix-store --lib
```

Result:

```text
test result: ok. 89 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s
```

Strict package Clippy found two pre-existing errors in `pathinfoservice/redb.rs` and `rpc.rs`. Neither file changed in this increment.

The focused Clippy rerun used only these named waivers:

```text
-A clippy::redundant_closure -A clippy::large_enum_variant -D warnings
```

This rerun passed. Leaf rustfmt and `git diff --check` also passed.

## Remaining boundary

Mantle still needs the independent first-party guard before persistence and side effects. Signed-wrong-path and mutation-counting fixtures also remain incomplete.
