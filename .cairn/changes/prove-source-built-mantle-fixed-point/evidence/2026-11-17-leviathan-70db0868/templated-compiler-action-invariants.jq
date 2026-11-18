# Inert check that every templated compiler action in a frozen Mantle stage
# recipe satisfies the strict shape admission now requires.
#
#   jq -c -f templated-compiler-action-invariants.jq mantle-stage1.json
#
# Output: {stage, templated, failures:[{id, errors:[...]}]}. No failures and a
# non-zero templated count means the strict rules match the authored bytes.
# Reads recipe data only; executes nothing.
.stage_id as $stage
| (.steps | map({ key: .id, value: . }) | from_entries) as $steps
| [
    { kind: "literal", text: "--compiler-invocation" },
    { kind: "input", name: "template", path: "" },
    { kind: "input", name: "actual-metadata", path: "compiler.env" },
    { kind: "output", name: "invocation", path: "" },
    { kind: "output", name: "environment-data", path: "" }
  ] as $helper_parts
| [$helper_parts[] | [.]] as $expected_helper_parts
| [
    .steps[]
    | select(
        .operation.kind == "exec"
        and .operation.executable_input == "rustc"
        and .operation.invocation_input != null
      )
    | . as $compile
    | $compile.operation.invocation_input as $invocation
    | ($steps[$invocation] // null) as $helper
    | (($helper.inputs.template.action_id // "") | split("/") | .[1] // "") as $template_id
    | ($steps[$template_id] // null) as $template_step
    | (($template_step.operation.text // "{}") | fromjson) as $template
    | {
        id: $compile.id,
        errors: [
          (if $invocation == ($compile.id + "-invocation") then empty else "invocation name" end),
          (if (($compile.operation.arguments | length) == 0
               and (($compile.operation.environment // {}) | length) == 0)
           then empty else "own words" end),
          (if $compile.inputs[$invocation] == { kind: "produced", action_id: ($stage + "/" + $invocation), output_id: "invocation" }
           then empty else "invocation product" end),
          (if $compile.inputs[$invocation + "-data"] == { kind: "produced", action_id: ($stage + "/" + $invocation), output_id: "environment-data" }
           then empty else "data product" end),
          (if ($helper != null
               and $helper.operation.executable_input == "metadata-helper"
               and ($helper.inputs["metadata-helper"].kind // "") == "produced"
               and (($helper.inputs["metadata-helper"].action_id // "") | startswith($stage + "/")))
           then empty else "helper executable" end),
          (if ($helper != null and [$helper.operation.arguments[] | .parts] == $expected_helper_parts)
           then empty else "helper operands" end),
          (if ($helper != null
               and $helper.inputs.template == { kind: "produced", action_id: ($stage + "/" + $invocation + "-template"), output_id: "template" })
           then empty else "template reference" end),
          (if ($template_step != null
               and $template_step.operation.kind == "write-text"
               and $template_step.operation.output == "template"
               and (($template_step.inputs // {}) | length) == 0)
           then empty else "template action" end),
          (if ($template | keys) == ["data_name", "inputs", "invocation", "outputs", "runtime"]
           then empty else "template fields" end),
          (if $template.data_name == ($invocation + "-data") then empty else "data name" end),
          (if $template.inputs == ([$compile.inputs | keys[] | select(. != $invocation and . != ($invocation + "-data"))])
           then empty else "template inputs" end),
          (if $template.outputs == ($compile.outputs | keys) then empty else "template outputs" end),
          (if $template.runtime == (($compile.operation.runtime_mounts // {}) | keys) then empty else "template runtime" end)
        ]
      }
    | select((.errors | length) > 0)
  ] as $failures
| {
    stage: $stage,
    templated: ([
      .steps[]
      | select(
          .operation.kind == "exec"
          and .operation.executable_input == "rustc"
          and .operation.invocation_input != null
        )
    ] | length),
    failures: $failures
  }
