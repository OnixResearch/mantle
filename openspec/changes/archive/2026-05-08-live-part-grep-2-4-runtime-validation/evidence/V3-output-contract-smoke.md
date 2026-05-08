# V3 grep 2.4 output-contract smoke evidence

Task-ID: V3
Covers: bootstrap.part.grep.2.4.runtime-validation
Captured: 2026-05-08T21:42:24Z

## Result

The `bootstrap/grep-2.4-musl.ncl` builder itself verifies the installed output contract before finishing:

- runs `echo "hello" | ./grep hello`;
- runs `echo "HELLO" | ./grep -q HELLO`;
- runs `echo "abc" | ./grep -E '^ab'`;
- installs `$out/bin/grep`;
- creates `$out/bin/egrep -> grep`;
- creates `$out/bin/fgrep -> grep`;
- fails closed on `test -x "$out/bin/grep"`, `test -L "$out/bin/egrep"`, and `test -L "$out/bin/fgrep"`.

The copied root derivation log records the successful builder transcript:

```text
# status: success
hello
abc
```

Because the focused validation exited 0 after these checks, `grep`, `egrep`, and `fgrep` are considered proven for this bootstrap bridge output.
