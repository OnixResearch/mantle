# Use GNU ftpmirror front door for bootstrap sources

## Why

Full strict self-hosting proof runs on this host repeatedly stalled or failed on
`https://ftp.gnu.org/gnu/...` fetches with timeouts and `Network is unreachable`,
even while other bootstrap sources such as GitHub release assets and `musl.cc`
were reachable.

The bootstrap source URLs in `bootstrap/*.ncl` are reliability-sensitive proof
inputs. Switching the GNU tarball URLs to `https://ftpmirror.gnu.org/...`
keeps the same pinned artifacts and hashes, but lets GNU route requests through
an available mirror instead of depending on a single front-end host.

This is separate from `tighten-self-build-proof-hermeticity`: that archived
change was about later-stage strict tool/source selection and proof reporting,
not bootstrap source mirror policy.

## What Changes

- rewrite GNU bootstrap tarball URLs from `https://ftp.gnu.org/gnu/...` to
  `https://ftpmirror.gnu.org/...`
- keep all existing names and hashes unchanged
- verify the strict self-hosting proof still passes with the mirror URLs

## Capabilities

### New Capabilities

- `bootstrap-gnu-mirror-frontdoor`: bootstrap fetches use GNU's mirror front
  door instead of a single `ftp.gnu.org` endpoint

## Impact

- **Files**: `bootstrap/binutils.ncl`, `bootstrap/dash.ncl`, `bootstrap/gcc.ncl`, `bootstrap/make.ncl`
- **Behavior**: bootstrap fetches for GNU-hosted tarballs become more resilient on flaky hosts
- **Testing**: validate with the full strict self-hosting proof run
