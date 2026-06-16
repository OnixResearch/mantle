## Why

The native closure materializer now fails closed on missing files, but it currently treats any provider metadata file as enough identity for both host and target roots. A target-only source-root provider can therefore reach file collection as a host root and report missing files instead of first rejecting the root capability mismatch.

## What Changes

- Parse provider metadata for host and target roots before collecting closure members.
- Require host roots to advertise host-compatible native-toolchain metadata for the Rust provider host triple.
- Require target roots to advertise either native-toolchain metadata or source-root metadata for the requested musl target.
- Add positive/negative tests and update the current frontier evidence to show the target-only source-root provider is not accepted as a host root.

## Impact

- **Files**: native closure materializer shell, source-built closure tests, Cairn spec/evidence.
- **Validation**: focused unit tests, formatting/whitespace checks, Cairn validate/gates, and a real frontier command transcript.
