# Protected transition v82 provider-role rerun

## Result

The complete protected rerun reached the final binutils audit check, then failed closed.
The exact failure was:

```text
GNU binutils event count is outside the accepted closure: observed 73991
```

The test ran through pueue task `3352` and finished in `1674.08s` with no successful test.
The scratch root is:

```text
/home/brittonr/.cargo-target/stagex-protected-transition-v82-provider-roles-20260730
```

## Diagnosis

The failed run retained `protected-exec-audit.json` before audit validation.
It contains 76,538 events. The binutils suffix contains 73,991 events.
The retained v81 audit contains 76,569 events, including a 74,022-event binutils suffix.

A BLAKE3/path authorization-count comparison found one difference in the binutils suffix:

```text
planned:coreutils-materialization:exec:coreutils-smoke:mkdir old=5411 new=5380
```

All other authorization counts matched. The difference is 31 allowed executions of the same declared coreutils `mkdir` identity. The failed run kept the same 4,891 sed invocations, component identities, installed-tool identities, archive identities, and no fallback events.

## Decision

Widen the lower bound from 73,996 to 73,991. Keep the upper bound at 74,057. Keep every exact critical executable count, unique executable identity check, ordering check, protected decision check, and no-fallback check unchanged.

This failed run is diagnostic evidence only. It is not a complete transition or provider-publication receipt.
