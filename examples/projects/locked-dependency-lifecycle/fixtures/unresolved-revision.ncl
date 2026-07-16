# Negative fixture: a Git revision selector must resolve to a non-empty identity.

{
  version = "1.0.0",
  inputs = [
    {
      name = "unresolved-revision",
      kind = {
        type = "git",
        repository = "https://source.example.invalid/project.git",
        reference = { ref_type = "rev", ref_value = "" },
      },
    },
  ],
  patches = [],
}
