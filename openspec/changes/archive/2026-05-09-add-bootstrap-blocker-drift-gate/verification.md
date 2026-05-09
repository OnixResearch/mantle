# Verification

## Positive inventory evidence

Command:

```sh
PATH=/nix/store/97vplpbajnr7x03fqh9biz5v6960sv22-clang-wrapper-21.1.8/bin:/nix/store/1sw8whfl5gfblp6r9qdkiw1b4j9fgwar-mold-2.40.4/bin:$PATH \
  ./scripts/check-bootstrap-blocker-inventory.sh \
  --json openspec/changes/archive/2026-05-09-add-bootstrap-blocker-drift-gate/evidence/current-inventory.json \
  --markdown openspec/changes/archive/2026-05-09-add-bootstrap-blocker-drift-gate/evidence/current-inventory.md
```

Result: passed. The current gated tree produced a deterministic inventory report with all configured marker classes present and zero promotion claims.

Evidence:

- `evidence/current-inventory.json`
- `evidence/current-inventory.md`

## Negative promotion-drift evidence

Command:

```sh
fixture=$(mktemp -d)
printf 'full-source bootstrap status: promoted\n' > "$fixture/claim.md"
printf '# bridge output remains, so promotion must fail\n' > "$fixture/blocker.ncl"
~/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin/cargo -Zscript \
  scripts/check-bootstrap-blocker-inventory.rs \
  --enforce \
  --json evidence/negative-promotion-drift.json \
  --markdown evidence/negative-promotion-drift.md \
  "$fixture"
```

Result: failed as expected with exit code 1 and diagnostic `promotion claim conflicts with remaining bootstrap blockers`.

Evidence:

- `evidence/negative-promotion-drift.log`
- `evidence/negative-promotion-drift.json`
- `evidence/negative-promotion-drift.md`

## Syntax and OpenSpec validation

Commands:

```sh
bash -n scripts/check-bootstrap-blocker-inventory.sh
./scripts/check-bootstrap-blocker-inventory.sh \
  --json openspec/changes/archive/2026-05-09-add-bootstrap-blocker-drift-gate/evidence/current-inventory.json \
  --markdown openspec/changes/archive/2026-05-09-add-bootstrap-blocker-drift-gate/evidence/current-inventory.md
openspec validate add-bootstrap-blocker-drift-gate --strict --json > /tmp/openspec-add-bootstrap-blocker-drift-gate-final.json
openspec validate --all --strict --json > /tmp/openspec-all-add-bootstrap-blocker-drift-gate-final.json
git diff --check
```

Result: passed before archive.

## Post-archive validation

Commands:

```sh
openspec validate bootstrap --strict --json > /tmp/openspec-bootstrap-after-add-bootstrap-blocker-drift-gate.json
openspec validate --all --strict --json > /tmp/openspec-all-after-add-bootstrap-blocker-drift-gate.json
./scripts/check-bootstrap-blocker-inventory.sh \
  --json openspec/changes/archive/2026-05-09-add-bootstrap-blocker-drift-gate/evidence/current-inventory.json \
  --markdown openspec/changes/archive/2026-05-09-add-bootstrap-blocker-drift-gate/evidence/current-inventory.md
git diff --check
```

Result: passed after archive; canonical `bootstrap` spec and global strict validation remained valid.
