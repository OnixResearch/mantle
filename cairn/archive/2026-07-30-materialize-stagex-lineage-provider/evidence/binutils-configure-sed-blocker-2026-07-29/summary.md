# Binutils configure sed blocker

## Result

Mantle authenticated and materialized the GNU binutils 2.30 source.
The retained source identity is `809e8c1d946b14650362cf2929520efe07623fe069ce8c16c5870dafe6d93603`.
The checked recipe identity is `406d4aaecf2cc8c9b357463fb8f48f185acaa67e77c4136e5908cca851efd177`.

The bounded configure probe ran the authenticated `intl/configure` script with the protected later Bash.
It used the declared TinyCC wrapper, native musl, and exact source-built tool paths.
Eleven bounded preprocessor probes ran and were recorded.
The configure checks completed and created `config.status`.

`config.status` then failed while it generated `Makefile`.
The protected sed output truncated Autoconf's substitution program to this invalid Gawk statement:

```text
S["LTLIBOBJS"]=
```

The command exited with status 1.
The negative test accepted only this exact bounded blocker.
It did not treat the failed configure as successful binutils evidence.

## Evidence

- Task `352`: the exact configure blocker test passed.
- Task `284`: authenticated source materialization passed.
- Task `332`: three positive and negative source-boundary tests passed.
- Task `363`: strict first-party Clippy passed.
- Task `375`: Cairn validation was blocked because the current sibling Cairn source has an unclosed delimiter in `crates/cairn-core/src/lib.rs`.
- `preprocess.audit` records each accepted preprocessor input.
- `intl-config.status` preserves the failing generated boundary.
- `configure.stderr.txt` preserves the final Gawk and `config.status` diagnostics.

## Boundary

No ambient shell, BusyBox, Nix-store discovery, or fallback result was accepted by the test.
The probe used a create-new tool namespace populated from explicit protected paths.
The namespace controls lookup mechanics; exact path and digest checks remain the intended authority.

This evidence does not prove successful configure, Make execution, binutils, compiler correctness, or provider admission.
The next accepted step must replace or repair the protected sed bridge under a separate exact producer boundary.
