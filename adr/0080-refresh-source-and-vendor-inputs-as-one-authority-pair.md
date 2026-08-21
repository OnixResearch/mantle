# ADR 0080: Refresh source and vendor inputs as one authority pair

## Status

Accepted (2026-08-21)

## Context

The V30 source-built proof used a refreshed Mantle source record and an older
vendor record. The source record contained a `Cargo.lock` with two pinned
bounded-tree Git packages. The vendor record did not contain those packages.
Its Cargo source config also lacked the new source replacement.

V30 completed StageX, the native provider, all host tools, and the complete
Rust provider. Cargo-free stage1 then rejected the missing Git manifests. The
failure was correct, but it occurred after almost all long proof work.

The existing `refresh-mantle-source` command replaced only the Mantle source
record. That design allowed one profile to bind a new lockfile and old vendor
material as separate, individually valid records.

## Decision Drivers

- Validate the complete Cargo source closure before long proof work.
- Keep the source profile free of ambient Cargo cache or network authority.
- Preserve exact package and file checksums from Cargo vendoring.
- Prevent a new lockfile from using an older vendor record.
- Keep unrelated native and StageX source records unchanged.

## Decision

`refresh-mantle-source` treats the Mantle source and checked vendor records as
one refresh authority pair. It derives `vendor-deps/` from the selected Mantle
source root. Before profile construction, it validates:

- `Cargo.lock` package membership;
- `.cargo/vendor-config.toml` and its source-tree directory target;
- every expected vendored package;
- every Cargo package checksum when one exists;
- every file named by each `.cargo-checksum.json` manifest;
- the absence of extra locked or vendored packages.

A successful refresh replaces exactly one Mantle source record and one vendor
record. It preserves every other classified record. It can still add only
materialized, unclassified fetch-source records.

The refresh report uses `mantle-source-built-profile-refresh-v2`. It binds the
old and new BLAKE3 values for both refreshed records.

The source-built proof repeats the same lock and vendor validation immediately
after materialization. This check runs before StageX. Profile construction and
proof execution do not use Cargo, the network, or ambient Cargo caches for this
validation.

## Alternatives Considered

### Preserve the vendor record during source refresh

Rejected. A source change can alter `Cargo.lock` or Cargo source config. The
result can remain structurally valid while its build closure is incomplete.

### Refresh vendor inputs only when the lockfile digest changes

Rejected. Cargo source config and vendored file checksums can change without a
lockfile change. Conditional replacement adds another parity rule.

### Let Cargo-free stage1 report missing packages

Rejected. The planner must still fail closed, but stage1 is too late for this
cheap profile check.

## Consequences

- Source-profile refresh scans the checked vendor tree and takes more time.
- A stale vendor tree fails before a new profile is written.
- A stale transferred profile fails before StageX starts.
- Operators must run locked Cargo vendoring after sourced dependency changes.
- Existing v1 refresh reports remain historical evidence. New reports use v2.
- This decision does not admit new dependency revisions, network fetches,
  provider outputs, or fallback behavior.
