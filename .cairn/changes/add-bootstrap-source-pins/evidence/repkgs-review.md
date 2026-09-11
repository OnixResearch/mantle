# Evidence: repkgs source review

## Source

- Repository: https://github.com/mic92/repkgs
- Reviewed revision: `1cd7b8b` (2026-09-10), local clone at
  `/tmp/repkgs-review` during the 2026-09-10 session.

## Mechanism mapped by this change

- `docs/uptrack.md`: the `sources.toml` record (purl identity, URL templates,
  machine-written `[pin]` and hash), the discover→resolve→decide→apply→verify
  pipeline with a pure decide stage and a plan JSON interface, batched
  etag-cached resolution with per-host token buckets, and per-package
  `update.nu` hooks as the declared exception.
- `pkgs/up/uptrack/src/`: about 700 lines implementing purl, version
  comparison, datasources, cached http, locks, and the CLI.
- Design lesson: "machine-written state lives in a data file next to the
  package; never edit Nix source."

## Adaptation boundary

Mantle's catalog-package updates stay with the accepted
`mantlepkgs-update-plans` family; this change covers only `bootstrap/`
source pins. Data format, commands, and policy are Mantle-owned; nothing is
copied from repkgs.
