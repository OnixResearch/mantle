# Mantle naming rules

Mantle is the project-facing name in user-facing prose, docs, examples, help
text, status summaries, and new evidence. The legacy Crunch name remains only
when an exact identifier, compatibility surface, or historical artifact still
requires that spelling.

## Replace stale prose

Use Mantle for the product, build tool, operator workflows, proof summaries,
status output, and examples. New docs should not describe the project as Crunch
unless the same line also names a legacy or compatibility reason.

## Preserve exact identifiers

Do not rename these without a separate compatibility change:

- crate, package, module, or API identifiers such as `crunch-build`,
  `crunch-eval`, `crunch_store`, and `CrunchDerivation`;
- compatibility file and command surfaces such as `crunch.ncl`, `crunch.lock`,
  `.crunch/`, `crunch.fetchurl`, and `/crunch/store`;
- serialized report or receipt tokens such as `crunch-build-report-v1`,
  `not-crunch-bootstrap`, and `crunch.self-build`;
- historical or archived evidence, including pre-rename transcripts;
- embedded bootstrap evidence markers whose spelling is part of checked evidence,
  such as `CRUNCH bridge TinyCC builtin va_list`.

## Guard and fixtures

Run the naming guard after touching user-facing docs, examples, scripts, or
proof text:

```bash
nix develop -c sh -c 'rustc scripts/check-stale-branding.rs -o /tmp/mantle-check-stale-branding && /tmp/mantle-check-stale-branding'
nix develop -c sh -c 'rustc scripts/check-stale-branding.rs -o /tmp/mantle-check-stale-branding && /tmp/mantle-check-stale-branding --self-test'
```

The self-test proves positive allowed contexts for exact compatibility tokens
and negative rejection for stale legacy project prose in new docs.
