# Tasks: Use GNU ftpmirror front door for bootstrap sources

## Phase 1: URL rewrite

- [x] Rewrite GNU bootstrap tarball URLs in `bootstrap/binutils.ncl`, `bootstrap/dash.ncl`, `bootstrap/gcc.ncl`, and `bootstrap/make.ncl` to `https://ftpmirror.gnu.org/...`
- [x] Keep existing source names and hashes unchanged
- [x] Keep this work documented separately from `tighten-self-build-proof-hermeticity`

## Phase 2: Validation

- [x] Run `curl -I -L --max-time 20 https://ftpmirror.gnu.org/mpc/mpc-1.2.1.tar.gz`
- [x] Run `CRUNCH_SELF_HOSTING_LATER_STAGE_HERMETICITY_MODE=strict SNIX_BUILD_SANDBOX_SHELL=target/proof-busybox-static/bin/busybox ./scripts/prove-self-hosting.sh --bundle-dir target/self-hosting-proof/strict-hermeticity-check`
- [x] Run `openspec validate use-gnu-ftpmirror-for-bootstrap-sources`
