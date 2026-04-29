Task-ID: V4
Covers: bootstrap.part.tinycc.0.9.26

# Host-tool/path leakage scan for tinycc 0.9.26

Command:

```sh
grep -nE '/usr|/run/current|/run/wrappers|/nix/store|/bin/(bash|dash|gcc|cc|make|tar|cp|sed|awk|grep)' bootstrap/tinycc-mes.ncl
grep -nE 'builder = "/bin/sh"|BB=/bin/busybox' bootstrap/tinycc-mes.ncl
grep -n 'hermeticity: practical (no degraded facts)' openspec/changes/archive/2026-04-29-fix-tinycc-mes-bufferedfile-codegen/evidence/V2-full-build.md
```

Result: PASS.

Forbidden host prefix scan:

```text
none
```

Allowed sandbox shell/busybox bindings:

```text
33:  builder = "/bin/sh",
39:      BB=/bin/busybox
```

Archived successful build transcript hermeticity marker:

```text
56:hermeticity: practical (no degraded facts)
```

Interpretation: `bootstrap/tinycc-mes.ncl` uses only sandbox `/bin/sh` and `/bin/busybox`; it has no hardcoded host `/usr`, `/run`, `/nix/store`, or host tool path. The successful build transcript reports no degraded hermeticity facts.
