let mantle = import "lib.ncl" in
let plan_producer = import "../run-positive/producer.ncl" in
{
  name = "static-duplicate-binding-consumer",
  builder = "/bin/sh",
  addressing_mode = 'input-addressed,
  inputs = [
    { name = "collision", producer = plan_producer, plan_output = "plan", root = "unit.ia", unit_output = "out" },
    { name = "collision", producer = plan_producer, plan_output = "plan", root = "unit.ca", unit_output = "out" },
  ],
  env = { ROOT = "{{mantle-plan-output:collision}}" },
  args = ["-c", "echo unexpected > \"$out\""],
} | mantle.Derivation
