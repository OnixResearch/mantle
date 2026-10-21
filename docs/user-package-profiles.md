# Explicit user package profiles

User profiles compose **already admitted exact outputs** into an explicit user-owned
symlink forest. They do not select versions, resolve package names, fetch missing
outputs, build derivations, configure an OS/service, modify shell startup files,
change global `PATH`, or remove store outputs. Build/evaluate packages through the
existing Mantle pipeline first. Selecting/removing a generation and collecting
store outputs are three separate operations.

## Historical reconciliation and scope

`openspec/changes/archive/2026-04-25-generation-profiles/proposal.md` is an archived
proposal, not an implementation or acceptance receipt. Its persistent manifest,
numbered symlink generations, rollback and GC requirements motivate this feature.
Its implicit default home-directory profile and suggested global-install wording
are **not** adopted: every command names an absolute `--profile` target. Shell
activation snippets and automatic environment edits are not needed to publish a
profile. Users may explicitly invoke `PROFILE/current/bin/tool` or choose their own
shell configuration.

The existing release-current pointer is still a separate feature. It selects an
immutable release-evidence identity; it does not compose a user environment or
retain package generations. User profiles reuse the synchronized immutable-object
then atomic-pointer publication pattern, store mutation lock, output authority and
owner-scoped GC roots, not a parallel package resolver or release lifecycle.

Publication is supported on local Linux filesystems with working advisory locks,
atomic same-filesystem rename, directory `fsync`, and Unix symlinks, for native
`x86_64-linux` and `aarch64-linux` packages. Windows, Darwin, network-filesystem
locking/durability, cross-platform execution and OS activation are not claimed.
The selected Snix or Casita backend remains authoritative. A physical `--store`
different from the logical prefix does not rewrite embedded runtime paths: a
profile does not prove package relocation, ABI compatibility or runtime closure
correctness. Packages must actually work at their admitted physical locations.

## Versioned exact manifest

`generations/N/manifest.json` has schema `mantle-user-profile-generation-v1`:

- `number`: positive monotonic generation number; removed/failed numbers are not
  reused. `parent` records the active generation when this generation was created.
- `manifest`: schema `mantle-user-profile-v1`, native `platform`, exact
  `store_prefix`, `conflict_policy: "merge-directories-reject-leaves-v1"`, and
  `packages` (at most 64 sorted local package slots).
- `links`: inspectable map of exposed relative leaves to their absolute physical
  admitted output leaves.

Each package is an exact request object containing `name` (local slot),
`derivation_path`, `derivation_aterm`, `output_name`, `output_path`, `nar_sha256`
(lowercase hex) and `nar_size`. A slot is not a registry selector or package
resolver expression. The ATerm field is the canonical
`nix_compat::Derivation::to_aterm_bytes()` **projection**: structural store paths
use that API's `/nix/store` serialization, while environment values retain their
actual bytes. Admission reconstructs the exact derivation identity using the
manifest's logical prefix. Do not paste a custom-prefix on-disk ATerm directly
into this field. The public `package_from_derivation` helper creates the request
from an evaluated derivation and observed PathInfo; it is not a trust bypass.

Admission checks that the derivation identity, named output (including an existing
CA realization mapping), recorded PathInfo producer, NAR hash/size and native
platform agree. It verifies a trusted PathInfo signature using the actual logical
prefix, requires complete admitted castore content, and observes the physical
filesystem NAR for **every reachable runtime reference**, rejecting
missing/untrusted/tampered members. A blob cache hit, an
unregistered directory, or a matching filename alone is insufficient. No unsigned
mode or ambient network/key/cache fallback exists. This uses Mantle's existing
producer metadata authority; it is not an independent proof of compiler or build
provenance correctness.

Only `bin`, `lib`, `libexec`, `share`, `include`, and `etc` are exposed. Real
directories merge recursively. Every leaf/leaf conflict (even identical content)
and leaf/directory conflict is an error; there is no implicit first/last package
winner. Output symlinks are leaves, not traversed directories. Special filesystem
objects, non-UTF8 paths, paths deeper than 64 components, or compositions exceeding
65,536 entries are rejected. No existing package/output bytes are changed.

## Commands and plans

The command contract uses the global `--store`, `--state-dir`, `--store-prefix`,
`--store-backend` and `--json` flags. The profile family takes:

```sh
mantle ... profile --profile "$PROFILE" \
  --expect-generation empty --trusted-public-key "$KEY" \
  install --package "$EXACT_PACKAGE_JSON"

mantle ... profile --profile "$PROFILE" \
  --expect-generation empty --trusted-public-key "$KEY" --apply \
  install --package "$EXACT_PACKAGE_JSON"

mantle ... profile --profile "$PROFILE" \
  --expect-generation 1 --trusted-public-key "$KEY" --apply \
  install --package "$UPGRADED_EXACT_PACKAGE_JSON" --replace

mantle ... profile --profile "$PROFILE" list
mantle ... profile --profile "$PROFILE" generations
mantle ... profile --profile "$PROFILE" --expect-generation 2 --apply remove tool
mantle ... profile --profile "$PROFILE" --expect-generation 3 \
  --trusted-public-key "$KEY" --apply rollback
mantle ... profile --profile "$PROFILE" --expect-generation 2 \
  --trusted-public-key "$KEY" --apply switch 1
mantle ... profile --profile "$PROFILE" --expect-generation 1 --apply remove-generation 2
```

