# Checked foreign-derivation validate/plan workflow.
let Workflow = {
  schema | String,
  id | String,
  positive_commands | Array String,
  negative_cases | Array String,
  evidence_fields | Array String,
  non_claims | Array String,
} in

{
  schema = "mantle-example-workflow-v1",
  id = "foreign-import-handoff",
  positive_commands = [
    "mantle --json foreign-import validate --graph fixtures/guix-hello.graph.json --package-index fixtures/guix-hello.index.json --policy fixtures/policy.json",
    "mantle --json foreign-import plan --graph fixtures/nix-hello.graph.json --package-index fixtures/nix-hello.index.json --policy fixtures/policy.json --package hello --system x86_64-linux",
  ],
  negative_cases = [
    "untrusted-cache-hint",
    "stale-raw-graph-digest",
    "undeclared-sandbox-capability",
  ],
  evidence_fields = ["accepted", "receipt.raw_graph_digest", "plan.roots", "plan.non_claims"],
  non_claims = [
    "not-output-trust",
    "not-local-rebuild-success",
    "not-foreign-frontend-correctness",
  ],
} | Workflow
