Task-ID: V1
Covers: bootstrap.part.stage0.posix

# V1: Source-pin audit

Command:

```sh
cargo -Zscript scripts/check-bootstrap-source-pins.rs bootstrap/stage0-posix.ncl
```

Exit status: 0

Output:

```text
warning: `package.edition` is unspecified, defaulting to the latest edition (currently `2024`)
source-pin audit: 1 files, 7 fetch blocks, 0 issues
```

Provider selection: not applicable; source-pin audit only.
Fallback status/event marker: not applicable; source-pin audit only.
Placeholder rejection result: no placeholder source block found; audit passed all fetch blocks.
