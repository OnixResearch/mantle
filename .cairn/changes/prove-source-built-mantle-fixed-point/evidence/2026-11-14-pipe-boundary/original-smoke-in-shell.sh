#!/usr/bin/env bash
set -euo pipefail
ROOT=/home/brittonr/sbfp-20261113-berdKH/pipe-boundary-validation.YinuAM
B3=/nix/store/lp2fcmbwq2g4yxmb91kmsrb1a09s5rf5-b3sum-1.8.7/bin/b3sum
BWRAP=/nix/store/3hqwbwxij9lxhyrf2amsm2xg24xszmmx-bubblewrap-0.12.0/bin/bwrap
COHORT=/home/brittonr/sbfp-20261113-berdKH/prepared-cohort-22eb45ca-resumed
ORIGINAL=/home/brittonr/sbfp-20261113-berdKH/aws-original-source-02561621.TGISLN/original
set -x
test "$("$B3" --no-names "$ROOT/source/src/source_built_mantle_programs_native.rs")" = 5cb0ebc1fa11afd7c28f8381a46791b4e20b57a10ab3b1ad4adfc9f734888403
rustc --edition=2024 -O "$ROOT/source/src/source_built_mantle_programs_aws_helper.rs" -o "$ROOT/aws-source-helper"
python3 - "$ROOT/source/src/source_built_mantle_programs_native.rs" "$ROOT/pipe-wrapper.pl" <<'PY'
import pathlib,sys
source=pathlib.Path(sys.argv[1]).read_text()
prefix='const PIPE_SOURCE: &str = r###"'
if source.count(prefix)!=1: raise RuntimeError('ambiguous emitted wrapper source')
body=source.split(prefix,1)[1].split('"###;',1)[0]
pathlib.Path(sys.argv[2]).write_text(body)
PY
jq '[.steps[] | select(.id | test("[.]sources[.]perl-[0-9]+[.]generate$|[.]perlasm-[0-9]+[.]generate$")) | {family:(if (.id|contains(".sources.perl-")) then "ring" else "aws" end),id:.id,relative:.operation.arguments[2].parts[0].path,filename:.operation.arguments[4].parts[0].text}]' "$COHORT/inputs/mantle-source/bootstrap/source-built/mantle-stage1.json" > "$ROOT/original-bindings.json"
PERL=$(command -v perl)
PYTHON=$(command -v python3)
CC=$(command -v cc)
mkdir "$ROOT/smoke-work"
"$BWRAP" --unshare-all --unshare-user --die-with-parent --new-session \
  --ro-bind /nix/store /nix/store --dir /inputs \
  --ro-bind "$ORIGINAL" /inputs/aws-original \
  --ro-bind "$COHORT/inputs/vendor-deps/ring" /inputs/ring \
  --ro-bind "$ROOT/original-source-smoke.py" /inputs/smoke.py \
  --ro-bind "$ROOT/original-bindings.json" /inputs/bindings.json \
  --ro-bind "$ROOT/pipe-wrapper.pl" /inputs/wrapper.pl \
  --ro-bind "$ROOT/aws-source-helper" /inputs/aws-helper \
  --bind "$ROOT/smoke-work" /work --proc /proc --dev /dev --tmpfs /tmp \
  --clearenv --setenv PATH "$PATH" --setenv HOME /work/home --setenv LANG C --setenv LC_ALL C --setenv TZ UTC \
  --chdir /work -- "$PYTHON" /inputs/smoke.py \
  --bindings /inputs/bindings.json --wrapper /inputs/wrapper.pl --helper /inputs/aws-helper \
  --aws /inputs/aws-original --ring /inputs/ring --perl "$PERL" --cc "$CC" --b3sum "$B3" --out /work/results
"$B3" "$ROOT/smoke-work/results/equivalence.json"
