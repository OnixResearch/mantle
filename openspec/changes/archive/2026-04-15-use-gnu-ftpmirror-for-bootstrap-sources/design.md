# Design: Use GNU ftpmirror front door for bootstrap sources

## Summary

Replace direct `https://ftp.gnu.org/gnu/...` URLs in checked-in bootstrap
GNU source fetchers with `https://ftpmirror.gnu.org/...` URLs.

This keeps the same filenames, logical source names, and pinned content hashes.
Only the front-door host changes.

## Approach

- update GNU-hosted tarball URLs in:
  - `bootstrap/binutils.ncl`
  - `bootstrap/dash.ncl`
  - `bootstrap/gcc.ncl`
  - `bootstrap/make.ncl`
- leave all `hash = ...` values unchanged
- leave all `name = ...` values unchanged
- validate with a mirror HEAD request and a full strict self-hosting proof run

## Rationale

`ftpmirror.gnu.org` is GNU's mirror front door. It redirects to a reachable
mirror while preserving the intended upstream artifact path.

That improves fetch reliability on hosts where direct `ftp.gnu.org` access is
slow or unreachable without changing bootstrap artifact identity.

## Non-Goals

- changing bootstrap source versions
- changing source hashes or names
- changing non-GNU bootstrap URLs such as GitHub or `musl.cc`
- changing self-build hermeticity policy
