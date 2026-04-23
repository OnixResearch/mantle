## Context

`crunch system` now has a native module loader, evaluator thread, fragment
collector, and phase-1 `nixos` backend. The current end-to-end tests prove the
CLI contract and merged JSON structure, but they stop before any running
system exists.

That leaves a narrow but important blind spot: the crunch path can emit a
structurally valid `merged_config.data.output.nixos` tree that later fails when
NixOS evaluates or boots it. The sibling `onix-modules` repo already solves the
same verification problem with `pkgs.testers.runNixOSTest`: evaluate modules,
feed the resulting config fragments into NixOS VMs, and assert runtime
behavior. This change adopts that verification pattern as a crunch-only,
test-only bridge without claiming that crunch now performs full NixOS assembly
itself.

## Goals / Non-Goals

**Goals**
- Prove that checked-in system-config fixtures survive the full crunch
  eval/merge path and still boot as NixOS systems.
- Keep crunch's live JSON output as the single source of truth for VM tests.
- Cover both positive and negative runtime behavior for the example
  multi-machine inventory.
- Keep the VM suite discoverable as a heavyweight validation rail, not an
  always-on edit-time gate.

**Non-Goals**
- Replacing the phase-1 `system-config.json` backend with full closure
  assembly.
- Adding a new public CLI command for VM tests.
- Generalizing the harness to non-NixOS backends in this change.
- Replacing Rust-side evaluator and CLI integration tests.

## Decisions

### 1. Bridge from `crunch system eval --stop-after fragments`

**Choice:** the VM harness will run `crunch system eval <inventory>
--stop-after fragments` and consume the resulting JSON envelope, not the
phase-1 build output from `crunch system build`.

**Rationale:** the fragment boundary is the narrowest seam that still exercises
live crunch evaluation, dependency ordering, provider threading, merge logic,
and machine selection. It avoids bwrap/store/build requirements while still
proving that the produced `output.nixos` tree is bootable.

### 2. Use a test-only Nix bridge instead of expanding crunch runtime scope

**Choice:** the harness will use `pkgs.testers.runNixOSTest` plus a small Nix
adapter that extracts `machines.<name>.merged_config.data.output.nixos` from a
JSON file produced by crunch.

**Rationale:** NixOS VM testing already exists and is reliable. Rebuilding a
parallel Rust/QEMU harness now would add infrastructure work without improving
confidence. Keeping the bridge test-only avoids overclaiming that crunch can
already assemble or boot full NixOS systems by itself.

### 3. Reject bad machine entries before boot starts

**Choice:** the Nix bridge will fail fast if a selected machine entry is
missing, marked failed, lacks `merged_config`, or lacks `output.nixos`.

**Rationale:** these are harness/preflight errors, not VM-runtime failures.
Rejecting them before `runNixOSTest` starts keeps diagnostics precise and
prevents confusing QEMU-level failures when the real problem is missing crunch
output.

### 4. Start with the checked-in `examples/system-config/` fixture

**Choice:** the first VM suite will boot the existing two-machine example
inventory and modules, while allowing thin test-only node wrappers to add tools
such as `curl`.

**Rationale:** this keeps the VM rail aligned with the same source fixtures
already covered by `tests/system_cli.rs`. The wrapper may add observation tools,
but it must not restate service behavior by hand or bypass crunch's evaluated
fragments.

### 5. Cover one single-machine smoke path and one multi-machine network path

**Choice:** the initial suite will include:
- a single-machine `server1` smoke boot that checks `openssh` and `nginx`
  behavior from the example inventory; and
- a two-machine boot that proves `server2 -> server1:8080` succeeds while
  `server1 -> server2:8081` fails because `server2`'s example firewall only
  allows TCP port `80`.

**Rationale:** the single-machine path proves basic boot/service activation. The
multi-machine path proves that the crunch-produced example inventory survives
through VM networking and preserves a meaningful negative case, not just happy
paths.

### 6. Register VM smoke as a heavyweight flake rail

**Choice:** each VM scenario will become a named flake check, and the docs will
call out the suite as a heavyweight KVM-backed validation rail distinct from
ordinary Rust checks.

**Rationale:** VM tests are slower and have stricter host prerequisites than
ordinary first-party edits. They need to be easy to find without quietly making
`check-first-party-quality.sh` or ordinary edit-time loops depend on KVM.

## Data Flow

```text
examples/system-config/{inventory.ncl,modules/*.ncl}
  -> built crunch binary
  -> `crunch system eval --stop-after fragments`
  -> JSON envelope
  -> Nix bridge extracts `machines.<name>.merged_config.data.output.nixos`
  -> test-only node wrapper adds observation tools/settings
  -> `pkgs.testers.runNixOSTest`
  -> Python testScript assertions against booted VMs
```

## Verification Strategy

- **Bridge preflight**: reject missing machine entries, failed entries, and
  missing `output.nixos` before VM launch.
- **Single-machine smoke**: boot `server1`, wait for default target, check
  `openssh.service`, and verify nginx answers on port `8080`.
- **Multi-machine runtime**: boot both example machines, verify `server2`
  reaches `server1:8080`, and verify `server1` cannot reach `server2:8081`.
- **Heavy rail wiring**: register named flake checks and document the exact
  commands and host assumptions.

## Risks / Trade-offs

**The bridge depends on NixOS test infra.** That is acceptable because this is a
verification-only seam. The runtime claim for `crunch system` remains unchanged.

**The bridge may need test-only IFD or derivation-generated JSON.** That is an
acceptable trade-off for a heavyweight VM rail so long as the source of truth is
still the built crunch binary and the checked-in fixtures.

**The first suite is intentionally narrow.** It proves bootability and one
meaningful negative path, not exhaustive module coverage. More scenarios can be
added later once the bridge exists.
