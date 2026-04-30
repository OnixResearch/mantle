Task-ID: V4
Covers: bootstrap.part.tinycc.0.9.27.amd64.compile

# Host-leakage scan

Result: PASS.

Command:

```sh
grep -nE '/usr/bin|/bin/(cc|gcc|ld|ar)|nix-|nix |/nix/store/.+-(gcc|binutils|glibc)' \
  bootstrap/tinycc.ncl \
  openspec/changes/repair-tinycc-0-9-27-amd64-compile/evidence/V2-build-full.log
```

Result:

```text
grep_rc=1
host_leakage=none
```

Allowed sandbox references (`/bin/sh` builder and `/bin/busybox`) were not counted as host leakage.

Full transcript: `evidence/V4-host-leakage-full.log`.
