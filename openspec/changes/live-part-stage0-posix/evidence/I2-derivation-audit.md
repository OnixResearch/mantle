Task-ID: I2
Covers: bootstrap.part.stage0.posix

# I2: `bootstrap/stage0-posix.ncl` upstream audit

Command/context:

```sh
sed -n '1,260p' bootstrap/stage0-posix.ncl
sed -n '12,158p' ~/git/pi-repos/fosslinux--live-bootstrap/parts.rst
find ~/git/pi-repos/fosslinux--live-bootstrap/seed -maxdepth 1 -type f -print
```

Audit result:

- Source scope matches the upstream early live-bootstrap boundary: `stage0-posix-amd64`, `M2-Planet`, `M2libc`, `mescc-tools`, `mescc-tools-extra`, `M2-Mesoplanet`, and `bootstrap-seeds`.
- Every repository source is pinned at the first consumer with `url`, `rev`, `hash`, and `name` fields in `bootstrap/stage0-posix.ncl`.
- Crunch intentionally uses the POSIX seed path rather than upstream kernel/Fiwix bootstrap startup. That is valid for this derivation because the output contract is the stage0-posix tool set, not a kernel boot proof.
- Crunch intentionally materializes `amd64.answers` and `after.kaem` inline. These are generated/control artifacts consumed only by this derivation and are part of the stage0-posix build transcript.
- Crunch intentionally normalizes install layout into `$out/bin` and `$out/lib/M2libc` so downstream Nickel derivations can consume a stable store contract instead of upstream in-tree `AMD64/bin` paths.
- The shell-side helper commands are limited to `/bin/busybox` for staging sources, chmod, directory creation, copying outputs, and assertions. The actual compiler/tool bootstrap still runs through the seed binaries and kaem chain.

Status: complete. No code change was needed for this task; the derivation is self-contained enough for the source-pin and build verification tasks to exercise it directly.
