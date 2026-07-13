# Stateful tool workspaces

Mantle supports explicit retained-tool workspace modes without treating mutable
history as a declared build input.

## Modes

- `none` keeps the existing ephemeral sandbox behavior.
- `immutable-snapshot` mounts one declared snapshot input read-only at a stable
  guest path. The snapshot reference participates in derivation identity.
- `mutable-session` exclusively leases a private worker-local directory and
  mounts it writable at a stable guest path. Its outputs are admitted normally,
  but the execution cannot publish or satisfy a strong shared action result.

Mantle never silently changes modes. `fallback = 'fail` is the default;
`fallback = 'to-none` is an explicit, reported downgrade.

## Nickel policy

```nickel
let Derivation = import "derivation.ncl" in
({
  name = "cargo-check",
  builder = "/bin/sh",
  workspace = {
    mode = 'mutable-session,
    workspace_id = "cargo-cache",
    guest_path = "/build/.mantle-workspace",
    compatibility = {
      authority_class = "tenant-a",
      action_class = "cargo-check",
      toolchain_refs = ["mantle-object://blake3/<toolchain>"],
    },
    quota = {
      bytes_max = 4294967296,
      files_max = 100000,
      snapshots_max = 8,
    },
    retention = {
      workspace_count_max = 64,
      idle_generations_max = 16,
      age_generations_max = 256,
      quarantine_count_max = 16,
    },
    snapshot.enabled = true,
    clean_rebuild.enabled = true,
  },
} | Derivation)
```

The closed contract also types scrub limits, sensitive paths, secret markers,
snapshot cleanliness, and clean-rebuild input requirements. Rust revalidates
all mode-specific invariants and arithmetic bounds before execution.

## Isolation and lifecycle

Mutable directories live below the configured state directory, not in action
identity or reports. Mantle validates contained component names, rejects
symlinked state roots, opens scanned files with no-follow semantics, and uses an
exclusive lock plus worker/job/attempt/fence lease. Failed scans, scrub,
cleanup, quota enforcement, or abandoned execution quarantine the directory.
Quarantined state is never reused.

Retention runs before admission. Active leases are preserved; old idle and
bounded quarantine entries are evicted deterministically. Remote coordinators
persist endpoint workspace registrations and fenced lease records atomically,
so restart does not erase current ownership.

## Evidence and claims

`crunch-build-report-v1.workspace_reports` records the actual mode, stable guest
path, compatibility digest, warm-state use, cleanup disposition, snapshot
reference, claim class, and optional clean/warm output-set digests. It never
contains the worker host path or retained file contents.

A clean comparison is a separate execution with no mutable workspace. Matching
output-set digests add comparison evidence only. They do not relabel the
original warm execution as hermetic and do not enable strong publication or
reuse for that original execution.
