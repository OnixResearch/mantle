# V2 grep 2.4 output-path evidence

Task-ID: V2
Covers: bootstrap.part.grep.2.4.runtime-validation
Captured: 2026-05-08T21:42:24Z

## Result

The focused validation produced one successful `grep-2.4-musl` output.

- derivation key: `/crunch/store/pjdkqdhzrp0mr0yrjj5002dpij3m979b-grep-2.4-musl.drv`
- physical output path at validation time: `/home/brittonr/git/crunch/crunch/.crunch-drain/grep-musl-store9/mnd29ba2jam4hwgfmrxg0k3ckxhqn2kl-grep-2.4-musl`
- logical output path: `/crunch/store/mnd29ba2jam4hwgfmrxg0k3ckxhqn2kl-grep-2.4-musl`
- build report schema: `crunch-build-report-v1`
- succeeded_total: `1`
- built_total: `1`
- failed_total: `0`

The scoped follow-up records this as runtime proof from the archived focused rerun. The transient physical `.crunch-drain/grep-musl-store9` path is not relied on as durable storage; the durable evidence is the copied build report, validation summary, and root derivation log.

Provider/fallback status: the derivation declares its bootstrap inputs (`stage0`, `tcc-musl-v2`, `musl-1.1.24-tcc-musl`, `make-tcc`, `sed-tcc`, and `grep-2.4-src`) and the validation report contains no failed outcomes or FOD mismatches. No host grep/provider fallback is claimed.
