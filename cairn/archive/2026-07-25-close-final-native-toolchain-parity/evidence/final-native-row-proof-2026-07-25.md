# Final native toolchain parity evidence

## Scope and decision

This change closes only the bounded `gcc.4.7`, `gcc.10`, and
`full-musl-binutils` parity rows. Each row now requires its own current source
set, immediate predecessor attestations, generated-artifact set, signed
no-substitution acceptance record, output attestation, positive runtime matrix,
negative rejection matrix, and explicit non-claims.

The previous provider-contract receipts remain useful historical diagnostics,
but they no longer promote these rows. A broad source-root provider, a later
compiler output, or another row's receipt cannot substitute for a row-local
receipt.

## Strict construction evidence

All accepted builds used `CRUNCH_NO_FUSE=1`, the committed derivations, the
strict state under `.pi/cairn-drain/early-native-strict`, `--no-substitute`, and
its selected signing key. The committed acceptance records require
`trust_unsigned=false`, `signing_key_selected=true`, and
`substitutions_enabled=false`.

| Stage | Accepted logical output | Canonical attestation BLAKE3 | NAR SHA-256 | Retained build log |
|---|---|---|---|---|
| GCC 4.7.4 | `/mantle/store/fr5r3bhxiaysf48lkl4aj5jqjfgxph4z-gcc-4.7.4-musl-gcc40-v1` | `8e4e022920f4d2ad673de72705e62ebc202513a2949cc79510aa3c21b2eafa86` | `0156d94cfd7e1c39e3160491f623939a07dd628db8bc5f90f183c0d5432afb6a` | `state/logs/vv3h44ci9hnaqqnm1rnild1d74qxbm6n-gcc-4.7.4-musl-gcc40-v1.drv.log` |
| GCC 10.5.0 | `/mantle/store/ysw79f732xmv92y8n63g6pjq3vid8445-gcc-10.5.0-musl-gcc47-v2` | `ef3778e9d4099f65e52e3f76c21cda132559c9daa4700db0064a604877be5f3a` | `f8e334316c0e598ec7baabc9e601c68938a3dece37c83ec1791b4561e4fa5288` | `state/logs/nw47j8f65gx1r496dsb52il4fydfhjif-gcc-10.5.0-musl-gcc47-v2.drv.log` |
| musl 1.2.5 | `/mantle/store/m7k4i3gsq169qp9ca9ckbrz2kmqqk01d-musl-1.2.5-gcc10-v1` | `8e096e43126225d797dbb789338f380cc586d6e98af6497451fca5f5452139d9` | `536e16442ba0acdea67dae2f06c3c93dd4e475ed2760bee0296e67fd4642a96c` | `state/logs/ncnw5i8jl4xr9qjk64liax67b92j0r0m-musl-1.2.5-gcc10-v1.drv.log` |
| final GCC 10.5.0 cycle | `/mantle/store/nvia8gzpydjda31lvca1c2kmbn4s6z3c-gcc-10.5.0-musl-final-v1` | `260f7c2f4bf507e194f49562afcaf13e04fec0a0e1b2bc0583df86a0a44ee5d6` | `bb62ea1d91f0d79d5decf045d98ba42a460a447df250d496486ee0303a62dca1` | `state/logs/9hkk0hrr7svm2dbbn9vbywwncakkkpx6-gcc-10.5.0-musl-final-v1.drv.log` |
| binutils 2.41 | `/mantle/store/c15xv2d8hifachmldawfp66p93i725l9-binutils-2.41-gcc10-v1` | `c7a17c5307db3823a94f774d70668ccb08322044c17c18092cca618ebaf082b4` | `3c95ace4f3ddf20490fc75173eda57a0459c5115c835ecd853460ecabb68b338` | `state/logs/lmvizn7ch54abdycdbsbvwdxywbs5300-binutils-2.41-gcc10-v1.drv.log` |

