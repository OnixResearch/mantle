# Mantle naming rules

Mantle is the project-facing name in user-facing prose, docs, examples, help
text, status summaries, and new evidence. The legacy Crunch name remains only
when an exact identifier, compatibility surface, or historical artifact still
requires that spelling.

## Replace stale prose

Use Mantle for the product, build tool, operator workflows, proof summaries,
status output, and examples. New docs must use Mantle. Use the legacy product
name only when the same line states the exact compatibility or history reason.

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

## Compatibility transitions

The typed inventory in `config/operator-surfaces.ncl` records each supported
operation and removal gate. Do not change a compatibility state from prose
alone.

A transition to `compatibility-read-only` requires all of this evidence:

- the canonical replacement is accepted and documented;
- supported readers still pass positive and malformed-input fixtures;
- supported writers no longer emit the old form, or the inventory records why
  production must continue;
- a consumer inventory and a rollback procedure exist.

A transition to `historical-only` requires all of this evidence:

- no supported command reads, writes, or emits the identifier;
- repository and downstream consumer searches have bounded receipts;
- archived artifacts keep their original bytes and provenance;
- the accepted migration includes rollback and retention decisions.

If this evidence is incomplete, keep the current compatibility state. Do not
remove or silently reclassify the identifier.

## Guard and fixtures

Run the naming guard after touching user-facing docs, examples, scripts, or
proof text:

```bash
nix develop -c sh -c 'rustc scripts/check-stale-branding.rs -o /tmp/mantle-check-stale-branding && /tmp/mantle-check-stale-branding'
nix develop -c sh -c 'rustc scripts/check-stale-branding.rs -o /tmp/mantle-check-stale-branding && /tmp/mantle-check-stale-branding --self-test'
```

The self-test proves positive allowed contexts for exact compatibility tokens
and negative rejection for stale legacy project prose in new docs.
