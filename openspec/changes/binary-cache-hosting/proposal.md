## Why

`crunch store push` and `crunch store pull` work against HTTP binary caches, but there's no story for hosting your own cache. For personal use across machines, you need a way to push build outputs somewhere and pull them from another machine. The current substituter code already speaks the narinfo/NAR protocol — what's missing is a simple server mode and guidance for hosting.

## What Changes

- **`crunch store serve`**: A built-in HTTP server that serves the local store as a binary cache. Implements the narinfo + NAR GET protocol that `crunch store pull` and the substituter already speak. Minimal — no authentication, no multi-tenant, just serve PathInfo and NARs from a local store directory.
- **Signing on push**: `crunch store push` should sign outputs before uploading if a signing key is configured. The signing infrastructure already exists (`crunch store sign`).
- **S3-compatible backend**: An optional `crunch store push --to s3://bucket/prefix` path for hosting on any S3-compatible storage (MinIO, R2, Backblaze B2, etc.). The upload format is the same narinfo + NAR files, just written to object storage instead of served directly.
- **Cache configuration**: `crunch-project.ncl` or a global config file gains `substituters` and `trusted-public-keys` fields so projects can declare where to pull pre-built outputs from.

## Capabilities

### New Capabilities
- `store-serve`: Serve local store as HTTP binary cache
- `store-push-s3`: Push signed outputs to S3-compatible object storage
- `cache-config`: Project-level substituter and trust configuration

### Modified Capabilities
- `store-push`: Adds automatic signing before upload
- `store-pull`: Already works — no changes needed
- `project`: Manifest gains cache configuration fields

## Impact

- **Files**: New `src/store_serve.rs` or extend `src/store_cmd.rs`, S3 upload module, config schema changes
- **APIs**: New CLI subcommands
- **Dependencies**: `hyper` or `axum` for HTTP server, `rust-s3` or `aws-sdk-s3` for object storage (optional feature)
- **Testing**: Integration test: serve a store, pull from it in a fresh store, verify PathInfo matches
