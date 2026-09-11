# Stable SpaceWasm report canonicalizer (grammar version 1).
#
# Input:  raw libtest JSON harness lines (one JSON object per line).
# Output: compact canonical identity-input JSON whose byte layout matches
#         crunch-spacewasm-core's `StableReportIdentityInput`
#         (serde_json compact, struct field order):
#
#   {"schema":...,"suite":...,"command":...,"tests":[{"name":...,"status":...}],"encoding_version":1}
#
# Admission rules mirror the Rust core:
#   - only `type == "test"` (event ok|failed|ignored) and `type == "suite"`
#     (event started|ok|failed) lines are admitted; anything else fails;
#   - test names must be non-empty and unique; duplicates fail;
#   - the suite summary must appear exactly once and must not contradict
#     the admitted test outcomes;
#   - facts are emitted in canonical (name) order.

def mapstatus:
  if . == "ok" then "passed" elif . == "failed" then "failed" elif . == "ignored" then "skipped"
  else error("unknown test event") end;

($capture | split("\n") | map(select(length > 0)) | map(fromjson))
| map(select(.type != null))
| map(
    if .type == "test" then
      (if ((. | keys) - ["type", "name", "event", "stdout"] | length) > 0
       then error("test line outside the versioned grammar") else . end)
      | (if .event == "started"
         then { kind: "started" }
         else { kind: "test", name: .name, event: .event } end)
    elif .type == "suite" then
      (if ((. | keys) - ["type", "event", "test_count", "passed", "failed", "ignored", "measured", "filtered_out", "exec_time"] | length) > 0
       then error("suite line outside the versioned grammar") else . end)
      | { kind: "suite", event: (.event // ""), test_count: (.test_count // 0) }
    else
      { kind: "unknown" }
    end
  )
| map(
    if .kind == "unknown" then error("line outside the versioned grammar")
    elif .kind == "started" then { kind: "skip" }
    elif .kind == "test" then
      (if (.name // "") == "" then error("empty test name") else . end)
      | (if (.event == "ok" or .event == "failed" or .event == "ignored")
         then { name: .name, status: (.event | mapstatus) }
         else error("unknown test event") end)
    elif .kind == "suite" then
      (if (.event == "started" or .event == "ok" or .event == "failed")
         and (.test_count <= 4096)
       then { name: "", status: ("suite:" + .event) }
       else error("unknown suite event") end)
    else . end
)
| map(select(.kind != "skip"))
| {
    tests: [ .[] | select(.name != "") ],
    summaries: [ .[] | select(.name == "") | .status ],
    names: [ .[] | select(.name != "") | .name ]
  }
| (if ([.summaries[] | select(. == "suite:ok" or . == "suite:failed")] | length) != 1
   then error("suite summary must appear exactly once") else . end)
| (if ((.summaries[0] == "suite:ok") and ([.tests[] | select(.status == "failed")] | length) > 0)
   then error("suite summary contradicts admitted outcomes")
   elif ((.summaries[0] == "suite:failed") and ([.tests[] | select(.status == "failed")] | length) == 0)
   then error("suite summary contradicts admitted outcomes")
   else . end)
| (if ((.names | length) != (.names | unique | length))
   then error("duplicate test records are errors") else . end)
| {
    schema: "mantle-spacewasm-stable-report-v1",
    suite: $suite,
    command: $command,
    tests: [ .tests | sort_by(.name)[] | { name: .name, status: .status } ],
    encoding_version: 1
  }
