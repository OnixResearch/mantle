# V80 Rust bootstrap vendored checksum blocker

## Outcome

V80 proved the isolated native-prefix boundary and cleared the V78 execution denials. It then stopped while building stage2 Cargo with Rust 1.91.1.

The MRustC stage reconciled 88,038 observed executions with zero denials and 177 promotions. The Rust 1.91.1 stage reconciled 54,850 observed executions with zero denials and 109 promotions. Its only blocker was `stage-execution-failed`.

The OpenSSL `no-asm` rewrite ran. The prior ambient `/bin/sh` request and the eight missing `g++` probes did not recur.

## Exact blocker

The source rewrite changed:

`vendor/openssl-src-300.5.2+3.5.2/src/lib.rs`

Cargo later checked the vendored source against `.cargo-checksum.json` and rejected it:

- expected SHA-256: `d29d6f116de18db723e93d74b4387eb0a672c574c8435526c71acbf00ca368ec`
- observed SHA-256: `ba030b6661a9c6db0943955346c77f42cc76452826dabc6b75a256f6b4810404`

SHA-256 is required here because this is Cargo interoperability metadata. Mantle-owned identities remain BLAKE3.

`v80-checksum-error.txt` preserves the exact diagnostic.

## Native-prefix isolation evidence

V80 admitted the repaired origin and the bounded working copy with provider BLAKE3 `63d9bc23cfcc232726527c141132ea35bc7ef9adf4ed7a952b5baa62bd466ed9`.

After the failed Rust stage, both provider trees had zero owner-writable regular files. A new full admission of the repaired origin also returned `63d9bc23…66ed9`. This confirms that normal Rust execution did not change the origin or working-copy modes. It also narrows V79’s mode change to shared inodes affected by V78’s recursive cleanup.

## Repair

The OpenSSL compatibility adapter now updates the matching Cargo checksum entry before it publishes the patched source file.

The adapter:

- uses the receipt-bound Python executable;
- opens the original source, patched source, and checksum file without following symbolic links;
- enforces explicit source, metadata, read, and write bounds;
- requires the checksum file to contain the exact `files` and `package` shape;
- requires `files["src/lib.rs"]` to match the pre-patch source SHA-256;
- replaces exactly one compact JSON entry without changing metadata length;
- writes through a create-new temporary file and atomic rename;
- remains idempotent when the source and checksum already contain the patched identity;
- fails closed when checksum metadata is missing or malformed.

## Validation

The Rust-provider suite passed 83 tests. The patch-plan suite passed seven tests. The fixed-point shell suite passed 36 tests, with one expected ignored test. `cargo check -p mantle --bin mantle`, edition-2024 rustfmt, and `git diff --check` passed.

## Evidence

The directory contains the fixed-point plan and status, source transfer and profile receipts, native-prefix materialization reports, both stage plans, compressed raw audits, reconciliations, compressed logs, exact launch scripts, the checksum diagnostic, and the post-run origin admission.

## Non-claims

V80 does not prove later Rust stages, provider checkpoint publication, fixed-point equality, final receipt verification, or complete action trust. A fresh detached proof must establish those results.
