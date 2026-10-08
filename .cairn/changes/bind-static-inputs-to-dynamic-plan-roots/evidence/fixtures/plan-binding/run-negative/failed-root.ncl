let mantle = import "lib.ncl" in
let root_builder = {
  name = "static-root-builder",
  builder = "builtin:fetchurl",
  env = {
    url = "file:///home/brittonr/mantle-cairn-finish/tmp/dyn-fixtures/plan-binding/run-positive/root-builder.sh",
    executable = "1",
  },
  fixed_output = {
    hash = "881ce6a52b548212a45b82bfb3db530a076d7e1d2e3838b440f5991ecae0cc57",
    algo = 'sha256,
    mode = 'flat,
  },
  addressing_mode = 'input-addressed,
} | mantle.Derivation in
let plan_producer = {
  name = "static-failed-root-producer", builder = "/bin/sh",
  addressing_mode = 'input-addressed,
  outputs = ["out", "plan"], dynamic_plan_outputs = ["plan"],
  inputs = [root_builder],
  args = ["-c", m%"
    set -eu
    for builder in "$NIX_STORE"/*-static-root-builder; do
      [ -x "$builder" ] && break
    done
    [ -x "$builder" ]
    printf '%s\n' '{"schema":"mantle-plan-v1","producer":{"logical_name":"static-failed-root-producer","goal_hint":null},"sources":[],"units":[{"id":"unit.fail","derivation":{"name":"static-root-fail","builder":"'"$builder"'","system":"x86_64-linux","args":["fail"],"outputs":["out"],"env":{},"inputs":[{"kind":"store_path","path":"'"$builder"'"}],"fixed_output":null,"addressing_mode":"input-addressed","sandbox":"native","dynamic_plan_outputs":[]},"requested_outputs":["out"],"policy":{"sandbox":"inherit","substitutions":"inherit","store_prefix":"inherit","host_paths":"none"}}],"roots":["unit.fail"],"provenance":{}}' > "$plan"
    echo producer-ready > "$out"
  "%],
} | mantle.Derivation in
{
  failed_root = {
    name = "static-failed-root-consumer", builder = "/bin/sh",
    addressing_mode = 'input-addressed,
    inputs = [{ name = "bad", producer = plan_producer, plan_output = "plan", root = "unit.fail", unit_output = "out" }],
    env = { ROOT = "{{mantle-plan-output:bad}}" },
    args = ["-c", "/bin/cat \"$ROOT\" > \"$out\""],
  } | mantle.Derivation,
}
