# Reviewable metadata for the source-bundle workflow.
# `source-root.ncl` is a constructor because file URLs must be explicit and absolute.
let Workflow = {
  workflow | String,
  source_constructor | String,
  payload | String,
  payload_hash | String,
} in

{
  workflow = "offline-source-bundle-handoff",
  source_constructor = "source-root.ncl",
  payload = "sources/payload.txt",
  payload_hash = "sha256-oi30yn6lrWqckP9gGVy/BNgDCI7nRC77ORXuqz3kR30=",
} | Workflow
