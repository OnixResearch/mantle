let mantle = import "lib.ncl" in
let plan_producer = import "producer.ncl" in
{
  static_ia = {
    name = "static-consumer-ia",
    builder = "/bin/sh",
    addressing_mode = 'input-addressed,
    inputs = [{ name = "ia", producer = plan_producer, plan_output = "plan", root = "unit.ia", unit_output = "out" }],
    env = { ROOT = "{{mantle-plan-output:ia}}" },
    args = ["-c", m%"
      set -eu
      IFS= read -r payload < "{{mantle-plan-output:ia}}"
      printf '%s\n%s\n' "$payload" "$ROOT" > "$out"
    "%],
  } | mantle.Derivation,
  static_ca = {
    name = "static-consumer-ca",
    builder = "/bin/sh",
    addressing_mode = 'input-addressed,
    inputs = [{ name = "ca", producer = plan_producer, plan_output = "plan", root = "unit.ca", unit_output = "out" }],
    env = { ROOT = "{{mantle-plan-output:ca}}" },
    args = ["-c", m%"
      set -eu
      IFS= read -r payload < "$ROOT"
      printf '%s\n%s\n' "$payload" "$ROOT" > "$out"
    "%],
  } | mantle.Derivation,
  static_both = {
    name = "static-consumer-both",
    builder = "/bin/sh",
    addressing_mode = 'input-addressed,
    inputs = [
      { name = "zeta", producer = plan_producer, plan_output = "plan", root = "unit.ia", unit_output = "out" },
      { name = "alpha", producer = plan_producer, plan_output = "plan", root = "unit.ca", unit_output = "out" },
    ],
    env = { IA = "{{mantle-plan-output:zeta}}", CA = "{{mantle-plan-output:alpha}}" },
    args = ["-c", m%"
      set -eu
      IFS= read -r ia_payload < "$IA"
      IFS= read -r ca_payload < "$CA"
      printf '%s\n%s\n%s\n%s\n' "$ia_payload" "$ca_payload" "$IA" "$CA" > "$out"
    "%],
  } | mantle.Derivation,
}