The GCC 4.7 and GCC 10 logs end with their final-native C/C++ runtime,
relocation, and rejection success markers. The final musl log proves static,
dynamic, and copied-tree runtime use plus malformed-C rejection. The final GCC
cycle proves static/shared C++17, `libgcc_s`, relocation, and malformed-input
behavior. The binutils log proves assembler/linker/archive/object operations,
copied-tree execution, and bounded non-timeout rejection of malformed assembly,
object, archive, and undefined-symbol link inputs.

The first binutils 2.41 fetch attempt failed before accepting source bytes when
`ftpmirror.gnu.org` selected an expired-certificate mirror. The derivation now
uses `mirrors.kernel.org` with the unchanged authenticated SHA-256. No artifact
from the failed attempt is part of the receipts.

## Committed receipt boundaries

- `bootstrap/evidence/final-native-gcc47-row-v1.json`
- `bootstrap/evidence/final-native-gcc10-row-v1.json`
- `bootstrap/evidence/final-native-musl-binutils-row-v1.json`
- matching `*-acceptance-v1.json` files
- matching portable output artifact envelopes
- portable immediate-predecessor envelopes under
  `bootstrap/evidence/final-native-predecessors/`

The shell independently reads and canonicalizes each attestation, recomputes
current source BLAKE3 values, checks artifact NAR facts, compares acceptance
trust/fallback facts, rejects path traversal, and scans declared derivations for
host, ambient-state, non-admission, and substitution markers before the pure
validator can accept a row.

## Verification evidence

- Pueue task `163`: `nix develop -c cargo test -p mantle --bin mantle bootstrap_parity -- --nocapture` — `88 passed; 0 failed`.
- Pueue task `166`: `nix develop -c cargo test -p mantle --test bootstrap_parity_cli -- --nocapture` — `19 passed; 0 failed`.
- The CLI suite includes final-row positive coverage and negative coverage for
  cross-row receipt substitution, state-pinned source text, missing runtime
  members, malformed/stale receipts, stale generated artifacts, corrupt
  attestations, missing output evidence, host discovery, predecessor wrapper
  delegation, digest mismatch, and unsigned acceptance.
- Pueue task `167`: `nix develop -c cargo test -p mantle --test bootstrap_eval -- --nocapture` — `30 passed; 0 failed`.
- Pueue task `165`: exact Cargo-script execution of
  `scripts/check-bootstrap-source-pins.rs` over all 13 receipt-bound source files
  — `13 files, 10 fetch blocks, 0 issues`.
- Pueue task `168`: `nix develop -c cargo fmt --check -p mantle -v` passed.
- Pueue task `169`: focused first-party Clippy with `-D warnings` passed; the
  compile emitted only the known vendored `snix-castore::Error::Unimplemented`
  dead-code warning outside the first-party lint scope.
- Pueue task `170`: `git diff --check` passed.
- Pueue task `171`: the current production parity report and
  `--require live-bootstrap` both passed. It reports `gcc.4.7`, `gcc.10`, and
  `full-musl-binutils` complete with `provider_kind=source-root`, and the
  `live-bootstrap` axis has no blocking rows.
- Pueue task `172`: Cairn validation returned `valid: true`.
- Pueue tasks `173`, `174`, and `175`: proposal, design, and tasks gates each
  returned `valid: true` and `verdict: "PASS"`; the tasks gate observed all
  eight tasks complete.

## Bounded claims and remaining blockers

This evidence establishes only the declared row-local derivational, artifact,
runtime, relocation, rejection, trust, and fallback facts. It does not establish
general compiler/libc/binutils correctness, provider or seed admission, or
whole-bootstrap correctness.

The production report still keeps `guix` blocked on `crunch.self-build`, and
keeps `stagex` blocked on `seed-full.stagex-lineage` plus
`crunch.self-build`. Closing these final-native rows does not satisfy those
separate proof authorities.

## Post-archive validation

Pueue task `180` reran
`nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- validate --root .`
after archive. The exact output is committed as
`evidence/post-archive-validation.txt`; it reports no issues or findings and
`valid: true` with four active changes remaining.
