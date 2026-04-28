Task-ID: V3
Covers: bootstrap.part.stage0.posix

# V3: Output-contract smoke test

Command:

```sh
out_path=target/live-part-stage0-posix/store/35ljc87nc2gcn7cxpj078qjmch8qpqzh-stage0-posix
for t in hex0 hex1 hex2 M1 M0 catm blood-elf kaem get_machine M2-Planet cp chmod mkdir untar ungz unbz2 unxz sha256sum match replace wrap; do
  test -x "$out_path/bin/$t"
done
printf '00\n' > "$tmp/zero.hex0"
"$out_path/bin/hex0" "$tmp/zero.hex0" "$tmp/zero.bin"
test "$(wc -c < "$tmp/zero.bin" | tr -d ' ')" = 1
test "$(od -An -tx1 "$tmp/zero.bin" | tr -d ' \n')" = 00
"$out_path/bin/get_machine" | grep -qx amd64
```

Exit status: 0
Provider selection: direct source build output from V2; legacy musl.cc provider not selected.
Fallback status/event marker: inherited from V2 build: `hermeticity: practical (no degraded facts)`.
Placeholder rejection result: not applicable to smoke command; V2 transcript contains no placeholder error.

Output:

```text
output=target/live-part-stage0-posix/store/35ljc87nc2gcn7cxpj078qjmch8qpqzh-stage0-posix
check executables
ok executable hex0
ok executable hex1
ok executable hex2
ok executable M1
ok executable M0
ok executable catm
ok executable blood-elf
ok executable kaem
ok executable get_machine
ok executable M2-Planet
ok executable cp
ok executable chmod
ok executable mkdir
ok executable untar
ok executable ungz
ok executable unbz2
ok executable unxz
ok executable sha256sum
ok executable match
ok executable replace
ok executable wrap
ok hex0 assembled one zero byte
get_machine=amd64
```

Raw output: `evidence/V3-smoke-output.txt`.
