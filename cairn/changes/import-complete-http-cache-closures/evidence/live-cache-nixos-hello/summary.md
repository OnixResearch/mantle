# Live `cache.nixos.org` hello closure proof

Date: 2026-08-01

## Goal

Import one recorded Nixpkgs `hello` runtime closure from `cache.nixos.org` into fresh Mantle store and state directories. Keep Nix commands unavailable during consumption.

## Inputs

- Root: `/nix/store/zi2bj2hlavv8q743li2s9diqbcpmrf9b-hello-2.12.3`
- Cache: `https://cache.nixos.org`
- Trusted key: `cache.nixos.org-1:6NCHdD59X431o0gWypbMrAURkbJ16ZPMQFGspcDShjY=`
- Logical store mode: `--nix-compat`
- Consumer `PATH`: `/nonexistent`
- Explicit certificate file: `/etc/ssl/certs/ca-bundle.crt`
- Pueue task: `7217`

The first scrubbed attempt, task `7213`, failed before HTTP client creation because `SSL_CERT_FILE` was set to an empty value. Task `7217` supplied the explicit certificate path and succeeded. No Nix command was available through `PATH` in the successful consumption run.

## Import result

```text
CLOSURE plan_blake3=41e2fae2f6a12d382c31ffda5346a2b738f51b3b7ad33a8a023b704469933cac members=5 reused=0 root=zi2bj2hlavv8q743li2s9diqbcpmrf9b-hello-2.12.3 admitted=true
PULL 57iz36553175g3178pvxjij8z5rcsd4n-glibc-2.42-61
PULL 6qa00czc79b3nb6ld0mdyacfp2p1k3jx-libidn2-2.3.8
PULL bf6wgamqnl3c91iamlb1branrfcwwy7x-libunistring-1.4.2
PULL g54b6ghpnn98hfdz4yqw87w10c3hx8bv-xgcc-15.2.0-libgcc
PULL zi2bj2hlavv8q743li2s9diqbcpmrf9b-hello-2.12.3
```

```text
imported=5 skipped_present=0 skipped_untrusted=0 skipped_hash_mismatch=0 skipped_missing_nar=0 skipped_parse_error=0 nar_bytes=38013944
```

Exit status: `0`.

Redb emitted repair warnings while opening the new proof-local databases. The import still returned success and the repeated completeness run reused every member.

## Complete local reuse

The same command ran again with `PATH=/nonexistent`. Pueue task `7224` returned:

```text
CLOSURE plan_blake3=41e2fae2f6a12d382c31ffda5346a2b738f51b3b7ad33a8a023b704469933cac members=5 reused=5 root=zi2bj2hlavv8q743li2s9diqbcpmrf9b-hello-2.12.3 admitted=true
```

```text
imported=0 skipped_present=5 skipped_untrusted=0 skipped_hash_mismatch=0 skipped_missing_nar=0 skipped_parse_error=0 nar_bytes=0
```

The unchanged plan BLAKE3 and five complete local reuses prove that every planned member had matching PathInfo, accepted signatures, and complete castore content in the proof state.

## Isolated execution

Pueue task `7221` mounted only the proof store over logical `/nix/store` in bubblewrap, then ran the imported binary.

```text
Hello, world!
```

Exit status: `0`.

## Strongest proven state

The recorded `hello` output and its five-member runtime closure were imported from `cache.nixos.org` under the selected public key. Every member passed Mantle's narinfo, path, signature, NAR hash, PathInfo, and castore admission. The repeated run reused all five members. The binary executed with logical `/nix/store` mapped to the proof store.

## Non-claims

This evidence does not bind the root to a foreign-import receipt. It does not prove nixpkgs evaluation, package correctness, local rebuild compatibility, evaluator parity, reproducibility, private-cache authentication, compiler correctness, or release eligibility.
