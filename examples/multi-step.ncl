# Multi-step build using only shell builtins (no coreutils).
# The sandbox's /bin/sh is busybox — mkdir/chmod aren't on PATH.
# So this writes a structured multi-line output to a flat $out file.
let crunch = import "lib.ncl" in
{
  name = "multi-step",
  builder = "/bin/sh",
  args = [
    "-c",
    m%"
      echo '--- manifest ---' > $out
      echo 'name: multi-step' >> $out
      echo 'version: 0.1' >> $out
      echo '--- content ---' >> $out
      echo 'line 1: hello' >> $out
      echo 'line 2: from crunch' >> $out
      echo '--- end ---' >> $out
    "%,
  ],
} | crunch.Derivation
