# Relevant Nix checks

## Bounded blockers

- Pueue task 11228 ran `nix flake check -L`. The unrelated `durable-file-publication-adoption` check rejected stale BLAKE3 bindings for the current manifest and flake files. Active Cairn changes own that broad-validation repair.
- Pueue task 11342 included the repository Tiger Style check. Existing findings in `crunch-gc-core` and `crunch-overlay-core` rejected that broad check.
- Pueue task 11365 ran the focused format and Mantle package checks. Formatting completed, but the Mantle package source filter omitted existing compile-time fixtures `fixtures/content-bound-requirements/mantle-registry.json` and `mantle-requirement-ref.json`. The package build stopped at those two missing files after it compiled `crunch-nar` and its dependencies.

These blockers do not provide NAR-boundary success evidence. The focused Nix formatting check below covers the changed source format.

## Focused Nix formatting check

```text
$ nix build .#checks.x86_64-linux.fmt -L
```

Focused Nix formatting verdict: PASS
