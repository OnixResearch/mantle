# Versioned producer contract for Mantle build requests and observations.
let Identity = std.contract.from_predicate (fun value =>
  std.is_string value && std.string.is_match "^b3:[0-9a-f]{64}$" value
) in
let Label = std.contract.from_predicate (fun value =>
  std.is_string value
  && std.string.length value > 0
  && std.string.length value <= 128
  && std.string.is_match "^[a-z][a-z0-9._:-]*$" value
) in
let CanonicalLabels = std.contract.from_predicate (fun value =>
  std.is_array value
  && std.array.length value <= 256
  && std.array.all (fun item => std.is_string item && std.string.is_match "^[a-z][a-z0-9._:-]*$" item) value
  && std.array.sort std.string.compare value == value
  && std.array.all (fun item => std.array.fold_left (fun count candidate => if candidate == item then count + 1 else count) 0 value == 1) value
) in
let Outcome = std.contract.from_predicate (fun value =>
  value == "success" || value == "failed" || value == "cancelled" || value == "timed_out" || value == "unknown"
) in
let CacheKind = std.contract.from_predicate (fun value =>
  value == "none" || value == "local" || value == "substitution"
) in
let Nullable = fun Contract => std.contract.custom (fun label value =>
  if value == null then 'Ok value else std.contract.check Contract label value
) in
let Request = {
  schema | std.contract.from_predicate (fun value => value == "mantle.build-request.v1"),
  identity | Identity,
  idempotency_key | Identity,
  effect_identity | Identity,
  attempt_identity | Identity,
  candidate_identity | Identity,
  pipeline_revision_identity | Identity,
  plan_identity | Identity,
  policy_identity | Identity,
  platform | Label,
  required_products | CanonicalLabels,
} in
let Product = {
  name | Label,
  artifact_identity | Identity,
  store_identity | Identity,
} in
let Metric = { name | Label, value | Number } in
let Cache = { kind | CacheKind, source_identity | Nullable Identity } in
let Observation = {
  schema | std.contract.from_predicate (fun value => value == "mantle.build-observation.v1"),
  identity | Identity,
  request_identity | Identity,
  outcome | Outcome,
  products | Array Product,
  builder_identity | Identity,
  worker_identity | Identity,
  store_identity | Identity,
  cache | Cache,
  logs | Array Identity,
  metrics | Array Metric,
  receipt_identity | Nullable Identity,
  non_claims | Array String,
} in
{
  request = Request,
  observation = Observation,
  identity = Identity,
  label = Label,
}
