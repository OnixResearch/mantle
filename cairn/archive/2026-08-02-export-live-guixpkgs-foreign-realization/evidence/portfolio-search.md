# Architecture portfolio

## Candidates

### Native Guix substitute support

This route would preserve `/gnu/store` and verify Guix canonical S-expression
signatures. It requires a new narinfo parser, libgcrypt-compatible signature
verification, Guix ACL policy, multi-URL compression selection, and new prefix
contracts.

**Disposition:** Rejected for this change. It proves direct Guix substitution,
not the requested GuixPkgs export boundary.

### Direct GuixPkgs Cachix hydration

GuixPkgs advertises a Nix-compatible Cachix cache. Its translated dependencies
were available, but the pinned raw `hello` root returned no NARInfo. Nix dry-run
reported one raw root derivation to build.

**Disposition:** Rejected as an incomplete root boundary.

### Local rebuild of the complete translated bootstrap

The exported graph has 1,176 units. A complete Mantle rebuild would test Guix
sandbox and bootstrap parity. It is not required for signed cache-only
consumption.

**Disposition:** Deferred as a separate claim.

### Producer-signed translated closure

Producer-side Nix built the single missing raw root from the pinned translated
graph. It then signed and exported the four-member runtime closure. Mantle used
its existing signed Nix cache route without Nix or Guix during consumption.

**Disposition:** Selected. It is the smallest boundary that proves live
GuixPkgs export, bounded realization, reuse, hydration, and audit.

## Adversarial review

The selected route shifts output trust to the dedicated exporter key. The key
does not prove `guix-transfer` correctness or original `/gnu/store` identity.
Those facts remain explicit non-claims.

The proof retains the public key, NARInfo facts, producer identities, and
consumer receipts. It does not retain the secret key or mutable store state.
Wrong keys, missing members, low limits, and receipt tampering fail before output
export.

The audit does not promote a passing state. It retains the one unsupported
ambient-Perl executable and keeps `realized` as the strongest state.

## Independent review

A secondary review challenged Unicode separators, decompression limits,
single-payload scanning, and ambient Perl handling. Unix store paths use `/` and
literal `..` components, so Unicode text cannot create a lexical parent step.
The shared decoder reads through the configured expanded-byte limit plus one.
A gzip or zstd stream is one payload unless its decompressed bytes are a known
archive.

The ambient Perl concern was valid. An explicit negative test now proves that an
executable mtrace-style shell and Perl polyglot remains `UnsupportedExecutable`.
