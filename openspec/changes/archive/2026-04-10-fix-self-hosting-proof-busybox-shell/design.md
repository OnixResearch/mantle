## Context

The failure was not in the source-binding proof logic itself. The proof helper
left `SNIX_BUILD_SANDBOX_SHELL` at `/bin/sh`, and on this host `/bin/sh` is a
DYN-linked bash from the fallback path. `snix-build` mounts that path into the
sandbox as `/bin/sh`, but it does not mount the host glibc closure, so the
first sandboxed bootstrap steps can die with:

```text
bwrap: execvp /bin/sh: No such file or directory
```

That made `./scripts/prove-self-hosting.sh --check` misleading: host tools were
present, but the actual proof still lacked a usable static shell.

## Goals / Non-Goals

**Goals**
- Make the proof helper pick a real static shell before launching the proof
- Keep the common path automatic on NixOS hosts
- Preserve an explicit user override when they already provide a non-placeholder
  `SNIX_BUILD_SANDBOX_SHELL`

**Non-Goals**
- Change the runtime shell-selection logic inside `vendor/snix-build`
- Add a second proof runner
- Guarantee proof speed; only guarantee that it starts with a usable shell

## Decisions

### 1. Treat `/bin/sh` as a placeholder in the proof helper

**Choice:** if `SNIX_BUILD_SANDBOX_SHELL` is unset or `/bin/sh`, the helper now
searches for a static shell instead of exporting `/bin/sh` directly.

**Rationale:** `/bin/sh` is executable on the host but unusable inside the
sandbox on this machine.

### 2. Prefer already-installed static outputs first

**Choice:** scan common locations first:
- `/run/current-system/sw/bin/busybox-static`
- `/bin/busybox.static`
- `/nix/store/*-busybox-static-*/bin/busybox`
- `/nix/store/*-bash-static-*/bin/bash`

**Rationale:** this keeps the happy path cheap and deterministic when the host
already has a suitable static shell.

### 3. Realize `pkgsStatic.busybox` as a fallback

**Choice:** if no static shell is already present, call
`nix-build '<nixpkgs>' -A pkgsStatic.busybox --no-out-link` and use the
resulting `bin/busybox`.

**Rationale:** on this host the proof helper already assumes a NixOS-like
environment for tool discovery. Realizing a small cached static busybox is far
better than starting a 30+ minute proof that will fail in stage0.

## Risks / Trade-offs

**[helper side effect]** `--check` can now realize `pkgsStatic.busybox` if it is
missing.
Mitigation: that fetch is small and directly tied to the proof prerequisite the
helper is validating.

**[host assumption]** the fallback uses `nix-build`.
Mitigation: if `nix-build` is unavailable and no static shell is already
installed, the helper now fails early with an actionable error instead of a
late stage0 sandbox crash.
