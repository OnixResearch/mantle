# I1 upstream ordering and output contract

- Task-ID: I1
- Covers: `bootstrap.part.sed.4.0.9.tcc`
- Status: complete

## Upstream ordering

Upstream `parts.rst` lists the first `sed 4.0.9` part after `tar 1.12` and before `bzip2 1.0.8` in the Mes/TinyCC phase (lines 300-305 in the local reference checkout). The later `sed 4.0.9` section at lines 448-451 is the musl rebuild and is tracked separately by `live-part-sed-4-0-9-musl`.

## Matching upstream scripts

- `steps/sed-4.0.9/pass1.kaem` performs the Mes-libc build, installs `${PREFIX}/bin/sed`, and records `/usr/bin/sed` checksum evidence.
- `steps/sed-4.0.9/mk/main.mk` builds library objects plus `sed/{compile,execute,regexp,fmt,sed}.o` with TinyCC and `LIBC=mes` selecting `getline`.
- `steps/sed-4.0.9/sources` pins `https://mirrors.kernel.org/gnu/sed/sed-4.0.9.tar.gz` with sha256 `c365874794187f8444e5d22998cd5888ffa47f36def4b77517a808dec27c0600`. Crunch uses the same content under the SRI hash `sha256-k2YhQtIAYx0rPNFootM1VKppNsaoJsaTFpJ34SSPkIk=`.

## Output contract

Crunch should produce `sed-4.0.9-tcc` with executable `bin/sed`. The direct consumer chain is the tcc-musl-v2/bzip2-musl prerequisite path; downstream stages should not depend on a `libsed.a` output from this derivation.