`--profile` is mandatory, absolute, and must not traverse symlink ancestors. The
target must be separate from (not inside or containing) the physical output store
and must not equal the state-directory root. An unbound target must be empty;
existing unrelated content is never adopted. Every mutation requires
`--expect-generation empty|N`; this is a compare-and-swap, not
an instruction to silently rebase on another writer's changes. Omitting `--apply`
returns an inspectable complete plan with admitted candidate manifest, exposed
links, target generation and exact GC root owner label. Plans can create private
lock/directories and open existing store authority, but do not publish profile
roots, generation intent, generations, or `current`. Applying rechecks inputs
under both locks; a prior plan is not permission to bypass new admission checks.

Installing into an existing slot fails unless `--replace` is explicit; replacing
a missing slot also fails. Removing a package creates a new generation, including
an empty generation when the last package is removed. Switching/rolling back
never creates a generation or removes another one. Rollback chooses the greatest
retained generation number **less than** the active number, not the last pointer
activation and not a toggle. Removing the active generation is rejected. List and
generations describe stored state; `admission_checked: false` is not a fresh
trust/runtime verification claim. Switch/install plans re-admit actual outputs.

The JSON report schema is `mantle-user-profile-report-v1`. It distinguishes
`previous_active`, `active`, `target_generation`, `retained_generations`,
`pending_generations`, `applied`, `admission_checked`, candidate `manifest`,
`links`, and `root_owner_label`. Human mode may render the same structured report;
stdout must not pretend a plan was applied.

## Transactions, locks and recovery

A nonblocking existing `StoreMutationGuard` excludes builds/substitution/GC, then
a nonblocking per-profile advisory lock excludes another writer or inspector,
including one using another state directory. Contention is an error, not a wait,
merge, or overwrite. `binding.json` binds the canonical state/output directories,
logical prefix and platform; using another store with this profile is rejected.
Filesystems/profile directories must be owned and protected by the calling user;
these locks do not protect against an adversarial process with the same account
that deliberately edits files outside the protocol.

A new generation allocates and synchronizes its sequence number, stages and syncs
its complete manifest/forest, durably records `.pending-N.json`, registers existing
owner-scoped explicit-pin roots, publishes the synchronized generation directory,
and only then atomically renames a new relative `current -> generations/N`
symlink into place. Readers see a complete old or complete new generation, never
a partially populated environment. The previous generation and its roots are not
removed. Pre-commit errors leave `current` unchanged; interruption before pointer
rename leaves the old active usable, possibly with an extra retained or pending
generation. Interruption after rename sees either complete generation after
recovery. A post-rename directory-sync I/O failure attempts to atomically restore
and synchronize the previous pointer before reporting an error. If both the
commit sync and recovery fail, the error explicitly reports **commit durability
uncertain**: inspect `current` before retrying. The old generation remains
separately retained and usable even when the filesystem cannot confirm recovery.

Recovery is explicit and conservative:

1. Run `generations`/`list` with the same binding. Inspect `current`, the returned
   active generation and pending generation numbers. An unselected published
   generation is retained; select it only with a normal admitted `switch`.
2. If an interrupted new generation is listed in `pending_generations`, inspect
   its `.pending-N.json` and plan `remove-generation N`. Apply with the current
   expected active number to release only that generation's interests. No output
   admission is needed to release an inactive generation containing missing or
   damaged outputs.
3. Generation removal atomically renames it to `.removed-N` before releasing its
   exact owner roots. A repeat `remove-generation N` safely resumes interrupted
   root release using that tombstone. A pending-only generation gets an equivalent
   tombstone. Tombstones retain diagnostic metadata/links, are never selectable,
   and are not GC roots. They do not keep outputs alive.
4. Interrupted `.stage-*`/`.switch-*`/`.removing-*` directories are never selected, and are not
   automatically removed by later commands. Do not point `current` at them.
5. Root release never removes another generation's/operator's interests. Active
   and all retained generations have independent explicit pins, not age/lease
   expiration. Ordinary `store gc` still traces the existing root graph. To reclaim
   outputs, first explicitly remove inactive generations, then inspect and apply a
   separate existing store-GC plan. No profile command performs destructive GC.

A corrupt metadata file, malformed/broken `current`, unexpected generation entry,
wrong binding, or tampered retained forest fails closed. Preserve the directory
for inspection; use a new explicit target if manual repair is necessary. Do not
silently reset state or import cached bytes as a recovery shortcut.

## Verification scope

The source includes isolated fixtures for both backend lifecycles and GC,
install/explicit upgrade/remove/rollback/switch, real executable selection,
content/signature/platform/conflict rejection, lock contention and simultaneous
compare-and-swap writers, retained-forest tampering, interrupted pending-root
recovery and publication-before-selection. Run the integrated checks and actual
CLI scenario on the final immutable source before treating these as observed
acceptance evidence. Fixture presence alone is not evidence that a check passed.
