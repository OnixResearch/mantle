# OpenSSL Perl `re` repair evidence

## Scope

This evidence covers the receipt-bound Perl repair for the full-source Rust route.
It does not prove Rust provider completion.

## Failed construction

The detached v3 Rust construction did not publish a provider output.
The OpenSSL build stopped at this generated command:

```text
perl "-I." "-Iproviders/common/der" "-Mconfigdata" "-Mconfigdata" "-Mconfigdata" "-Moids_to_c" "util/dofile.pl" "-oMakefile" providers/common/include/prov/der_digests.h.in > providers/common/include/prov/der_digests.h
```

A direct replay with the v2 Perl output returned status 255:

```text
're' not installed!? (Can't load module re, dynamic loading not available in this perl.
  (You may need to build a new perl executable which either supports
  dynamic loading or has the re module statically linked into it.)
 at providers/common/der/oids_to_c.pm line 83
)
```

The durable direct-replay stderr is:

```text
cairn/changes/bind-full-source-rust-provider/evidence/openssl-perl-re-v2-probe.stderr.txt
```

The failed scratch directory was removed only after the missing provider output was confirmed.

## Repair

`bootstrap/perl-5.10.1-gcc10.ncl` now links `re` as a static extension.
Its positive checks now load `re` before publication and after copied-tree relocation.

The rebuilt Perl output is:

```text
/home/brittonr/.cache/mantle-full-source-20260718/gcc40-store/3frwc382f3c7k8h0yi256v5q2g1lxwip-perl-5.10.1-full-source-gcc10-v3
```

Pueue task 456 built this output from authenticated Perl 5.10.1 source.
The task used the admitted full-source native provider and declared bootstrap tools.

Pueue task 457 replayed the exact failed OpenSSL generator with the v3 Perl output:

```text
status=0
bytes=6130
```

The durable replay summary is `openssl-perl-re-v3-probe.stdout.txt`.

## Bound evidence

The v3 host-tool manifest is:

```text
cairn/changes/bind-full-source-rust-provider/evidence/full-source-rust-host-tools-v3-2026-07-26/full-source-rust-host-tools.json
```

Its Perl construction receipt includes `openssl-generator-re` as a positive check.
Pueue task 468 created this manifest without replacing earlier evidence.

Pueue task 478 ran the focused Rust tests:

```text
running 15 tests
...............
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 1623 filtered out; finished in 0.00s
```

## Scalar-output failure in detached v4

The detached v4 construction passed the earlier `re` blocker.
It reached the final first-stage Cargo link and then failed.
The linker reported an undefined `ossl_param_find_pidx` reference.
The durable failure excerpt is `openssl-params-idx-v4-link-failure.txt`.

OpenSSL generated and compiled `crypto/params_idx.c`.
The archive contained `libcrypto-lib-params_idx.o`, but that object defined no symbols.
Pueue task 489 inspected the archive with receipt-bound `ar` and `nm`.
The durable summary is `openssl-params-idx-v4-symbol-probe.stdout.txt`.

`OpenSSL::paramnames::produce_decoder` captures generated C through scalar-backed Perl output.
The v3 Perl could open this stream, but it returned an empty scalar.
Pueue task 491 reproduced that failure with the same `open local *STDOUT` operation.
The probe returned status 255.
Its stderr is `openssl-perl-scalar-v3-probe.stderr.txt`.

## Scalar-output repair

`bootstrap/perl-5.10.1-gcc10.ncl` now links `PerlIO/scalar` statically.
The derivation now checks the exact scalar-capture operation before publication.
It repeats this check after copied-tree relocation.
The host-tool contract names this check as `openssl-param-decoder-scalar-capture`.

Pueue task 499 built this repaired output:

```text
/home/brittonr/.cache/mantle-full-source-20260718/gcc40-store/hk6iyjf76n3c5dsqwcfyjvbqdm5fxi7s-perl-5.10.1-full-source-gcc10-v4
```

Pueue task 500 passed the exact scalar-capture probe.
It also generated a decoder that defines `ossl_param_find_pidx`.
The durable output is `openssl-perl-scalar-v4-probe.stdout.txt`.

Pueue task 507 created this receipt-bound host-tool manifest:

```text
cairn/changes/bind-full-source-rust-provider/evidence/full-source-rust-host-tools-v4-2026-07-26/full-source-rust-host-tools.json
```

Its Perl receipt includes `openssl-param-decoder-scalar-capture`.

## Current non-claim

Detached v4 did not publish a Rust provider.
The Perl scalar-output repair is not Rust provider completion evidence.
A fresh construction must pass all provider and binding checks.
