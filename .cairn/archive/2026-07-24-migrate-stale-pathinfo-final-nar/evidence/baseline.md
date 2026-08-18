# Baseline evidence

## Exact retained failure

The retained historical record is:

```text
logical_path=/mantle/store/57dqxg2kjddkvjmwkvkr54nifqfs38l0-bison-2.3-gcc-v6
state_dir=/home/brittonr/.cache/mantle-full-source-20260718/gcc40-state
physical_store=/home/brittonr/.cache/mantle-full-source-20260718/gcc40-store
recorded_nar_size=972064
recorded_nar_sha256=fff5e607c24805da4403a7619a5a967b61d0ea3fca268103773a47bde813f3eb
observed_nar_size=972064
observed_nar_sha256=c4bc724b6e5f92047cc57e6d9da09a71eb6c46110fac04254eba87062f1c6738
```

Repaired archive export correctly rejects it before bytes:

```text
exit_status=3
archive_bytes=0
error: archive export: export: stale final NAR facts for 57dqxg2kjddkvjmwkvkr54nifqfs38l0-bison-2.3-gcc-v6: recorded size 972064 sha256 fff5e607c24805da4403a7619a5a967b61d0ea3fca268103773a47bde813f3eb, observed size 972064 sha256 c4bc724b6e5f92047cc57e6d9da09a71eb6c46110fac04254eba87062f1c6738
```

Ordinary `store sign` signs the recorded fingerprint and therefore cannot repair the mismatch. Archive export/import is not a migration path because weakening export preflight would regress the accepted fail-closed boundary.

## Baseline tests

Pre-change focused results:

```text
$ nix develop -c cargo test -p crunch-store --lib --tests
test result: ok. 225 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.26s

$ nix develop -c cargo test -p mantle --test integration store_sign
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 59 filtered out; finished in 0.07s
```

## Claim boundary

A successful migration can prove that one exact local PathInfo was remeasured from complete castore content and signed by the selected local key. It cannot recover historical signer consent, prove builder/source/compiler correctness, establish independent reproducibility, validate arbitrary stale metadata, or establish release eligibility.
