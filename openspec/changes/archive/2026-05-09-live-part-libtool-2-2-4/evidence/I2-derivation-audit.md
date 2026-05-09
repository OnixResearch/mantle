# I2 derivation audit evidence

Task-ID: I2
Covers: bootstrap.part.libtool.2.2.4

Audited `bootstrap/libtool-2.2.4.ncl`:

- source pin: `https://mirrors.kernel.org/gnu/libtool/libtool-2.2.4.tar.bz2`
- expected output contract: installed `bin/libtool`, `bin/libtoolize`, and `share/libtool`
- declared predecessors: TinyCC/musl v2, Make, sed, m4, perl, coreutils, gawk, grep, diffutils, bash, autoconf 2.69, automake 1.15.1, and stage0
- removed deviation: configure/install suppression and partial manual copy fallback

No host libtool, Nix-provided libtool, or legacy tool output may be substituted for bootstrap proof.
