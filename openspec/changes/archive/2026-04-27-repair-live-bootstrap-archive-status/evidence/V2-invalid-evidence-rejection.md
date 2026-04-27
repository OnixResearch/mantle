Task-ID: V2
Covers: Full-source bootstrap claim requires evidence

# Invalid completion evidence audit

Status: complete for repair-slice audit. Full implementation remains deferred to
`live-bootstrap-source-chain`.

## Placeholder markers still present

Command:

```sh
rg 'ERROR: .*placeholder|TODO:.*placeholder' bootstrap --glob '*.ncl'
```

Result: matches exist, therefore full-source completion remains blocked.

```text
bootstrap/binutils-full.ncl:    echo "ERROR: binutils-full.ncl is a placeholder." >&2
bootstrap/gcc-4.7.ncl:    echo "ERROR: gcc-4.7.ncl is a placeholder." >&2
bootstrap/gcc-4.0.ncl:    echo "ERROR: gcc-4.0.ncl is a placeholder." >&2
bootstrap/gcc-10.ncl:    echo "ERROR: gcc-10.ncl is a placeholder." >&2
bootstrap/musl-full.ncl:    echo "ERROR: musl-full.ncl is a placeholder." >&2
bootstrap/binutils-tcc.ncl:      echo "ERROR: binutils-tcc.ncl is a placeholder." >&2
bootstrap/seed-full.ncl:# TODO: once the GCC ladder placeholder derivations are implemented,
```

## Archived partial scaffolding

The archived changes remain historical context only:

- `openspec/changes/archive/2026-04-26-live-bootstrap-seed-chain/`
- `openspec/changes/archive/2026-04-26-live-bootstrap-intermediate-tools/`

Their deferred validation lines do not satisfy
`bootstrap.fullsource.claim.evidence`. The active successor
`live-bootstrap-source-chain` owns implementation and proof tasks.

## Spec rule

`openspec/changes/repair-live-bootstrap-archive-status/specs/bootstrap/spec.md`
requires that prerequisite-only checks, placeholder derivations, deferred tasks,
archived partial-scaffolding changes, and unfinished successor tasks MUST NOT
count as full-source bootstrap evidence.
