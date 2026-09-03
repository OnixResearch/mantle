# Typed review authority for Mantle's public machine-readable JSON surfaces.
# Rust reads only the JSON-compatible Nickel value between the data markers.
let NonEmpty = std.contract.from_predicate (fun value => std.is_string value && std.string.length value > 0) in
let SurfaceClass = std.contract.from_predicate (fun value => std.array.elem value ["contracted", "internal", "debug", "compatibility", "external"]) in
let StringArray = Array String in
let Artifacts = {
  schema | String,
  generated_contract | String,
  positive_fixtures | StringArray,
  negative_fixture_set | String,
} in
let VersionPolicy = {
  current | String,
  supported | StringArray,
  unknown_version_behavior | NonEmpty,
  compatibility_converter | String,
  compatibility_fixtures | StringArray,
} in
let Freshness = {
  schema_blake3 | String,
  contract_blake3 | String,
  prelude_blake3 | String,
  fixture_set_blake3 | String,
  producer_identity_blake3 | String,
  consumer_policy_blake3 | String,
} in
let Surface = {
  id | NonEmpty,
  class | SurfaceClass,
  rust_owner | NonEmpty,
  producer | { command_or_api | NonEmpty, marker | NonEmpty, source_paths | StringArray },
  consumers | StringArray,
  artifacts | Artifacts,
  version_policy | VersionPolicy,
  validation_commands | StringArray,
  fixture_coverage | StringArray,
  freshness_strategy | NonEmpty,
  freshness | Freshness,
  non_claims | StringArray,
  rationale | NonEmpty,
} in
let Registry = {
  registry_schema | std.contract.Equal "mantle-machine-artifact-registry-v1",
  prelude_path | NonEmpty,
  supported_classes | Array SurfaceClass,
  initial_cohort | StringArray,
  surfaces | Array Surface,
} in
# registry-data-begin
{
  "registry_schema"= "mantle-machine-artifact-registry-v1",
  "prelude_path"= "schemas/machine-contracts/prelude.ncl",
  "supported_classes"= [
    "contracted",
    "internal",
    "debug",
    "compatibility",
    "external"
  ],
  "initial_cohort"= [
    "operator-diagnostics.doctor-report",
    "build.build-json-report",
    "build.build-plan-report",
    "realization.route-plan-report",
    "receipt.bundle-report",
    "receipt.verify-report",
    "receipt.import-report",
    "source-bundle.plan-report",
    "source-bundle.verify-report",
    "source-bundle.self-build-hydration-report",
    "bootstrap.full-source-provider-admission-report",
    "self-build.hydrated-fresh-clone-fixed-point-report",
    "source-bundle.offline-preflight-report",
    "nickel-export.report",
    "nickel-export.receipt",
    "attestation.release-envelope",
    "oci.export-report",
    "oci.import-report",
    "oci.registry-push-report",
    "oci.registry-pull-report",
    "release.function-address-binding",
    "release.source-review-attachment",
    "mantlepkgs.package-impact-report",
    "mantlepkgs.update-plan"
  ],
  "surfaces"= [
    {
      "id"= "release.source-review-attachment",
      "class"= "contracted",
      "rust_owner"= "crates/crunch-release-core/src/source_review.rs::SourceReviewAttachment",
      "producer"= {
        "command_or_api"= "external Cairn/Valence producers; consumed by mantle release create --source-review-attachment and release verify --source-review",
        "marker"= "release.source-review-attachment",
        "source_paths"= [
          "crates/crunch-release-core/src/source_review.rs",
          "src/source_review_release.rs"
        ]
      },
      "consumers"= [
        "Mantle release verification",
        "review automation"
      ],
      "artifacts"= {
        "schema"= "schemas/machine-contracts/source-review-attachment.schema.json",
        "generated_contract"= "schemas/machine-contracts/source-review-attachment.contract.ncl",
        "positive_fixtures"= [
          "tests/fixtures/source-review-attachment/attachment.valid.json"
        ],
        "negative_fixture_set"= "schemas/machine-contracts/fixtures/source-review-attachment.negatives.json"
      },
      "version_policy"= {
        "current"= "mantle.source-review.attachment.v1",
        "supported"= [
          "mantle.source-review.attachment.v1"
        ],
        "unknown_version_behavior"= "reject before source-review verification or reviewed-source policy evaluation",
        "compatibility_converter"= "",
        "compatibility_fixtures"= []
      },
      "validation_commands"= [
        "nix develop -c cargo -Zscript scripts/check-machine-schema-contracts.rs",
        "cargo test -p crunch-release-core source_review"
      ],
      "fixture_coverage"= [
        "schema",
        "version",
        "enum",
        "bounds",
        "unknown-field"
      ],
      "freshness_strategy"= "BLAKE3 binds schema, generated contract, shared prelude, fixtures, producer source, and consumer policy",
      "freshness"= {
        "schema_blake3"= "8881f2fbc89fa7d46fe00bb92ad4389b85643b6e033d126a911573df2183c4fc",
        "contract_blake3"= "bd5e73c8be1d9348c4d6e9a2ad9049a71d3f9c976ecadc746e28dda9b4edb6ff",
        "prelude_blake3"= "d48a9410c5f120d10c5d815d1c3386358ca92342c6ce9c58c62401db0e97f55e",
        "fixture_set_blake3"= "8c0fbeb7501409d9d871f79578e693670b262a87e13a7f95bd3eb0eea79b4c54",
        "producer_identity_blake3"= "fff6ab6538fd1a03611c0d730bce630898fd60912daef3d6079784ff7852bc62",
        "consumer_policy_blake3"= "7fa65ad759e4eee6d33ff8ec0f84d1fe1ac3915b9d94e40e1c87ca19c0d20103"
      },
      "non_claims"= [
        "attachment admission does not prove source correctness, review completeness, reviewer competence, build-witness quorum, or release eligibility"
      ],
      "rationale"= "Stable external review attachment consumed by release verification and reviewed-source policy."
    },
    {
      "id"= "operator-diagnostics.doctor-report",
      "class"= "contracted",
      "rust_owner"= "src/operator_diagnostics.rs::PreflightReport",
      "producer"= {
        "command_or_api"= "mantle --json doctor",
        "marker"= "operator-diagnostics.doctor-report",
        "source_paths"= [
          "src/operator_diagnostics.rs"
        ]
      },
      "consumers"= [
        "operators",
        "Cairn readiness",
        "support automation"
      ],
      "artifacts"= {
        "schema"= "schemas/machine-contracts/doctor-report.schema.json",
        "generated_contract"= "schemas/machine-contracts/doctor-report.contract.ncl",
        "positive_fixtures"= [
          "schemas/machine-contracts/fixtures/doctor-report-valid.json"
        ],
        "negative_fixture_set"= "schemas/machine-contracts/fixtures/doctor-report.negatives.json"
      },
      "version_policy"= {
        "current"= "crunch-doctor-report-v1",
        "supported"= [
          "crunch-doctor-report-v1"
        ],
        "unknown_version_behavior"= "reject before consuming checks",
        "compatibility_converter"= "",
        "compatibility_fixtures"= []
      },
      "validation_commands"= [
        "nix develop -c cargo -Zscript scripts/check-machine-schema-contracts.rs",
        "cargo test -p mantle operator_diagnostics::tests"
      ],
      "fixture_coverage"= [
        "enum",
        "redaction",
        "cross-field"
      ],
      "freshness_strategy"= "BLAKE3 binds schema, generated contract, shared prelude, fixtures, producer source, and consumer policy",
      "freshness"= {
        "schema_blake3"= "27e36c9c867910b2358b8c9520d6bb1075a7aab7744db3eff7719f6f644e640e",
        "contract_blake3"= "1e26a6fd898d7bb8091f51b15baed11d1f0779399f21bcd3f03e839ab2bf78b3",
        "prelude_blake3"= "d48a9410c5f120d10c5d815d1c3386358ca92342c6ce9c58c62401db0e97f55e",
        "fixture_set_blake3"= "b2c0eee0dfbf01929d3f4b3aca13cab3695a693053e5249ca5d79c186966b586",
        "producer_identity_blake3"= "02b2b7a1d2ac2f7a488d820236d6a85d32643b52dc9390a624a2103a03fdb4b3",
        "consumer_policy_blake3"= "9e1b4b671ba1ac277a6faee85e944fee5bc1b51ce308638f9a977b9b75606b82"
      },
      "non_claims"= [
        "shape conformance does not prove host readiness or build correctness"
      ],
      "rationale"= "Stable bounded operator report used by automated readiness consumers."
    },
    {
      "id"= "build.build-json-report",
      "class"= "contracted",
      "rust_owner"= "src/build_report.rs::BuildJsonReport",
      "producer"= {
        "command_or_api"= "mantle --json build",
        "marker"= "build.build-json-report",
        "source_paths"= [
          "src/build_report.rs",
          "src/build_failure.rs",
          "src/build_log.rs",
          "src/cache_substitution.rs",
          "src/frontend_artifact_spec.rs",
          "src/ast_grep_evidence.rs",
          "src/remote_build.rs",
          "src/remote_telemetry_export.rs",
          "src/remote_trace_context.rs",
          "crates/crunch-release-core/src/ast_grep.rs",
          "crates/crunch-build/src/scheduling.rs",
          "crates/crunch-build/src/distributed/remote_telemetry.rs"
        ]
      },
      "consumers"= [
        "build automation",
        "audit bundles",
        "Cairn evidence"
      ],
      "artifacts"= {
        "schema"= "schemas/machine-contracts/build-json-report.schema.json",
        "generated_contract"= "schemas/machine-contracts/build-json-report.contract.ncl",
        "positive_fixtures"= [
          "schemas/machine-contracts/fixtures/build-json-report.valid.json"
        ],
        "negative_fixture_set"= "schemas/machine-contracts/fixtures/build-json-report.negatives.json"
      },
      "version_policy"= {
        "current"= "crunch-build-report-v1",
        "supported"= [
          "crunch-build-report-v1"
        ],
        "unknown_version_behavior"= "reject before interpreting outcomes",
        "compatibility_converter"= "",
        "compatibility_fixtures"= []
      },
      "validation_commands"= [
        "nix develop -c cargo -Zscript scripts/check-machine-schema-contracts.rs",
        "cargo test -p mantle build_report::tests"
      ],
      "fixture_coverage"= [
        "schema",
        "enum",
        "unknown-field"
      ],
      "freshness_strategy"= "BLAKE3 binds the full report projection and all directly serialized owner modules",
      "freshness"= {
        "schema_blake3"= "71a65f05d439dd7357d00c19bf9500e674c627592e1c92b6f73eb21fa2cc0fdc",
        "contract_blake3"= "440f27f17e164a74c1c7983ace734f7cd26298c910d36c8d0cdf3059786bb3c4",
        "prelude_blake3"= "d48a9410c5f120d10c5d815d1c3386358ca92342c6ce9c58c62401db0e97f55e",
        "fixture_set_blake3"= "335d0fb384cefd4fb92e0894a7a3346265730cfec73cf4ca91a8d1e4423e57d0",
        "producer_identity_blake3"= "c3363e0d04000abdd152c4e1477f9cbc01333ce577742e656ff9096da1b8b6c4",
        "consumer_policy_blake3"= "d4d7d58e21738c707f6071b435807382c28939a76da4cbb88e9c32c6b80af72b"
      },
      "non_claims"= [
        "report conformance does not prove build success, trust, reproducibility, or release eligibility"
      ],
      "rationale"= "Primary stable automation response for concrete builds."
    },
    {
      "id"= "build.build-plan-report",
      "class"= "contracted",
      "rust_owner"= "src/build_plan.rs::BuildPlanReport",
      "producer"= {
        "command_or_api"= "mantle --json build-plan",
        "marker"= "build.build-plan-report",
        "source_paths"= [
          "src/build_plan.rs"
        ]
      },
      "consumers"= [
        "operators",
        "CI planners",
        "route reviewers"
      ],
      "artifacts"= {
        "schema"= "schemas/machine-contracts/build-plan-report.schema.json",
        "generated_contract"= "schemas/machine-contracts/build-plan-report.contract.ncl",
        "positive_fixtures"= [
          "schemas/machine-contracts/fixtures/build-plan-report.valid.json"
        ],
        "negative_fixture_set"= "schemas/machine-contracts/fixtures/build-plan-report.negatives.json"
      },
      "version_policy"= {
        "current"= "crunch-build-plan-v1",
        "supported"= [
          "crunch-build-plan-v1"
        ],
        "unknown_version_behavior"= "reject before scheduling from the plan",
        "compatibility_converter"= "",
        "compatibility_fixtures"= []
      },
      "validation_commands"= [
        "nix develop -c cargo -Zscript scripts/check-machine-schema-contracts.rs",
        "cargo test -p mantle build_plan::tests"
      ],
      "fixture_coverage"= [
        "version"
      ],
      "freshness_strategy"= "BLAKE3 binds plan owner, schema, contract, fixtures, and policy",
      "freshness"= {
        "schema_blake3"= "bb5e5fb41273e20dceb422b7dcb3e448f81fa7b262201011ba7603eae6274fbd",
        "contract_blake3"= "16ccc6644e6a2b74b1b4db20014818bb73028343e9c3da8a92a0711ed520d434",
        "prelude_blake3"= "d48a9410c5f120d10c5d815d1c3386358ca92342c6ce9c58c62401db0e97f55e",
        "fixture_set_blake3"= "576b0cc4dbed5d3c850a408c032411c180e9a0446507ec333606c25c4b15f719",
        "producer_identity_blake3"= "836479c5a54cd3d7d304d5c42e4cb017a092588b57ad3d4c0fb7df17c5a7c2b4",
        "consumer_policy_blake3"= "d9c1ccf9e8d250b9ce0b12e6b0b73a2e13db503096b5d725ac3a16e9d65061ed"
      },
      "non_claims"= [
        "a plan does not prove execution, substitution, or output correctness"
      ],
      "rationale"= "Stable dry-run planning boundary consumed before side effects."
    },
    {
      "id"= "realization.route-plan-report",
      "class"= "contracted",
      "rust_owner"= "crates/crunch-build-planning-core/src/lib.rs::RoutePlanReport",
      "producer"= {
        "command_or_api"= "plan_realization_route",
        "marker"= "realization.route-plan-report",
        "source_paths"= [
          "crates/crunch-build-planning-core/src/lib.rs",
          "src/realization_routing.rs"
        ]
      },
      "consumers"= [
        "build-plan",
        "remote routing",
        "operator diagnostics"
      ],
      "artifacts"= {
        "schema"= "schemas/machine-contracts/route-plan-report.schema.json",
        "generated_contract"= "schemas/machine-contracts/route-plan-report.contract.ncl",
        "positive_fixtures"= [
          "schemas/machine-contracts/fixtures/route-plan-report.valid.json"
        ],
        "negative_fixture_set"= "schemas/machine-contracts/fixtures/route-plan-report.negatives.json"
      },
      "version_policy"= {
        "current"= "mantle-realization-route-plan-v1",
        "supported"= [
          "mantle-realization-route-plan-v1"
        ],
        "unknown_version_behavior"= "reject before route selection is trusted",
        "compatibility_converter"= "",
        "compatibility_fixtures"= []
      },
      "validation_commands"= [
        "nix develop -c cargo -Zscript scripts/check-machine-schema-contracts.rs",
        "cargo test -p mantle realization_routing::tests"
      ],
      "fixture_coverage"= [
        "cross-field"
      ],
      "freshness_strategy"= "BLAKE3 binds routing policy projection and tie-break semantics",
      "freshness"= {
        "schema_blake3"= "4ea9f5313831926c75c0868a71e2763ee08953fc4de6f8b2281a302215d3b929",
        "contract_blake3"= "e2edd1851549b62bd82a7da7048d1510a1850d756d2a5a5b732d27d603023b9a",
        "prelude_blake3"= "d48a9410c5f120d10c5d815d1c3386358ca92342c6ce9c58c62401db0e97f55e",
        "fixture_set_blake3"= "df3e2f92ce6755726b74253e72c45001d8b7452062a03f2a9f38ada5986088d9",
        "producer_identity_blake3"= "6a9a5e18d46a039bd8109a9362642117b411d2c30081132e4aeb7c692e414b0b",
        "consumer_policy_blake3"= "7660bcc9154bd8b4e2e5f317f88b8a276523f64e31aad9c2fef0bb68d5f89dc1"
      },
      "non_claims"= [
        "route eligibility does not prove transport or artifact correctness"
      ],
      "rationale"= "Shared stable route decision embedded in build planning and remote operation."
    },
    {
      "id"= "receipt.bundle-report",
      "class"= "contracted",
      "rust_owner"= "src/portable_receipt.rs::ReceiptBundleReport",
      "producer"= {
        "command_or_api"= "mantle receipt bundle",
        "marker"= "receipt.bundle-report",
        "source_paths"= [
          "src/portable_receipt.rs"
        ]
      },
      "consumers"= [
        "portable receipt tooling",
        "Cairn evidence",
        "offline importers"
      ],
      "artifacts"= {
        "schema"= "schemas/machine-contracts/receipt-bundle-report.schema.json",
        "generated_contract"= "schemas/machine-contracts/receipt-bundle-report.contract.ncl",
        "positive_fixtures"= [
          "schemas/machine-contracts/fixtures/receipt-bundle-report.valid.json"
        ],
        "negative_fixture_set"= "schemas/machine-contracts/fixtures/receipt-bundle-report.negatives.json"
      },
      "version_policy"= {
        "current"= "mantle-build-receipt-bundle-v1",
        "supported"= [
          "mantle-build-receipt-bundle-v1"
        ],
        "unknown_version_behavior"= "reject before evidence classification",
        "compatibility_converter"= "",
        "compatibility_fixtures"= []
      },
      "validation_commands"= [
        "nix develop -c cargo -Zscript scripts/check-machine-schema-contracts.rs",
        "cargo test -p mantle portable_receipt::tests"
      ],
      "fixture_coverage"= [
        "digest"
      ],
      "freshness_strategy"= "BLAKE3 binds portable receipt report shape and evidence policy",
      "freshness"= {
        "schema_blake3"= "e6af958159a48f87e3b1a1ba21e4891091ad0ac358d405648ba5e8fd23e4867e",
        "contract_blake3"= "5b922a6b7066bce68131419ccfb95619022698e170375d2669c21d9408165129",
        "prelude_blake3"= "d48a9410c5f120d10c5d815d1c3386358ca92342c6ce9c58c62401db0e97f55e",
        "fixture_set_blake3"= "f8efd884abc9b6beffd9527db139ab53356b198331c91adb4b41259ca6684c88",
        "producer_identity_blake3"= "2933ecd451aaa0d83acdb2668a01e46045cc15d7fe5ac033527c0ca0cf3dc508",
        "consumer_policy_blake3"= "499fa604950b42de900457a05728162a807c3d38acaa39f691602bcf30d5ef06"
      },
      "non_claims"= [
        "receipt shape does not prove the truth of evidence records"
      ],
      "rationale"= "Portable evidence summary crosses machines and repositories."
    },
    {
      "id"= "receipt.verify-report",
      "class"= "contracted",
      "rust_owner"= "src/portable_receipt.rs::ReceiptVerifyReport",
      "producer"= {
        "command_or_api"= "mantle receipt verify",
        "marker"= "receipt.verify-report",
        "source_paths"= [
          "src/portable_receipt.rs"
        ]
      },
      "consumers"= [
        "receipt verifiers",
        "release policy",
        "CI"
      ],
      "artifacts"= {
        "schema"= "schemas/machine-contracts/receipt-verify-report.schema.json",
        "generated_contract"= "schemas/machine-contracts/receipt-verify-report.contract.ncl",
        "positive_fixtures"= [
          "schemas/machine-contracts/fixtures/receipt-verify-report.valid.json"
        ],
        "negative_fixture_set"= "schemas/machine-contracts/fixtures/receipt-verify-report.negatives.json"
      },
      "version_policy"= {
        "current"= "mantle-build-receipt-bundle-v1",
        "supported"= [
          "mantle-build-receipt-bundle-v1"
        ],
        "unknown_version_behavior"= "reject before trust conclusions",
        "compatibility_converter"= "",
        "compatibility_fixtures"= []
      },
      "validation_commands"= [
        "nix develop -c cargo -Zscript scripts/check-machine-schema-contracts.rs",
        "cargo test -p mantle portable_receipt::tests"
      ],
      "fixture_coverage"= [
        "digest"
      ],
      "freshness_strategy"= "BLAKE3 binds verification projection, trust fields, fixtures, and policy",
      "freshness"= {
        "schema_blake3"= "a1e0b82024f62b39d917514de7585e2e6dd9365d564a5e58feb23484b2305aec",
        "contract_blake3"= "ebdd22a938ddc93c38f7fd0d27594be7456374e379227206e7a357fe46d4a00d",
        "prelude_blake3"= "d48a9410c5f120d10c5d815d1c3386358ca92342c6ce9c58c62401db0e97f55e",
        "fixture_set_blake3"= "a9cd57d937da9107de66d87888629c51f19a07dc585beda5192baa817bf77ec9",
        "producer_identity_blake3"= "ee9f5da4f9027026164073273ff3572789838917322a914100bdd0784836eff1",
        "consumer_policy_blake3"= "c8b7b1c63ebb54254bf7cce2d855df6a7e48601089a7102d3698e757827e974e"
      },
      "non_claims"= [
        "verification report shape does not itself verify signatures or local bytes"
      ],
      "rationale"= "Stable trust result consumed by automation."
    },
    {
      "id"= "receipt.import-report",
      "class"= "contracted",
      "rust_owner"= "src/portable_receipt.rs::ReceiptImportReport",
      "producer"= {
        "command_or_api"= "mantle receipt import",
        "marker"= "receipt.import-report",
        "source_paths"= [
          "src/portable_receipt.rs"
        ]
      },
      "consumers"= [
        "offline importers",
        "receipt workflows"
      ],
      "artifacts"= {
        "schema"= "schemas/machine-contracts/receipt-import-report.schema.json",
        "generated_contract"= "schemas/machine-contracts/receipt-import-report.contract.ncl",
        "positive_fixtures"= [
          "schemas/machine-contracts/fixtures/receipt-import-report.valid.json"
        ],
        "negative_fixture_set"= "schemas/machine-contracts/fixtures/receipt-import-report.negatives.json"
      },
      "version_policy"= {
        "current"= "mantle-build-receipt-bundle-v1",
        "supported"= [
          "mantle-build-receipt-bundle-v1"
        ],
        "unknown_version_behavior"= "reject before local mutation",
        "compatibility_converter"= "",
        "compatibility_fixtures"= []
      },
      "validation_commands"= [
        "nix develop -c cargo -Zscript scripts/check-machine-schema-contracts.rs",
        "cargo test -p mantle portable_receipt::tests"
      ],
      "fixture_coverage"= [
        "cross-field"
      ],
      "freshness_strategy"= "BLAKE3 binds import result and idempotency semantics",
      "freshness"= {
        "schema_blake3"= "fa5a902ed68036aa6e1bb1472b10b845c60980cb2cb34665f6cb6625e7fb0355",
        "contract_blake3"= "acef59d7e93e39b8b3be1d791e5a3615cec92dbae6735153b0fc1aa9147be32e",
        "prelude_blake3"= "d48a9410c5f120d10c5d815d1c3386358ca92342c6ce9c58c62401db0e97f55e",
        "fixture_set_blake3"= "6788977d15737936e07f67f8cc3ee1c5fd7acbe275762801e54e680681de19d2",
        "producer_identity_blake3"= "1a99f4f76441eb434d779d3fbde604392e737873450df8b08e6db1959c24bc82",
        "consumer_policy_blake3"= "15cedc21de51570e0fa4d214d357f33de4adafa7d8674274eec0285d6f18d7fd"
      },
      "non_claims"= [
        "import report conformance does not prove imported evidence is true"
      ],
      "rationale"= "Mutation result requires explicit idempotency and count semantics."
    },
    {
      "id"= "source-bundle.plan-report",
      "class"= "contracted",
      "rust_owner"= "src/source_bundle.rs::SourceBundlePlanReport",
      "producer"= {
        "command_or_api"= "mantle source bundle plan",
        "marker"= "source-bundle.plan-report",
        "source_paths"= [
          "src/source_bundle.rs"
        ]
      },
      "consumers"= [
        "offline build planning",
        "source exporters",
        "operators"
      ],
      "artifacts"= {
        "schema"= "schemas/machine-contracts/source-bundle-plan-report.schema.json",
        "generated_contract"= "schemas/machine-contracts/source-bundle-plan-report.contract.ncl",
        "positive_fixtures"= [
          "schemas/machine-contracts/fixtures/source-bundle-plan-report.valid.json"
        ],
        "negative_fixture_set"= "schemas/machine-contracts/fixtures/source-bundle-plan-report.negatives.json"
      },
      "version_policy"= {
        "current"= "mantle-source-bundle-v1",
        "supported"= [
          "mantle-source-bundle-v1"
        ],
        "unknown_version_behavior"= "reject before source planning",
        "compatibility_converter"= "",
        "compatibility_fixtures"= []
      },
      "validation_commands"= [
        "nix develop -c cargo -Zscript scripts/check-machine-schema-contracts.rs",
        "cargo test -p mantle source_bundle::tests"
      ],
      "fixture_coverage"= [
        "bounds"
      ],
      "freshness_strategy"= "BLAKE3 binds source-plan projection and bounded record policy",
      "freshness"= {
        "schema_blake3"= "89b0785a736e8c4854c41d54c1fe75e22ba43b0cae90399d4aff1c863dc9deed",
        "contract_blake3"= "d3b07cd0d3396e6798ec94d68d37cf06a5ee35f81866016fbf1f18e0f401c7ae",
        "prelude_blake3"= "d48a9410c5f120d10c5d815d1c3386358ca92342c6ce9c58c62401db0e97f55e",
        "fixture_set_blake3"= "996d0ea5521663d8180e90e955fcba9c92b4570b8935e13b2e57af45c81260e2",
        "producer_identity_blake3"= "e8082168a1063088f7f1684ad5d3004454c1315850aa9de2c7a180450f623588",
        "consumer_policy_blake3"= "b41da07fbbd17d793f07c51876465ca6e73d83db0dc79b9beb02c8a8110b59d1"
      },
      "non_claims"= [
        "source availability shape does not prove build correctness or provenance trust"
      ],
      "rationale"= "Source plans determine offline readiness before mutation."
    },
    {
      "id"= "source-bundle.verify-report",
      "class"= "contracted",
      "rust_owner"= "src/source_bundle.rs::SourceBundleVerifyReport",
      "producer"= {
        "command_or_api"= "mantle source bundle verify",
        "marker"= "source-bundle.verify-report",
        "source_paths"= [
          "src/source_bundle.rs"
        ]
      },
      "consumers"= [
        "offline source admission",
        "CI",
        "operators"
      ],
      "artifacts"= {
        "schema"= "schemas/machine-contracts/source-bundle-verify-report.schema.json",
        "generated_contract"= "schemas/machine-contracts/source-bundle-verify-report.contract.ncl",
        "positive_fixtures"= [
          "schemas/machine-contracts/fixtures/source-bundle-verify-report.valid.json"
        ],
        "negative_fixture_set"= "schemas/machine-contracts/fixtures/source-bundle-verify-report.negatives.json"
      },
      "version_policy"= {
        "current"= "mantle-source-bundle-v1",
        "supported"= [
          "mantle-source-bundle-v1"
        ],
        "unknown_version_behavior"= "reject before readiness is accepted",
        "compatibility_converter"= "",
        "compatibility_fixtures"= []
      },
      "validation_commands"= [
        "nix develop -c cargo -Zscript scripts/check-machine-schema-contracts.rs",
        "cargo test -p mantle source_bundle::tests"
      ],
      "fixture_coverage"= [
        "cross-field"
      ],
      "freshness_strategy"= "BLAKE3 binds readiness classes and blocker arrays",
      "freshness"= {
        "schema_blake3"= "5350efbbd43f29ff26bf6ccdf64cdfff33c6cc7f17ea90252dc679a6fb2c524c",
        "contract_blake3"= "1c816681434e8114cd6c4bab4a48fd5c5f86ce7c9f33c187ab609ec73f814fd8",
        "prelude_blake3"= "d48a9410c5f120d10c5d815d1c3386358ca92342c6ce9c58c62401db0e97f55e",
        "fixture_set_blake3"= "1ad119357b7d7217396fe2f47598b14cb8d87bc105982a428e3924e7c3213c11",
        "producer_identity_blake3"= "2997f3bb460185a4c6e2bee6b7a3ed6ca089697572da8435d8ebdd7849732c33",
        "consumer_policy_blake3"= "edc96af15dd9073d7cd0178ed0d617acd41c8cf91c6719a674dd1422a7e285e4"
      },
      "non_claims"= [
        "readiness shape does not prove source provenance or downstream success"
      ],
      "rationale"= "Stable source verification decision consumed by offline workflows."
    },
    {
      "id"= "source-bundle.self-build-hydration-report",
      "class"= "contracted",
      "rust_owner"= "src/source_bundle.rs::SelfBuildHydrationReport",
      "producer"= {
        "command_or_api"= "mantle --json source bundle hydrate-self-build",
        "marker"= "source-bundle.self-build-hydration-report",
        "source_paths"= [
          "src/source_bundle.rs"
        ]
      },
      "consumers"= [
        "fresh-clone bootstrap operators",
        "CI",
        "release source preparation"
      ],
      "artifacts"= {
        "schema"= "schemas/machine-contracts/self-build-source-hydration-report.schema.json",
        "generated_contract"= "schemas/machine-contracts/self-build-source-hydration-report.contract.ncl",
        "positive_fixtures"= [
          "schemas/machine-contracts/fixtures/self-build-source-hydration-report.valid.json"
        ],
        "negative_fixture_set"= "schemas/machine-contracts/fixtures/self-build-source-hydration-report.negatives.json"
      },
      "version_policy"= {
        "current"= "mantle-self-build-source-hydration-v1",
        "supported"= [
          "mantle-self-build-source-hydration-v1"
        ],
        "unknown_version_behavior"= "reject before checkout or source-state mutation",
        "compatibility_converter"= "",
        "compatibility_fixtures"= []
      },
      "validation_commands"= [
        "nix develop -c cargo -Zscript scripts/check-machine-schema-contracts.rs",
        "cargo test -p mantle --bin mantle self_build_hydration"
      ],
      "fixture_coverage"= [
        "version"
      ],
      "freshness_strategy"= "BLAKE3 binds bundle, vendor, provider, producer, and consumer policy identities",
      "freshness"= {
        "schema_blake3"= "bad5e9dc0552bbfcbab86fe6ca835ac3b00b3754e58167c6c33a2e7375c531b6",
        "contract_blake3"= "953be8500e8110bc2682b8d4aa14136c274f675dc951126f0b6a165f062c5bd3",
        "prelude_blake3"= "d48a9410c5f120d10c5d815d1c3386358ca92342c6ce9c58c62401db0e97f55e",
        "fixture_set_blake3"= "cad5f96d2366110d1a790d5ab644163380b9af4f0a5b2be947ceaa39ee03fbd6",
        "producer_identity_blake3"= "d6a9db2809aa09c8f1c88bcbe67457e461749628617fa420db689cb59b457512",
        "consumer_policy_blake3"= "8af50ac86145048afc879ae1b6722e155ee6960c6261186014bb505db83b6cb8"
      },
      "non_claims"= [
        "hydrated source availability does not prove fixed-point self-build success",
        "bundle identity does not prove compiler correctness, seed trust removal, or release reproducibility"
      ],
      "rationale"= "Fresh-clone mutation requires a contracted identity-bound success receipt."
    },
    {
      "id"= "bootstrap.full-source-provider-admission-report",
      "class"= "contracted",
      "rust_owner"= "src/full_source_provider.rs::FullSourceProviderAdmissionReport",
      "producer"= {
        "command_or_api"= "mantle bootstrap full-source-provider-admit --provider-dir ... --expected-output-blake3 ... --source-closure ... --expected-source-closure-blake3 ... --report ...",
        "marker"= "bootstrap.full-source-provider-admission-report",
        "source_paths"= [
          "src/full_source_provider.rs"
        ]
      },
      "consumers"= [
        "full-source bootstrap operators",
        "self-hosting proof bundles",
        "Cairn evidence"
      ],
      "artifacts"= {
        "schema"= "schemas/machine-contracts/full-source-provider-admission-report.schema.json",
        "generated_contract"= "schemas/machine-contracts/full-source-provider-admission-report.contract.ncl",
        "positive_fixtures"= [
          "schemas/machine-contracts/fixtures/full-source-provider-admission-report.valid.json"
        ],
        "negative_fixture_set"= "schemas/machine-contracts/fixtures/full-source-provider-admission-report.negatives.json"
      },
      "version_policy"= {
        "current"= "mantle-full-source-provider-admission-v2",
        "supported"= [
          "mantle-full-source-provider-admission-v2"
        ],
        "unknown_version_behavior"= "reject before selecting or citing the provider",
        "compatibility_converter"= "",
        "compatibility_fixtures"= []
      },
      "validation_commands"= [
        "nix develop -c cargo -Zscript scripts/check-machine-schema-contracts.rs",
        "nix develop -c cargo test -p mantle --bin mantle full_source_provider::tests",
        "nix develop -c cargo test -p mantle --test self_hosting -- --skip self_hosting_stage0_stage1_stage2"
      ],
      "fixture_coverage"= [
        "version",
        "bounds"
      ],
      "freshness_strategy"= "BLAKE3 binds provider output, provider metadata, complete source closure, producer, and consumer policy identities",
      "freshness"= {
        "schema_blake3"= "e9f1440954ebe8f5e56eb4ab7ea6f2a0f3985e87d3954de9bcbe7720e667ed4d",
        "contract_blake3"= "dca78b427a831e803eac4c9d94f2f209ad93cedf041941e2cd688cf1ac8c34f2",
        "prelude_blake3"= "d48a9410c5f120d10c5d815d1c3386358ca92342c6ce9c58c62401db0e97f55e",
        "fixture_set_blake3"= "18dbdaece5c99497cbd663230682e0b5bce9bf0fcb22a0e2a0664377c445ff0f",
        "producer_identity_blake3"= "cd67b384e9acb8125bdde0fe1149ba6c0296190091d72b6149b24eb9a246b6ca",
        "consumer_policy_blake3"= "aa2b9979ef6f03d98bbe0861e19fd9ab0c88af64d6b803fce04789e58904e123"
      },
      "non_claims"= [
        "provider admission does not prove compiler correctness or bootstrap-seed correctness",
        "one provider build does not prove independent rebuild agreement, release reproducibility, deployment success, or full Cargo compatibility"
      ],
      "rationale"= "Bootstrap authority changes require a contracted runtime-tested provider and source/output identity receipt."
    },
    {
      "id"= "self-build.hydrated-fresh-clone-fixed-point-report",
      "class"= "contracted",
      "rust_owner"= "src/fresh_clone_fixed_point.rs::HydratedFreshCloneFixedPointReport",
      "producer"= {
        "command_or_api"= "scripts/prove-self-hosting.sh --source-state ... --source-manifest-blake3 ... --hydration-report ...",
        "marker"= "self-build.hydrated-fresh-clone-fixed-point-report",
        "source_paths"= [
          "src/fresh_clone_fixed_point.rs"
        ]
      },
      "consumers"= [
        "fresh-clone bootstrap operators",
        "release source preparation",
        "Cairn evidence"
      ],
      "artifacts"= {
        "schema"= "schemas/machine-contracts/hydrated-fresh-clone-fixed-point-report.schema.json",
        "generated_contract"= "schemas/machine-contracts/hydrated-fresh-clone-fixed-point-report.contract.ncl",
        "positive_fixtures"= [
          "schemas/machine-contracts/fixtures/hydrated-fresh-clone-fixed-point-report.valid.json"
        ],
        "negative_fixture_set"= "schemas/machine-contracts/fixtures/hydrated-fresh-clone-fixed-point-report.negatives.json"
      },
      "version_policy"= {
        "current"= "mantle-hydrated-fresh-clone-fixed-point-v1",
        "supported"= [
          "mantle-hydrated-fresh-clone-fixed-point-v1"
        ],
        "unknown_version_behavior"= "reject before admitting or citing fixed-point success",
        "compatibility_converter"= "",
        "compatibility_fixtures"= []
      },
      "validation_commands"= [
        "nix develop -c cargo -Zscript scripts/check-machine-schema-contracts.rs",
        "cargo test -p mantle fresh_clone_fixed_point::tests",
        "cargo test -p mantle --test self_hosting -- --skip self_hosting_stage0_stage1_stage2"
      ],
      "fixture_coverage"= [
        "version",
        "bounds"
      ],
      "freshness_strategy"= "BLAKE3 binds source authority, hydration linkage, stage policy, fixed-point binaries, producer, and consumer policy",
      "freshness"= {
        "schema_blake3"= "7b713d5f7c12350a9ce62605b11d04c56c2e4a13ad4696d4596ccc05ffac3c12",
        "contract_blake3"= "80d0cbf9eba66de7deac1d2248d4d34ce7d6cc84ca4498baa3647cf12f9acc19",
        "prelude_blake3"= "d48a9410c5f120d10c5d815d1c3386358ca92342c6ce9c58c62401db0e97f55e",
        "fixture_set_blake3"= "1270ca65f6528bae5f8ecd3510bd43f5891ad31bd4931e001bdf5fdf33259027",
        "producer_identity_blake3"= "b626cd01696185971961ca75eb9a946837a939212d4b192f6acb1bb3a203049d",
        "consumer_policy_blake3"= "cf90ceab2c26008ceb2d2c66ea079113c0f6c989aa8600d4c5d0f01177ac605c"
      },
      "non_claims"= [
        "one hydrated fixed point does not prove compiler correctness or bootstrap-seed correctness",
        "shape conformance does not prove source trust, independent rebuild agreement, release reproducibility, deployment success, or full Cargo compatibility"
      ],
      "rationale"= "The source-authority and binary-equality claim needs a path-redacted contracted success report."
    },
    {
      "id"= "source-bundle.offline-preflight-report",
      "class"= "contracted",
      "rust_owner"= "src/source_bundle.rs::SourceOfflinePreflightReport",
      "producer"= {
        "command_or_api"= "offline_source_preflight",
        "marker"= "source-bundle.offline-preflight-report",
        "source_paths"= [
          "src/source_bundle.rs"
        ]
      },
      "consumers"= [
        "build-plan",
        "offline builders",
        "operator remediation"
      ],
      "artifacts"= {
        "schema"= "schemas/machine-contracts/source-offline-preflight-report.schema.json",
        "generated_contract"= "schemas/machine-contracts/source-offline-preflight-report.contract.ncl",
        "positive_fixtures"= [
          "schemas/machine-contracts/fixtures/source-offline-preflight-report.valid.json"
        ],
        "negative_fixture_set"= "schemas/machine-contracts/fixtures/source-offline-preflight-report.negatives.json"
      },
      "version_policy"= {
        "current"= "mantle-source-offline-preflight-v1",
        "supported"= [
          "mantle-source-offline-preflight-v1"
        ],
        "unknown_version_behavior"= "reject before an offline build starts",
        "compatibility_converter"= "",
        "compatibility_fixtures"= []
      },
      "validation_commands"= [
        "nix develop -c cargo -Zscript scripts/check-machine-schema-contracts.rs",
        "cargo test -p mantle source_bundle::tests"
      ],
      "fixture_coverage"= [
        "reference"
      ],
      "freshness_strategy"= "BLAKE3 binds preflight state, remediation, and source summaries",
      "freshness"= {
        "schema_blake3"= "40aaec6972d156951612eee123c5878b829f7a822a5eb8a4a3dc32c23dd7c059",
        "contract_blake3"= "d7b30efe0e40442383eb7c654a3bb0c88b508f059b88008085fd59f0a88b0f23",
        "prelude_blake3"= "d48a9410c5f120d10c5d815d1c3386358ca92342c6ce9c58c62401db0e97f55e",
        "fixture_set_blake3"= "74b65baffbfcfb59d5557a99ea0ed63c02711ee4c865c5f7be1eb14ad57071c8",
        "producer_identity_blake3"= "32ef83b17f1c86c53abd5c46737adf87b01ea3546963e23598a69392a9a445f2",
        "consumer_policy_blake3"= "2604bdd4cb6aa3824e00fa515e64f68a01ab5df84d2f2342ee0f1e7029f91f07"
      },
      "non_claims"= [
        "offline readiness does not prove a build or source trust"
      ],
      "rationale"= "Preflight result is directly embedded into build route decisions."
    },
    {
      "id"= "nickel-export.report",
      "class"= "contracted",
      "rust_owner"= "src/nickel_export.rs::NickelExportReport",
      "producer"= {
        "command_or_api"= "mantle nickel export --json",
        "marker"= "nickel-export.report",
        "source_paths"= [
          "src/nickel_export.rs"
        ]
      },
      "consumers"= [
        "frontend adapters",
        "CI",
        "export tooling"
      ],
      "artifacts"= {
        "schema"= "schemas/machine-contracts/nickel-export-report.schema.json",
        "generated_contract"= "schemas/machine-contracts/nickel-export-report.contract.ncl",
        "positive_fixtures"= [
          "schemas/machine-contracts/fixtures/nickel-export-report.valid.json"
        ],
        "negative_fixture_set"= "schemas/machine-contracts/fixtures/nickel-export-report.negatives.json"
      },
      "version_policy"= {
        "current"= "mantle-nickel-export-report-v1",
        "supported"= [
          "mantle-nickel-export-report-v1"
        ],
        "unknown_version_behavior"= "reject before consuming export status",
        "compatibility_converter"= "",
        "compatibility_fixtures"= []
      },
      "validation_commands"= [
        "nix develop -c cargo -Zscript scripts/check-machine-schema-contracts.rs",
        "cargo test -p mantle nickel_export::tests"
      ],
      "fixture_coverage"= [
        "cross-field"
      ],
      "freshness_strategy"= "BLAKE3 binds export status, receipt projection, diagnostics, and policy",
      "freshness"= {
        "schema_blake3"= "c42a0b70093997a3d741882d93be44f43e732e59a6ecbfee232bb86feceef564",
        "contract_blake3"= "c476dbe914a257fd0f752bd7036ceb9e3361898761a025d77c191654bc6065e0",
        "prelude_blake3"= "d48a9410c5f120d10c5d815d1c3386358ca92342c6ce9c58c62401db0e97f55e",
        "fixture_set_blake3"= "3cf4824049a7e2294fe7aa79115d67f79739889171e3e022cb7c07da622af610",
        "producer_identity_blake3"= "dc6191652ea87147ab0ad393819debc5d79d18a876f22c06390690a4e3504a70",
        "consumer_policy_blake3"= "197e7e60804e37ca2cbb8548b889953f4392802d4f04d0f1a56fc5d346528442"
      },
      "non_claims"= [
        "export success does not prove deployability, frontend correctness, or build success"
      ],
      "rationale"= "Stable machine response for Nickel-to-JSON exports."
    },
    {
      "id"= "nickel-export.receipt",
      "class"= "contracted",
      "rust_owner"= "src/nickel_export.rs::NickelExportReceipt",
      "producer"= {
        "command_or_api"= "NickelExportReport.receipt",
        "marker"= "nickel-export.receipt",
        "source_paths"= [
          "src/nickel_export.rs",
          "src/nickel_export_core_adapter.rs"
        ]
      },
      "consumers"= [
        "export verifiers",
        "frontend adapters",
        "audit tooling"
      ],
      "artifacts"= {
        "schema"= "schemas/machine-contracts/nickel-export-receipt.schema.json",
        "generated_contract"= "schemas/machine-contracts/nickel-export-receipt.contract.ncl",
        "positive_fixtures"= [
          "schemas/machine-contracts/fixtures/nickel-export-receipt.valid.json"
        ],
        "negative_fixture_set"= "schemas/machine-contracts/fixtures/nickel-export-receipt.negatives.json"
      },
      "version_policy"= {
        "current"= "mantle-nickel-export-receipt-v1",
        "supported"= [
          "mantle-nickel-export-receipt-v1"
        ],
        "unknown_version_behavior"= "reject before digest linkage is consumed",
        "compatibility_converter"= "",
        "compatibility_fixtures"= []
      },
      "validation_commands"= [
        "nix develop -c cargo -Zscript scripts/check-machine-schema-contracts.rs",
        "cargo test -p mantle nickel_export::tests"
      ],
      "fixture_coverage"= [
        "unknown-field"
      ],
      "freshness_strategy"= "BLAKE3 binds source references, evaluator identity, output digest, and non-claim",
      "freshness"= {
        "schema_blake3"= "5f4fe9ff3df4c34fe20480caaaa0e810ec6715b543c740173a9fe4f100b472d4",
        "contract_blake3"= "3810c6aaa46851ddef9d4288ebb83e6fca65fea69b82c45ab4d19b9614dddfbb",
        "prelude_blake3"= "d48a9410c5f120d10c5d815d1c3386358ca92342c6ce9c58c62401db0e97f55e",
        "fixture_set_blake3"= "e39044713ad33d6d6153018742b25fa6b1da7c3b5b013537854684078292a2c8",
        "producer_identity_blake3"= "59c5fab4fac7c0a1b74a8cb9cd76ba7a26a14d5e2843048a4e36ea38fe32ce36",
        "consumer_policy_blake3"= "dc57ec430384237cfec878c7eb95770c629a3afa911374a8de998780d26b0842"
      },
      "non_claims"= [
        "receipt linkage does not establish evaluator or output correctness"
      ],
      "rationale"= "Cross-repository export handoff needs explicit source and evaluator linkage."
    },
    {
      "id"= "attestation.release-envelope",
      "class"= "contracted",
      "rust_owner"= "src/attest_cmd.rs::AttestationEnvelopeOutput",
      "producer"= {
        "command_or_api"= "mantle attest release-show",
        "marker"= "attestation.release-envelope",
        "source_paths"= [
          "src/attest_cmd.rs",
          "crates/crunch-attestation-core/src/release.rs"
        ]
      },
      "consumers"= [
        "release verifiers",
        "witness handoff",
        "Cairn release evidence"
      ],
      "artifacts"= {
        "schema"= "schemas/machine-contracts/release-attestation-envelope.schema.json",
        "generated_contract"= "schemas/machine-contracts/release-attestation-envelope.contract.ncl",
        "positive_fixtures"= [
          "schemas/machine-contracts/fixtures/release-attestation-envelope.valid.json",
          "schemas/machine-contracts/fixtures/release-attestation-envelope-without-stored-path.valid.json"
        ],
        "negative_fixture_set"= "schemas/machine-contracts/fixtures/release-attestation-envelope.negatives.json"
      },
      "version_policy"= {
        "current"= "mantle-release-attestation-v1",
        "supported"= [
          "mantle-release-attestation-v1"
        ],
        "unknown_version_behavior"= "reject before release or witness policy evaluation",
        "compatibility_converter"= "",
        "compatibility_fixtures"= []
      },
      "validation_commands"= [
        "nix develop -c cargo -Zscript scripts/check-machine-schema-contracts.rs",
        "cargo test -p mantle attest_cmd::tests"
      ],
      "fixture_coverage"= [
        "digest"
      ],
      "freshness_strategy"= "BLAKE3 binds envelope, release attestation owner, fixture, contract, and consumer trust policy",
      "freshness"= {
        "schema_blake3"= "97d72f5b5229fedff1c013230edc431c675f22bc7bf404f677f9db30311db932",
        "contract_blake3"= "45a5be6ff94dc2b3fe1ae248e6074bce475bdcc72cf42951d6b8d89936fd5431",
        "prelude_blake3"= "d48a9410c5f120d10c5d815d1c3386358ca92342c6ce9c58c62401db0e97f55e",
        "fixture_set_blake3"= "67d9e9232706f73f157c68abea93383ffa5af8ef83a924ecc2fb8fadce10a357",
        "producer_identity_blake3"= "5b13b275eaf4d012f96f4994d44d161ebd376c4139ae45910277eaea692cad38",
        "consumer_policy_blake3"= "fce9cfc511ed3931f6804f5e99cd0ae3d6d325a2a6e47899b8f027ccaa6787ec"
      },
      "non_claims"= [
        "shape and digest syntax do not prove attestation truth, signer trust, reproducibility, or release eligibility"
      ],
      "rationale"= "Selected stable release handoff envelope crosses publisher, witness, and verifier boundaries."
    },
    {
      "id"= "oci.export-report",
      "class"= "contracted",
      "rust_owner"= "src/oci_projection.rs::OciExportReport",
      "producer"= {
        "command_or_api"= "mantle artifact export --destination oci-layout",
        "marker"= "oci.export-report",
        "source_paths"= [
          "src/oci_projection.rs",
          "src/oci_projection/archive.rs",
          "src/oci_projection/layout.rs",
          "src/oci_projection/tests.rs",
          "src/oci_projection_shell.rs"
        ]
      },
      "consumers"= [
        "local OCI layout tooling",
        "frontend adapters",
        "audit and handoff tooling"
      ],
      "artifacts"= {
        "schema"= "schemas/machine-contracts/oci-export-report.schema.json",
        "generated_contract"= "schemas/machine-contracts/oci-export-report.contract.ncl",
        "positive_fixtures"= [
          "tests/fixtures/kernel-bundle-oci/mantle-export-minimal.json",
          "tests/fixtures/kernel-bundle-oci/mantle-export-full.json"
        ],
        "negative_fixture_set"= "schemas/machine-contracts/fixtures/oci-export-report.negatives.json"
      },
      "version_policy"= {
        "current"= "mantle-oci-export-report-v1",
        "supported"= [
          "mantle-oci-export-report-v1"
        ],
        "unknown_version_behavior"= "reject before report verification or OCI layout handoff",
        "compatibility_converter"= "",
        "compatibility_fixtures"= []
      },
      "validation_commands"= [
        "nix develop -c cargo -Zscript scripts/check-machine-schema-contracts.rs",
        "cargo test -p mantle oci_projection::tests::golden_reports_cover_full_minimal_and_both_import_states"
      ],
      "fixture_coverage"= [
        "version",
        "enum",
        "digest",
        "reference",
        "bounds",
        "unknown-field",
        "redaction",
        "cross-field"
      ],
      "freshness_strategy"= "BLAKE3 binds the frontend-neutral Rust report owner, exact digest-role schema, generated contract, reviewed fixtures, and consumer non-claims",
      "freshness"= {
        "schema_blake3"= "0db24b0f3f4952c80b074eed1d7ff8e03d3685a57deeb51ad2d318d31f27de85",
        "contract_blake3"= "43a06bea0e9950c5d9e4e1ee7f84eaa3d716f8386d02003f655dda393707f644",
        "prelude_blake3"= "d48a9410c5f120d10c5d815d1c3386358ca92342c6ce9c58c62401db0e97f55e",
        "fixture_set_blake3"= "cce0055d42d2e754649fabbedcbee44224f948b72e7d5f1adb3d0e24787674c2",
        "producer_identity_blake3"= "fbdb0dd5d1ea1e9d2df6a3f313b143f95fe81af09d28362328b63c9f53719268",
        "consumer_policy_blake3"= "d2045dba8cfd1dea820752fb60f94beb4cd88674ba02fabca73b2fbe53ffb251"
      },
      "non_claims"= [
        "report conformance does not prove registry publication, frontend semantics, kernel compatibility, bootability, deployability, signature policy, or release eligibility"
      ],
      "rationale"= "The stable local OCI export report crosses a generic build and transport boundary while frontend meaning remains external."
    },
    {
      "id"= "oci.import-report",
      "class"= "contracted",
      "rust_owner"= "src/oci_projection.rs::OciImportReport",
      "producer"= {
        "command_or_api"= "mantle artifact import --source oci-layout",
        "marker"= "oci.import-report",
        "source_paths"= [
          "src/oci_projection.rs",
          "src/oci_projection/archive.rs",
          "src/oci_projection/layout.rs",
          "src/oci_projection/tests.rs",
          "src/oci_projection_shell.rs"
        ]
      },
      "consumers"= [
        "local OCI layout tooling",
        "frontend reconstruction adapters",
        "audit and handoff tooling"
      ],
      "artifacts"= {
        "schema"= "schemas/machine-contracts/oci-import-report.schema.json",
        "generated_contract"= "schemas/machine-contracts/oci-import-report.contract.ncl",
        "positive_fixtures"= [
          "tests/fixtures/kernel-bundle-oci/mantle-import-admitted.json",
          "tests/fixtures/kernel-bundle-oci/mantle-import-compatibility-only.json"
        ],
        "negative_fixture_set"= "schemas/machine-contracts/fixtures/oci-import-report.negatives.json"
      },
      "version_policy"= {
        "current"= "mantle-oci-import-report-v1",
        "supported"= [
          "mantle-oci-import-report-v1"
        ],
        "unknown_version_behavior"= "reject before CAS admission or frontend reconstruction handoff",
        "compatibility_converter"= "",
        "compatibility_fixtures"= []
      },
      "validation_commands"= [
        "nix develop -c cargo -Zscript scripts/check-machine-schema-contracts.rs",
        "cargo test -p mantle oci_projection::tests::golden_reports_cover_full_minimal_and_both_import_states"
      ],
      "fixture_coverage"= [
        "version",
        "enum",
        "digest",
        "reference",
        "bounds",
        "unknown-field",
        "redaction"
      ],
      "freshness_strategy"= "BLAKE3 binds admitted and compatibility-only Rust report projections, OCI SHA-256 roles, Mantle BLAKE3 identities, fixtures, and consumer non-claims",
      "freshness"= {
        "schema_blake3"= "6cebc78ec0db150ffa5d5511ca0a562b7d0033e48c83a6c85f2137fe373e70f3",
        "contract_blake3"= "ce21d8ae6b38624e8cda66d54822e5d0eef0ba0328b92461174f748b17b26c21",
        "prelude_blake3"= "d48a9410c5f120d10c5d815d1c3386358ca92342c6ce9c58c62401db0e97f55e",
        "fixture_set_blake3"= "254a926114d7891231e339f4ad017e40ea01877eb69a1cc8c579eb3021c9c7aa",
        "producer_identity_blake3"= "12d0961e508f818740c474788638306dfd988b176946277d7a6af1fe80602d1a",
        "consumer_policy_blake3"= "d9734a8c052a4e80a65917ca0a08b68273017dfe057ab8f7bbd59d423b66d963"
      },
      "non_claims"= [
        "report conformance does not recover frontend semantics or prove signature trust, kernel compatibility, bootability, deployability, publication, or release eligibility"
      ],
      "rationale"= "The stable import report distinguishes admitted and compatibility-only layouts without granting frontend semantic authority to Mantle."
    },
    {
      "id"= "oci.registry-push-report",
      "class"= "contracted",
      "rust_owner"= "src/oci_registry.rs::OciRegistryPushReport",
      "producer"= {
        "command_or_api"= "mantle --json artifact oci-push",
        "marker"= "oci.registry-push-report",
        "source_paths"= [
          "src/oci_registry.rs",
          "src/oci_registry_shell.rs",
          "src/artifact_cmd.rs",
          "src/main.rs"
        ]
      },
      "consumers"= [
        "registry publication operators",
        "subsequent immutable OCI pulls",
        "audit and handoff tooling"
      ],
      "artifacts"= {
        "schema"= "schemas/machine-contracts/oci-registry-push-report.schema.json",
        "generated_contract"= "schemas/machine-contracts/oci-registry-push-report.contract.ncl",
        "positive_fixtures"= [
          "tests/fixtures/kernel-bundle-oci/registry-push-report.json"
        ],
        "negative_fixture_set"= "schemas/machine-contracts/fixtures/oci-registry-push-report.negatives.json"
      },
      "version_policy"= {
        "current"= "mantle-oci-registry-push-report-v2",
        "supported"= [
          "mantle-oci-registry-push-report-v2"
        ],
        "unknown_version_behavior"= "reject before using publication facts for pull or audit",
        "compatibility_converter"= "",
        "compatibility_fixtures"= []
      },
      "validation_commands"= [
        "nix develop -c cargo -Zscript scripts/check-machine-schema-contracts.rs",
        "nix develop -c cargo test -p mantle --lib oci_registry::tests::registry_report_golden_fixtures_match_rust_serialization"
      ],
      "fixture_coverage"= [
        "version",
        "digest",
        "enum",
        "bounds",
        "redaction",
        "unknown-field"
      ],
      "freshness_strategy"= "BLAKE3 binds the Rust push report, three immutable OCI manifest identities, verified signer/key evidence, local policy identity, generated contract, reviewed fixtures, credential-mode policy, and consumer non-claims",
      "freshness"= {
        "schema_blake3"= "1a8f2b78efd9676b0011370651b0f0dbe4eb4efba1a276c7276c88f1a7b0b62d",
        "contract_blake3"= "d158d85e21edcccbf6d6ae7bc6d3d59aea4acb8f1dc8a52438be38567d4c50f2",
        "prelude_blake3"= "d48a9410c5f120d10c5d815d1c3386358ca92342c6ce9c58c62401db0e97f55e",
        "fixture_set_blake3"= "230228d02be1df38f427caa0f6ca4cd7c1ec8666c2350508d409aef177ccd494",
        "producer_identity_blake3"= "85afd9299e1a7f14e45d6ec487fba0e03ad60f4af90c2485bff7ab247b6f71c0",
        "consumer_policy_blake3"= "c81ddbc9befcf012a5b42b3f44c2f17fe44f890ce0dfcc228b2cfefd5eb30dd4"
      },
      "non_claims"= [
        "report conformance proves neither registry authorization, transparency, revocation freshness, tag immutability, exactly-once publication, upload resumption, kernel compatibility, bootability, deployability, nor release eligibility; signature evidence authenticates only the digest pair under supplied local policy"
      ],
      "rationale"= "The push receipt is the immutable operator handoff from local admitted layout identity to three registry manifests and bounded signature-policy evidence."
    },
    {
      "id"= "oci.registry-pull-report",
      "class"= "contracted",
      "rust_owner"= "src/oci_registry.rs::OciRegistryPullReport",
      "producer"= {
        "command_or_api"= "mantle --json artifact oci-pull",
        "marker"= "oci.registry-pull-report",
        "source_paths"= [
          "src/oci_registry.rs",
          "src/oci_registry_shell.rs",
          "src/artifact_cmd.rs",
          "src/main.rs"
        ]
      },
      "consumers"= [
        "registry pull operators",
        "frontend reconstruction adapters",
        "audit and handoff tooling"
      ],
      "artifacts"= {
        "schema"= "schemas/machine-contracts/oci-registry-pull-report.schema.json",
        "generated_contract"= "schemas/machine-contracts/oci-registry-pull-report.contract.ncl",
        "positive_fixtures"= [
          "tests/fixtures/kernel-bundle-oci/registry-pull-report.json"
        ],
        "negative_fixture_set"= "schemas/machine-contracts/fixtures/oci-registry-pull-report.negatives.json"
      },
      "version_policy"= {
        "current"= "mantle-oci-registry-pull-report-v2",
        "supported"= [
          "mantle-oci-registry-pull-report-v2"
        ],
        "unknown_version_behavior"= "reject before treating registry bytes as an admitted local OCI handoff",
        "compatibility_converter"= "",
        "compatibility_fixtures"= []
      },
      "validation_commands"= [
        "nix develop -c cargo -Zscript scripts/check-machine-schema-contracts.rs",
        "nix develop -c cargo test -p mantle --lib oci_registry::tests::registry_report_golden_fixtures_match_rust_serialization"
      ],
      "fixture_coverage"= [
        "version",
        "digest",
        "enum",
        "bounds",
        "unknown-field"
      ],
      "freshness_strategy"= "BLAKE3 binds the Rust pull report, three immutable manifest expectations, verified signer/key and local policy evidence, local admission linkage, generated contract, reviewed fixtures, and consumer non-claims",
      "freshness"= {
        "schema_blake3"= "2eeaa551333a6fe6f7e044c7b2f49e115cebfc2c0d739f726204616c4b0c2840",
        "contract_blake3"= "723027e965189230cabb05b8ba069effec984e9bd527625a4320c8bd3cbe1af3",
        "prelude_blake3"= "d48a9410c5f120d10c5d815d1c3386358ca92342c6ce9c58c62401db0e97f55e",
        "fixture_set_blake3"= "9acb57b93f3b9150f79d59ff8a46ca3b5c2e5b98e83320b0308d57334a13d4a3",
        "producer_identity_blake3"= "a9b5e95c15994afc49a699936030209e1d91ec8e386daf04ec2674999984829b",
        "consumer_policy_blake3"= "e81324a18c19a70aaa2bec634d4a0b7dccceede6f048bbd332b432a9938adefe"
      },
      "non_claims"= [
        "report conformance proves neither registry authorization, transparency, revocation freshness, tag immutability, arbitrary-registry compatibility, kernel compatibility, bootability, deployability, publication atomicity, nor release eligibility; signature evidence authenticates only the digest pair under supplied local policy"
      ],
      "rationale"= "The pull receipt binds mutable registry routing to three expected manifest identities, bounded signature-policy evidence, and the ordinary admitted OCI import receipt."
    },
    {
      "id"= "release.function-address-binding",
      "class"= "contracted",
      "rust_owner"= "crates/crunch-release-core/src/function_address_binding.rs::FunctionAddressBindingReceipt",
      "producer"= {
        "command_or_api"= "mantle --json release function-address-bind",
        "marker"= "release.function-address-binding",
        "source_paths"= [
          "crates/crunch-release-core/src/function_address_binding.rs",
          "crates/crunch-release-core/src/manifest.rs",
          "crates/crunch-release-core/src/opaque_evidence.rs",
          "src/function_address_binding_cmd.rs"
        ]
      },
      "consumers"= [
        "Cairn release readiness",
        "Mantle release binding CLI",
        "release audit tooling"
      ],
      "artifacts"= {
        "schema"= "schemas/machine-contracts/function-address-binding.schema.json",
        "generated_contract"= "schemas/machine-contracts/function-address-binding.contract.ncl",
        "positive_fixtures"= [
          "tests/fixtures/function-address-release-binding/mantle-binding.valid.json",
          "tests/fixtures/function-address-preserves-sidecars/mantle-binding.valid.json"
        ],
        "negative_fixture_set"= "schemas/machine-contracts/fixtures/function-address-binding.negatives.json"
      },
      "version_policy"= {
        "current"= "mantle.function-address-binding.v1",
        "supported"= [
          "mantle.function-address-binding.v1"
        ],
        "unknown_version_behavior"= "reject before function-address readiness or release-policy consumption",
        "compatibility_converter"= "",
        "compatibility_fixtures"= []
      },
      "validation_commands"= [
        "nix develop -c cargo -Zscript scripts/check-machine-schema-contracts.rs",
        "cargo test -p crunch-release-core function_address_binding",
        "cairn release-readiness --function-address-mantle <binding.json>"
      ],
      "fixture_coverage"= [
        "version",
        "schema",
        "digest",
        "enum",
        "cross-field"
      ],
      "freshness_strategy"= "BLAKE3 binds the Rust receipt owner, exact linkage schema, generated contract, adversarial fixtures, verification summary, and Cairn-facing non-claims",
      "freshness"= {
        "schema_blake3"= "d6590e3214fc24f4036c8acd4f33833b20f0f87402fa0f4687cd012f1df3e1c0",
        "contract_blake3"= "ca290b374f6b2987e6bd56f541a4ac2539df2fcc55878eb9572e25734783f16f",
        "prelude_blake3"= "d48a9410c5f120d10c5d815d1c3386358ca92342c6ce9c58c62401db0e97f55e",
        "fixture_set_blake3"= "3d447dccbcf05cc8bdf1875896266084327a7ae352416e319d7408125dd9cd41",
        "producer_identity_blake3"= "3811492d59f709d8733ea4faec739115f87d50d436558eb0679119dfd619a02a",
        "consumer_policy_blake3"= "f1fe25ad7bf31510a0d970cabe8f0eea37b17342b7743bcd39c3858a4e5adb0a"
      },
      "non_claims"= [
        "binding conformance proves typed identity and linkage plus supplied lifecycle-policy facts only; it does not prove Rust semantics, compiler correctness, build correctness, verifier soundness, or release eligibility"
      ],
      "rationale"= "The direct Mantle-to-Cairn release boundary requires a versioned receipt rather than smoke-specific wrapper fields."
    },
    {
      "id"= "cli.error-envelope",
      "class"= "compatibility",
      "rust_owner"= "src/errors.rs::RunError",
      "producer"= {
        "command_or_api"= "mantle --json command failures",
        "marker"= "cli.error-envelope",
        "source_paths"= [
          "src/errors.rs",
          "src/main.rs"
        ]
      },
      "consumers"= [
        "CLI automation"
      ],
      "artifacts"= {
        "schema"= "",
        "generated_contract"= "",
        "positive_fixtures"= [],
        "negative_fixture_set"= ""
      },
      "version_policy"= {
        "current"= "unversioned",
        "supported"= [],
        "unknown_version_behavior"= "compatibility text/JSON boundary; consumers must not infer a schema version",
        "compatibility_converter"= "",
        "compatibility_fixtures"= []
      },
      "validation_commands"= [
        "cargo test -p mantle errors::tests"
      ],
      "fixture_coverage"= [
        "classification"
      ],
      "freshness_strategy"= "classification only until an explicit versioned envelope lands",
      "freshness"= {
        "schema_blake3"= "",
        "contract_blake3"= "",
        "prelude_blake3"= "",
        "fixture_set_blake3"= "",
        "producer_identity_blake3"= "",
        "consumer_policy_blake3"= ""
      },
      "non_claims"= [
        "compatibility does not promise stable fields"
      ],
      "rationale"= "Existing machine errors are public but unversioned and must not be silently frozen."
    },
    {
      "id"= "mantlepkgs.catalog-domain-reports",
      "class"= "compatibility",
      "rust_owner"= "src/mantlepkgs_cmd.rs::public_json_family",
      "producer"= {
        "command_or_api"= "mantle --json mantlepkgs",
        "marker"= "mantlepkgs.catalog-domain-reports",
        "source_paths"= [
          "src/mantlepkgs_adapter.rs",
          "src/mantlepkgs_cmd.rs",
          "src/mantlepkgs_version_cmd.rs"
        ]
      },
      "consumers"= [
        "Mantlepkgs producers",
        "no-Nix catalog consumers",
        "Cairn evidence"
      ],
      "artifacts"= {
        "schema"= "",
        "generated_contract"= "",
        "positive_fixtures"= [],
        "negative_fixture_set"= ""
      },
      "version_policy"= {
        "current"= "mantlepkgs-v1-family",
        "supported"= [
          "mantlepkgs-v1-family"
        ],
        "unknown_version_behavior"= "reject unknown member schemas before planning, publication, or evidence admission",
        "compatibility_converter"= "",
        "compatibility_fixtures"= []
      },
      "validation_commands"= [
        "cargo test -p mantlepkgs-core",
        "cargo test --bin mantle mantlepkgs_cmd::tests"
      ],
      "fixture_coverage"= [
        "version",
        "identity",
        "tamper",
        "no-Nix"
      ],
      "freshness_strategy"= "classification only until the versioned Mantlepkgs artifact families receive generated contracts",
      "freshness"= {
        "schema_blake3"= "",
        "contract_blake3"= "",
        "prelude_blake3"= "",
        "fixture_set_blake3"= "",
        "producer_identity_blake3"= "",
        "consumer_policy_blake3"= ""
      },
      "non_claims"= [
        "compatibility classification does not prove package correctness, reproducibility, evaluator parity, or release eligibility"
      ],
      "rationale"= "Mantlepkgs emits several explicit v1 artifacts; this family records the public boundary until each artifact receives a generated contract."
    },
    {
      "id"= "mantlepkgs.package-impact-report",
      "class"= "contracted",
      "rust_owner"= "crates/mantlepkgs-core/src/impact.rs::MantlePackageImpactReport",
      "producer"= {
        "command_or_api"= "mantle mantlepkgs impact",
        "marker"= "mantlepkgs.package-impact-report",
        "source_paths"= [
          "crates/mantlepkgs-core/src/impact.rs",
          "src/mantlepkgs_cmd.rs"
        ]
      },
      "consumers"= [
        "external CI adapters",
        "Cairn evidence",
        "local package reviewers"
      ],
      "artifacts"= {
        "schema"= "schemas/machine-contracts/mantle-package-impact.schema.json",
        "generated_contract"= "schemas/machine-contracts/mantle-package-impact.contract.ncl",
        "positive_fixtures"= [
          "schemas/machine-contracts/fixtures/mantle-package-impact.valid.json"
        ],
        "negative_fixture_set"= "schemas/machine-contracts/fixtures/mantle-package-impact.negatives.json"
      },
      "version_policy"= {
        "current"= "mantle-package-impact-v1",
        "supported"= [
          "mantle-package-impact-v1"
        ],
        "unknown_version_behavior"= "reject before interpreting package, observation, or closure deltas",
        "compatibility_converter"= "",
        "compatibility_fixtures"= []
      },
      "validation_commands"= [
        "cargo test -p mantlepkgs-core impact::tests",
        "cargo test --bin mantle mantlepkgs_cmd::tests::impact_command"
      ],
      "fixture_coverage"= [
        "version",
        "digest",
        "unknown-field"
      ],
      "freshness_strategy"= "BLAKE3 binds the report schema, generated contract, fixtures, pure core, shell, and external adapter policy",
      "freshness"= {
        "schema_blake3"= "27c7087a4adacc0b40df941182603b724f7118aa407fc76c585ee3afa024fac8",
        "contract_blake3"= "7cd1d3ffbfe1324271c5531447350406ae29898b845e4aea58102e9bfc23e0d8",
        "prelude_blake3"= "d48a9410c5f120d10c5d815d1c3386358ca92342c6ce9c58c62401db0e97f55e",
        "fixture_set_blake3"= "1602d93a04a749077c6f0150e3e8c32ce41fc295585396e6e71f45938101fd0a",
        "producer_identity_blake3"= "1e28cc6b142988fe4baf2c6e832e5213f05a52e59849b1c20ffd5dc04ae12fcc",
        "consumer_policy_blake3"= "094ac32746282ab41656266b861ea5b70bc9f4c54f3b9a3aedf1680c6695d49b"
      },
      "non_claims"= [
        "report conformance does not prove package correctness, unchanged behavior, reproducibility, deployment safety, or release eligibility"
      ],
      "rationale"= "This stable local report lets external adapters render review results without forge authority in Mantle."
    },
    {
      "id"= "mantlepkgs.update-plan",
      "class"= "contracted",
      "rust_owner"= "crates/mantlepkgs-core/src/updates.rs::UpdatePlan",
      "producer"= {
        "command_or_api"= "mantle mantlepkgs update-plan",
        "marker"= "mantlepkgs.update-plan",
        "source_paths"= [
          "crates/mantlepkgs-core/src/updates.rs",
          "src/mantlepkgs_cmd.rs"
        ]
      },
      "consumers"= [
        "package maintainers",
        "Cairn evidence",
        "external review adapters"
      ],
      "artifacts"= {
        "schema"= "schemas/machine-contracts/mantlepkgs-update-plan.schema.json",
        "generated_contract"= "schemas/machine-contracts/mantlepkgs-update-plan.contract.ncl",
        "positive_fixtures"= [
          "schemas/machine-contracts/fixtures/mantlepkgs-update-plan.valid.json"
        ],
        "negative_fixture_set"= "schemas/machine-contracts/fixtures/mantlepkgs-update-plan.negatives.json"
      },
      "version_policy"= {
        "current"= "mantlepkgs-update-plan-v1",
        "supported"= [
          "mantlepkgs-update-plan-v1"
        ],
        "unknown_version_behavior"= "reject before interpreting candidates, effects, advisory status, or validation links",
        "compatibility_converter"= "",
        "compatibility_fixtures"= []
      },
      "validation_commands"= [
        "cargo test -p mantlepkgs-core updates::tests",
        "cargo test --bin mantle mantlepkgs_cmd::tests::update"
      ],
      "fixture_coverage"= [
        "version",
        "digest",
        "cross-field",
        "unknown-field"
      ],
      "freshness_strategy"= "BLAKE3 binds the update schema, generated contract, fixtures, pure core, and mutation shell",
      "freshness"= {
        "schema_blake3"= "21008e670f90bd69421fbae95e971bb2280f709a94a9fb48663a2a6c544720c9",
        "contract_blake3"= "a3125cbf30f9cc54e2e9f372e58002415bee51e25fa539fc25a29210de8af948",
        "prelude_blake3"= "d48a9410c5f120d10c5d815d1c3386358ca92342c6ce9c58c62401db0e97f55e",
        "fixture_set_blake3"= "144796c61a4d4ab37f35cf525e3947021ca18ada50293a777fdd644697847598",
        "producer_identity_blake3"= "5aa31575c10e1ad2b5a303ce8eddc320e6a4edf4d193095bfa3891ccfbb74bae",
        "consumer_policy_blake3"= "be8086e77584c68066c9042b83c5eff2df1d1f31680dec677d4cd4f3cd764a0f"
      },
      "non_claims"= [
        "plan conformance does not prove source trust, advisory completeness, package correctness, reproducibility, deployment safety, or release eligibility"
      ],
      "rationale"= "The stable plan lets local and external reviewers replay candidate decisions without network or mutation authority."
    },
    {
      "id"= "eval.raw-json-output",
      "class"= "external",
      "rust_owner"= "src/main.rs::Eval",
      "producer"= {
        "command_or_api"= "mantle eval --json",
        "marker"= "eval.raw-json-output",
        "source_paths"= [
          "src/main.rs"
        ]
      },
      "consumers"= [
        "user-selected pipelines"
      ],
      "artifacts"= {
        "schema"= "",
        "generated_contract"= "",
        "positive_fixtures"= [],
        "negative_fixture_set"= ""
      },
      "version_policy"= {
        "current"= "user-defined",
        "supported"= [],
        "unknown_version_behavior"= "shape is owned by evaluated Nickel data",
        "compatibility_converter"= "",
        "compatibility_fixtures"= []
      },
      "validation_commands"= [
        "cargo test -p mantle --test smoke"
      ],
      "fixture_coverage"= [
        "classification"
      ],
      "freshness_strategy"= "external shape; Mantle binds only evaluation mechanics",
      "freshness"= {
        "schema_blake3"= "",
        "contract_blake3"= "",
        "prelude_blake3"= "",
        "fixture_set_blake3"= "",
        "producer_identity_blake3"= "",
        "consumer_policy_blake3"= ""
      },
      "non_claims"= [
        "Mantle does not own arbitrary user JSON shape"
      ],
      "rationale"= "The output is deliberately defined by user Nickel rather than a Mantle DTO."
    },
    {
      "id"= "project.command-reports",
      "class"= "compatibility",
      "rust_owner"= "src/project_cmd.rs::public_json_family",
      "producer"= {
        "command_or_api"= "mantle project commands --json",
        "marker"= "project.command-reports",
        "source_paths"= [
          "src/project_cmd.rs",
          "src/project_build.rs",
          "src/project_resolve.rs",
          "src/offline_cargo.rs"
        ]
      },
      "consumers"= [
        "project tooling"
      ],
      "artifacts"= {
        "schema"= "",
        "generated_contract"= "",
        "positive_fixtures"= [],
        "negative_fixture_set"= ""
      },
      "version_policy"= {
        "current"= "mixed",
        "supported"= [],
        "unknown_version_behavior"= "reject only where the owning project format is versioned",
        "compatibility_converter"= "",
        "compatibility_fixtures"= []
      },
      "validation_commands"= [
        "cargo test -p mantle project_cmd::tests"
      ],
      "fixture_coverage"= [
        "classification"
      ],
      "freshness_strategy"= "classification only pending family split",
      "freshness"= {
        "schema_blake3"= "",
        "contract_blake3"= "",
        "prelude_blake3"= "",
        "fixture_set_blake3"= "",
        "producer_identity_blake3"= "",
        "consumer_policy_blake3"= ""
      },
      "non_claims"= [
        "group classification is not a stable aggregate schema"
      ],
      "rationale"= "Several existing project command projections require a later bounded split."
    },
    {
      "id"= "import.command-reports",
      "class"= "compatibility",
      "rust_owner"= "src/cargo_import.rs::public_json_family",
      "producer"= {
        "command_or_api"= "mantle cargo/pin/foreign import --json",
        "marker"= "import.command-reports",
        "source_paths"= [
          "src/cargo_import.rs",
          "src/pin_import.rs",
          "src/foreign_import_cmd.rs",
          "src/foreign_derivation_import.rs",
          "src/foreign_executable_plan.rs",
          "src/foreign_graph_compiler.rs",
          "src/foreign_realization.rs",
          "src/foreign_realization_receipt.rs",
          "src/foreign_realization_shell.rs",
          "src/foreign_provenance_audit.rs"
        ]
      },
      "consumers"= [
        "migration tooling"
      ],
      "artifacts"= {
        "schema"= "",
        "generated_contract"= "",
        "positive_fixtures"= [],
        "negative_fixture_set"= ""
      },
      "version_policy"= {
        "current"= "mixed",
        "supported"= [],
        "unknown_version_behavior"= "do not consume unknown import projection fields",
        "compatibility_converter"= "",
        "compatibility_fixtures"= []
      },
      "validation_commands"= [
        "cargo test -p mantle cargo_import::tests",
        "cargo test -p mantle pin_import::tests",
        "cargo test -p mantle foreign_realization::tests",
        "cargo test -p mantle foreign_realization_receipt::tests",
        "cargo test -p mantle foreign_provenance_audit::tests",
        "cargo test -p crunch-store http_closure::tests",
        "cargo test -p crunch-store provenance",
        "cargo test -p mantle --test foreign_import_cli"
      ],
      "fixture_coverage"= [
        "classification",
        "positive-provenance-audit",
        "negative-provenance-audit",
        "zstd-stream-classification",
        "normalized-store-suffixes"
      ],
      "freshness_strategy"= "classification only pending per-import schemas",
      "freshness"= {
        "schema_blake3"= "",
        "contract_blake3"= "",
        "prelude_blake3"= "",
        "fixture_set_blake3"= "",
        "producer_identity_blake3"= "",
        "consumer_policy_blake3"= ""
      },
      "non_claims"= [
        "import report shape does not prove imported build equivalence"
      ],
      "rationale"= "Existing migration reports remain compatibility surfaces outside the first cohort."
    },
    {
      "id"= "filegen.command-reports",
      "class"= "compatibility",
      "rust_owner"= "src/filegen_cmd.rs::public_json_family",
      "producer"= {
        "command_or_api"= "mantle filegen --json",
        "marker"= "filegen.command-reports",
        "source_paths"= [
          "src/filegen_cmd.rs"
        ]
      },
      "consumers"= [
        "file generation automation"
      ],
      "artifacts"= {
        "schema"= "",
        "generated_contract"= "",
        "positive_fixtures"= [],
        "negative_fixture_set"= ""
      },
      "version_policy"= {
        "current"= "unversioned",
        "supported"= [],
        "unknown_version_behavior"= "treat unknown fields as incompatible",
        "compatibility_converter"= "",
        "compatibility_fixtures"= []
      },
      "validation_commands"= [
        "cargo test -p mantle filegen_cmd::tests"
      ],
      "fixture_coverage"= [
        "classification"
      ],
      "freshness_strategy"= "classification only pending explicit versioning",
      "freshness"= {
        "schema_blake3"= "",
        "contract_blake3"= "",
        "prelude_blake3"= "",
        "fixture_set_blake3"= "",
        "producer_identity_blake3"= "",
        "consumer_policy_blake3"= ""
      },
      "non_claims"= [
        "plan output does not prove applied filesystem state"
      ],
      "rationale"= "Public plan/apply reports are retained for compatibility but are not in this cohort."
    },
    {
      "id"= "semantic-graph.command-reports",
      "class"= "internal",
      "rust_owner"= "src/semantic_graph.rs::public_json_family",
      "producer"= {
        "command_or_api"= "semantic graph APIs and CLI",
        "marker"= "semantic-graph.command-reports",
        "source_paths"= [
          "src/semantic_graph.rs"
        ]
      },
      "consumers"= [
        "Mantle diagnostics"
      ],
      "artifacts"= {
        "schema"= "",
        "generated_contract"= "",
        "positive_fixtures"= [],
        "negative_fixture_set"= ""
      },
      "version_policy"= {
        "current"= "internal",
        "supported"= [],
        "unknown_version_behavior"= "no external version support",
        "compatibility_converter"= "",
        "compatibility_fixtures"= []
      },
      "validation_commands"= [
        "cargo test -p mantle semantic_graph::tests"
      ],
      "fixture_coverage"= [
        "classification"
      ],
      "freshness_strategy"= "internal DTO tests",
      "freshness"= {
        "schema_blake3"= "",
        "contract_blake3"= "",
        "prelude_blake3"= "",
        "fixture_set_blake3"= "",
        "producer_identity_blake3"= "",
        "consumer_policy_blake3"= ""
      },
      "non_claims"= [
        "internal graph DTOs are not a compatibility promise"
      ],
      "rationale"= "The graph remains implementation-owned pending a dedicated public query contract."
    },
    {
      "id"= "attestation.general-envelopes",
      "class"= "compatibility",
      "rust_owner"= "src/attest_cmd.rs::AttestationEnvelopeOutput",
      "producer"= {
        "command_or_api"= "mantle attest show/closure/project/witness",
        "marker"= "attestation.general-envelopes",
        "source_paths"= [
          "src/attest_cmd.rs"
        ]
      },
      "consumers"= [
        "attestation tooling"
      ],
      "artifacts"= {
        "schema"= "",
        "generated_contract"= "",
        "positive_fixtures"= [],
        "negative_fixture_set"= ""
      },
      "version_policy"= {
        "current"= "per-attestation schema",
        "supported"= [],
        "unknown_version_behavior"= "delegate to each canonical attestation schema",
        "compatibility_converter"= "",
        "compatibility_fixtures"= []
      },
      "validation_commands"= [
        "cargo test -p mantle attest_cmd::tests"
      ],
      "fixture_coverage"= [
        "classification"
      ],
      "freshness_strategy"= "classification only; release envelope is contracted separately",
      "freshness"= {
        "schema_blake3"= "",
        "contract_blake3"= "",
        "prelude_blake3"= "",
        "fixture_set_blake3"= "",
        "producer_identity_blake3"= "",
        "consumer_policy_blake3"= ""
      },
      "non_claims"= [
        "generic envelope does not flatten each attestation schema"
      ],
      "rationale"= "Only the selected release handoff is frozen in the initial cohort."
    },
    {
      "id"= "attestation.verification-reports",
      "class"= "compatibility",
      "rust_owner"= "src/release_attestation.rs::ReleaseVerificationOutput",
      "producer"= {
        "command_or_api"= "mantle attest release-verify and policy commands",
        "marker"= "attestation.verification-reports",
        "source_paths"= [
          "src/release_attestation.rs"
        ]
      },
      "consumers"= [
        "release operators"
      ],
      "artifacts"= {
        "schema"= "",
        "generated_contract"= "",
        "positive_fixtures"= [],
        "negative_fixture_set"= ""
      },
      "version_policy"= {
        "current"= "mixed",
        "supported"= [],
        "unknown_version_behavior"= "reject unknown canonical attestation inputs",
        "compatibility_converter"= "",
        "compatibility_fixtures"= []
      },
      "validation_commands"= [
        "cargo test -p mantle release_attestation::tests"
      ],
      "fixture_coverage"= [
        "classification",
        "optional and explicit quorum status"
      ],
      "freshness_strategy"= "classification only pending policy-output cohort",
      "freshness"= {
        "schema_blake3"= "",
        "contract_blake3"= "",
        "prelude_blake3"= "",
        "fixture_set_blake3"= "",
        "producer_identity_blake3"= "",
        "consumer_policy_blake3"= ""
      },
      "non_claims"= [
        "verification output shape is not a substitute for cryptographic evidence"
      ],
      "rationale"= "Policy and witness reports retain exact evidence facts and separate zero-threshold status from positive quorum claims."
    },
    {
      "id"= "release.evidence-reports",
      "class"= "compatibility",
      "rust_owner"= "src/release_evidence.rs::public_json_family",
      "producer"= {
        "command_or_api"= "mantle release create/verify/reproducibility --json",
        "marker"= "release.evidence-reports",
        "source_paths"= [
          "src/release_evidence.rs",
          "src/release_cmd.rs",
          "src/release_reproducibility.rs",
          "src/global_reproducibility_cmd.rs",
          "src/global_reproducibility_release.rs",
          "src/release_nix_witness.rs",
          "src/release_source.rs",
          "src/cairn_release_handoff.rs",
          "src/release_publication.rs",
          "src/content_bound_requirement_evidence.rs",
          "src/release_chapter_transport.rs",
          "src/source_review_release.rs"
        ]
      },
      "consumers"= [
        "release automation"
      ],
      "artifacts"= {
        "schema"= "",
        "generated_contract"= "",
        "positive_fixtures"= [],
        "negative_fixture_set"= ""
      },
      "version_policy"= {
        "current"= "per-manifest format",
        "supported"= [],
        "unknown_version_behavior"= "reject unsupported release manifest formats",
        "compatibility_converter"= "",
        "compatibility_fixtures"= []
      },
      "validation_commands"= [
        "cargo test -p mantle release_evidence::tests"
      ],
      "fixture_coverage"= [
        "classification"
      ],
      "freshness_strategy"= "classification only; canonical manifests retain their own digest rails",
      "freshness"= {
        "schema_blake3"= "",
        "contract_blake3"= "",
        "prelude_blake3"= "",
        "fixture_set_blake3"= "",
        "producer_identity_blake3"= "",
        "consumer_policy_blake3"= ""
      },
      "non_claims"= [
        "report shape does not prove release eligibility or reproducibility"
      ],
      "rationale"= "Large release family needs a separate bounded contract migration."
    },
    {
      "id"= "release.witness-handoff",
      "class"= "compatibility",
      "rust_owner"= "src/witness_handoff.rs::public_json_family",
      "producer"= {
        "command_or_api"= "mantle release witness export/import/rebuild",
        "marker"= "release.witness-handoff",
        "source_paths"= [
          "src/witness_handoff.rs",
          "src/witness_rebuild.rs"
        ]
      },
      "consumers"= [
        "publishers",
        "witnesses"
      ],
      "artifacts"= {
        "schema"= "",
        "generated_contract"= "",
        "positive_fixtures"= [],
        "negative_fixture_set"= ""
      },
      "version_policy"= {
        "current"= "request-specific",
        "supported"= [],
        "unknown_version_behavior"= "reject unsupported witness request identity",
        "compatibility_converter"= "",
        "compatibility_fixtures"= []
      },
      "validation_commands"= [
        "cargo test -p mantle witness_handoff::tests"
      ],
      "fixture_coverage"= [
        "classification"
      ],
      "freshness_strategy"= "classification only under witness workflow evidence",
      "freshness"= {
        "schema_blake3"= "",
        "contract_blake3"= "",
        "prelude_blake3"= "",
        "fixture_set_blake3"= "",
        "producer_identity_blake3"= "",
        "consumer_policy_blake3"= ""
      },
      "non_claims"= [
        "handoff metadata does not establish witness independence or correctness"
      ],
      "rationale"= "Cross-machine workflow remains owned by release/witness changes."
    },
    {
      "id"= "store.command-reports",
      "class"= "compatibility",
      "rust_owner"= "src/store_cmd.rs::public_json_family",
      "producer"= {
        "command_or_api"= "mantle store commands --json",
        "marker"= "store.command-reports",
        "source_paths"= [
          "src/store_cmd.rs",
          "src/artifact_cmd.rs"
        ]
      },
      "consumers"= [
        "store operators"
      ],
      "artifacts"= {
        "schema"= "",
        "generated_contract"= "",
        "positive_fixtures"= [],
        "negative_fixture_set"= ""
      },
      "version_policy"= {
        "current"= "mixed",
        "supported"= [],
        "unknown_version_behavior"= "do not infer compatibility across command families",
        "compatibility_converter"= "",
        "compatibility_fixtures"= []
      },
      "validation_commands"= [
        "cargo test -p mantle store_cmd::tests"
      ],
      "fixture_coverage"= [
        "classification"
      ],
      "freshness_strategy"= "classification only pending command-family split",
      "freshness"= {
        "schema_blake3"= "",
        "contract_blake3"= "",
        "prelude_blake3"= "",
        "fixture_set_blake3"= "",
        "producer_identity_blake3"= "",
        "consumer_policy_blake3"= ""
      },
      "non_claims"= [
        "store report shape does not prove trust or reachability"
      ],
      "rationale"= "Existing operator-facing store projections require separate stable versions."
    },
    {
      "id"= "frontend.artifact-reports",
      "class"= "compatibility",
      "rust_owner"= "src/frontend_artifact_export.rs::public_json_family",
      "producer"= {
        "command_or_api"= "mantle frontend artifact export/import/admission",
        "marker"= "frontend.artifact-reports",
        "source_paths"= [
          "src/frontend_artifact_export.rs",
          "src/frontend_artifact_spec.rs",
          "src/frontend_artifact_store.rs"
        ]
      },
      "consumers"= [
        "frontend adapters"
      ],
      "artifacts"= {
        "schema"= "",
        "generated_contract"= "",
        "positive_fixtures"= [],
        "negative_fixture_set"= ""
      },
      "version_policy"= {
        "current"= "per-artifact spec",
        "supported"= [],
        "unknown_version_behavior"= "reject unsupported frontend spec versions",
        "compatibility_converter"= "",
        "compatibility_fixtures"= []
      },
      "validation_commands"= [
        "cargo test -p mantle frontend_artifact"
      ],
      "fixture_coverage"= [
        "classification"
      ],
      "freshness_strategy"= "classification only pending frontend-specific migration",
      "freshness"= {
        "schema_blake3"= "",
        "contract_blake3"= "",
        "prelude_blake3"= "",
        "fixture_set_blake3"= "",
        "producer_identity_blake3"= "",
        "consumer_policy_blake3"= ""
      },
      "non_claims"= [
        "artifact report shape does not prove frontend semantics"
      ],
      "rationale"= "Frontend specs have an independent owner and are not frozen by the build report embedding."
    },
    {
      "id"= "bootstrap.validation-reports",
      "class"= "internal",
      "rust_owner"= "src/bootstrap_validate.rs::public_json_family",
      "producer"= {
        "command_or_api"= "mantle bootstrap validation",
        "marker"= "bootstrap.validation-reports",
        "source_paths"= [
          "src/bootstrap.rs",
          "src/bootstrap_validate.rs",
          "src/bootstrap_source_root.rs",
          "src/source_root_capability.rs",
          "src/source_root_provider.rs",
          "src/rust_source_provider.rs",
          "src/rust_bootstrap_patch_plan.rs",
          "src/source_toolchain_closure.rs",
          "src/native_toolchain_closure.rs",
          "src/vendor_source_manifest.rs",
          "src/full_source_rust_binding.rs",
          "src/full_source_rust_binding_shell.rs",
          "src/source_built_derivation_action_plan.rs",
          "src/source_built_fixed_point.rs",
          "src/source_built_fixed_point_checkpoint.rs",
          "src/source_built_fixed_point_checkpoint_shell.rs",
          "src/source_built_fixed_point_receipt.rs",
          "src/source_built_fixed_point_shell.rs",
          "src/source_built_fixed_point_shell/checkpoint_integration.rs",
          "src/source_built_root_action_trust.rs",
          "src/source_built_rust_action_plan.rs",
          "src/source_built_rust_action_shell.rs",
          "src/source_built_rust_provider_action.rs",
          "src/source_built_trust_report_shell.rs",
          "src/stagex_bash.rs",
          "src/stagex_bash_full.rs",
          "src/stagex_binutils.rs",
          "src/stagex_bison.rs",
          "src/stagex_bzip2.rs",
          "src/stagex_coreutils.rs",
          "src/stagex_diffutils.rs",
          "src/stagex_flex.rs",
          "src/stagex_gawk.rs",
          "src/stagex_gnu_patch.rs",
          "src/stagex_grep.rs",
          "src/stagex_gzip.rs",
          "src/stagex_m4.rs",
          "src/stagex_make.rs",
          "src/stagex_mes.rs",
          "src/stagex_mes_lib.rs",
          "src/stagex_mes_sources.rs",
          "src/stagex_musl.rs",
          "src/stagex_musl_native.rs",
          "src/stagex_oyacc.rs",
          "src/stagex_provider.rs",
          "src/stagex_sed.rs",
          "src/stagex_sources.rs",
          "src/stagex_stage0.rs",
          "src/stagex_stage0_full.rs",
          "src/stagex_tar.rs",
          "src/stagex_tcc_musl.rs",
          "src/stagex_tcc_musl_prep.rs",
          "src/stagex_tcc_musl_v2.rs",
          "src/stagex_tcc_selfhost.rs",
          "src/stagex_tinycc.rs",
          "src/stagex_tinycc27.rs",
          "src/stagex_transition.rs"
        ]
      },
      "consumers"= [
        "bootstrap implementation"
      ],
      "artifacts"= {
        "schema"= "",
        "generated_contract"= "",
        "positive_fixtures"= [],
        "negative_fixture_set"= ""
      },
      "version_policy"= {
        "current"= "internal",
        "supported"= [],
        "unknown_version_behavior"= "no external compatibility",
        "compatibility_converter"= "",
        "compatibility_fixtures"= []
      },
      "validation_commands"= [
        "cargo test -p mantle bootstrap"
      ],
      "fixture_coverage"= [
        "classification"
      ],
      "freshness_strategy"= "internal tests and proof bundles",
      "freshness"= {
        "schema_blake3"= "",
        "contract_blake3"= "",
        "prelude_blake3"= "",
        "fixture_set_blake3"= "",
        "producer_identity_blake3"= "",
        "consumer_policy_blake3"= ""
      },
      "non_claims"= [
        "internal validation DTOs are not a public schema"
      ],
      "rationale"= "Bootstrap evidence is governed by proof manifests rather than this first cohort."
    },
    {
      "id"= "bootstrap.parity-reports",
      "class"= "debug",
      "rust_owner"= "src/bootstrap_parity.rs::public_json_family",
      "producer"= {
        "command_or_api"= "bootstrap parity diagnostics",
        "marker"= "bootstrap.parity-reports",
        "source_paths"= [
          "src/bootstrap_parity.rs"
        ]
      },
      "consumers"= [
        "developers"
      ],
      "artifacts"= {
        "schema"= "",
        "generated_contract"= "",
        "positive_fixtures"= [],
        "negative_fixture_set"= ""
      },
      "version_policy"= {
        "current"= "debug",
        "supported"= [],
        "unknown_version_behavior"= "not accepted as automation input",
        "compatibility_converter"= "",
        "compatibility_fixtures"= []
      },
      "validation_commands"= [
        "cargo test -p mantle bootstrap_parity::tests"
      ],
      "fixture_coverage"= [
        "classification"
      ],
      "freshness_strategy"= "debug tests only",
      "freshness"= {
        "schema_blake3"= "",
        "contract_blake3"= "",
        "prelude_blake3"= "",
        "fixture_set_blake3"= "",
        "producer_identity_blake3"= "",
        "consumer_policy_blake3"= ""
      },
      "non_claims"= [
        "debug parity output is not release evidence"
      ],
      "rationale"= "Diagnostic surface intentionally remains unstable."
    },
    {
      "id"= "nix-free-demo.reports",
      "class"= "debug",
      "rust_owner"= "src/nix_free_demo_cmd.rs::public_json_family",
      "producer"= {
        "command_or_api"= "mantle nix-free demo --json",
        "marker"= "nix-free-demo.reports",
        "source_paths"= [
          "src/nix_free_demo_cmd.rs",
          "src/nix_free_demo_bundle.rs"
        ]
      },
      "consumers"= [
        "demonstration tooling"
      ],
      "artifacts"= {
        "schema"= "",
        "generated_contract"= "",
        "positive_fixtures"= [],
        "negative_fixture_set"= ""
      },
      "version_policy"= {
        "current"= "demo",
        "supported"= [],
        "unknown_version_behavior"= "not accepted as readiness evidence",
        "compatibility_converter"= "",
        "compatibility_fixtures"= []
      },
      "validation_commands"= [
        "cargo test -p mantle nix_free_demo"
      ],
      "fixture_coverage"= [
        "classification"
      ],
      "freshness_strategy"= "demo tests only",
      "freshness"= {
        "schema_blake3"= "",
        "contract_blake3"= "",
        "prelude_blake3"= "",
        "fixture_set_blake3"= "",
        "producer_identity_blake3"= "",
        "consumer_policy_blake3"= ""
      },
      "non_claims"= [
        "demo output is not release or bootstrap proof"
      ],
      "rationale"= "Demonstration bundles are bounded diagnostics, not a stable API."
    },
    {
      "id"= "verification.gauntlet-reports",
      "class"= "debug",
      "rust_owner"= "src/verification_gauntlet_cmd.rs::public_json_family",
      "producer"= {
        "command_or_api"= "mantle verification gauntlet",
        "marker"= "verification.gauntlet-reports",
        "source_paths"= [
          "src/verification_gauntlet_cmd.rs"
        ]
      },
      "consumers"= [
        "developers",
        "CI diagnostics"
      ],
      "artifacts"= {
        "schema"= "",
        "generated_contract"= "",
        "positive_fixtures"= [],
        "negative_fixture_set"= ""
      },
      "version_policy"= {
        "current"= "debug",
        "supported"= [],
        "unknown_version_behavior"= "do not use as stable evidence",
        "compatibility_converter"= "",
        "compatibility_fixtures"= []
      },
      "validation_commands"= [
        "cargo test -p mantle verification_gauntlet_cmd::tests"
      ],
      "fixture_coverage"= [
        "classification"
      ],
      "freshness_strategy"= "debug rail tests",
      "freshness"= {
        "schema_blake3"= "",
        "contract_blake3"= "",
        "prelude_blake3"= "",
        "fixture_set_blake3"= "",
        "producer_identity_blake3"= "",
        "consumer_policy_blake3"= ""
      },
      "non_claims"= [
        "gauntlet status is not release eligibility"
      ],
      "rationale"= "Aggregated test diagnostics can evolve with the rail."
    },
    {
      "id"= "self-build.cargo-free-reports",
      "class"= "debug",
      "rust_owner"= "src/cargo_free_self_build.rs::public_json_family",
      "producer"= {
        "command_or_api"= "mantle self-build --cargo-free",
        "marker"= "self-build.cargo-free-reports",
        "source_paths"= [
          "src/cargo_free_self_build.rs",
          "src/self_build.rs",
          "src/protected_exec.rs",
          "src/protected_exec_seccomp.rs"
        ]
      },
      "consumers"= [
        "self-hosting proof tooling"
      ],
      "artifacts"= {
        "schema"= "",
        "generated_contract"= "",
        "positive_fixtures"= [],
        "negative_fixture_set"= ""
      },
      "version_policy"= {
        "current"= "proof-local",
        "supported"= [],
        "unknown_version_behavior"= "validate through proof manifest",
        "compatibility_converter"= "",
        "compatibility_fixtures"= []
      },
      "validation_commands"= [
        "cargo test -p mantle cargo_free_self_build::tests"
      ],
      "fixture_coverage"= [
        "classification"
      ],
      "freshness_strategy"= "proof bundle digests rather than public contract",
      "freshness"= {
        "schema_blake3"= "",
        "contract_blake3"= "",
        "prelude_blake3"= "",
        "fixture_set_blake3"= "",
        "producer_identity_blake3"= "",
        "consumer_policy_blake3"= ""
      },
      "non_claims"= [
        "debug report alone does not prove a fixed point"
      ],
      "rationale"= "Self-build receipts are evidence-local and await their owning proof migration."
    },
    {
      "id"= "rust-plan.receipts",
      "class"= "compatibility",
      "rust_owner"= "src/rust_plan.rs::RustUnitTopologyExecutionReceipt",
      "producer"= {
        "command_or_api"= "mantle rust-plan --json",
        "marker"= "rust-plan.receipts",
        "source_paths"= [
          "src/rust_plan.rs"
        ]
      },
      "consumers"= [
        "native Rust build tooling"
      ],
      "artifacts"= {
        "schema"= "",
        "generated_contract"= "",
        "positive_fixtures"= [],
        "negative_fixture_set"= ""
      },
      "version_policy"= {
        "current"= "receipt-specific",
        "supported"= [],
        "unknown_version_behavior"= "reject unsupported receipt schemas",
        "compatibility_converter"= "",
        "compatibility_fixtures"= []
      },
      "validation_commands"= [
        "cargo test -p mantle rust_plan::tests"
      ],
      "fixture_coverage"= [
        "classification"
      ],
      "freshness_strategy"= "existing receipt tests pending dedicated contract",
      "freshness"= {
        "schema_blake3"= "",
        "contract_blake3"= "",
        "prelude_blake3"= "",
        "fixture_set_blake3"= "",
        "producer_identity_blake3"= "",
        "consumer_policy_blake3"= ""
      },
      "non_claims"= [
        "receipt shape does not prove Cargo parity or fixed point"
      ],
      "rationale"= "Large topology receipt remains owned by the native Rust planning changes."
    },
    {
      "id"= "remote.execution-reports",
      "class"= "compatibility",
      "rust_owner"= "src/remote_build.rs and src/remote_failure_debug.rs public JSON family",
      "producer"= {
        "command_or_api"= "mantle remote build/status/serve/failure-debug --json",
        "marker"= "remote.execution-reports",
        "source_paths"= [
          "src/remote_build.rs",
          "src/remote_hexagon.rs",
          "src/remote_build/tests/external_batch_hardware_tests.rs",
          "src/remote_failure_debug.rs",
          "src/external_batch_dispatch.rs",
          "src/remote_farm_config.rs",
          "src/remote_credentials.rs"
        ]
      },
      "consumers"= [
        "remote clients",
        "operators"
      ],
      "artifacts"= {
        "schema"= "",
        "generated_contract"= "",
        "positive_fixtures"= [],
        "negative_fixture_set"= ""
      },
      "version_policy"= {
        "current"= "protocol-specific",
        "supported"= [],
        "unknown_version_behavior"= "negotiate or reject unsupported remote protocol",
        "compatibility_converter"= "",
        "compatibility_fixtures"= []
      },
      "validation_commands"= [
        "cargo test -p mantle remote_build::tests",
        "cargo test -p mantle remote_failure_debug::tests",
        "cargo test -p mantle external_batch_dispatch::tests",
        "cargo test -p mantle remote_farm_config::tests"
      ],
      "fixture_coverage"= [
        "classification"
      ],
      "freshness_strategy"= "owned remote protocol tests; no cohort contract",
      "freshness"= {
        "schema_blake3"= "",
        "contract_blake3"= "",
        "prelude_blake3"= "",
        "fixture_set_blake3"= "",
        "producer_identity_blake3"= "",
        "consumer_policy_blake3"= ""
      },
      "non_claims"= [
        "remote status and failure diagnostics do not prove output admission, exactly-once execution, determinism, or replay authority"
      ],
      "rationale"= "Current remote execution and bounded failure-diagnostic reports stay under one compatibility family until a closed cohort contract is adopted."
    },
    {
      "id"= "remote.attempt-future",
      "class"= "internal",
      "rust_owner"= "src/remote_build.rs::PlannedFencedAttemptArtifacts",
      "producer"= {
        "command_or_api"= "planned fenced remote attempt records",
        "marker"= "remote.attempt-future",
        "source_paths"= [
          "src/remote_build.rs"
        ]
      },
      "consumers"= [
        "fence-durable-remote-attempts change"
      ],
      "artifacts"= {
        "schema"= "",
        "generated_contract"= "",
        "positive_fixtures"= [],
        "negative_fixture_set"= ""
      },
      "version_policy"= {
        "current"= "planned",
        "supported"= [],
        "unknown_version_behavior"= "owning change must version before exposure",
        "compatibility_converter"= "",
        "compatibility_fixtures"= []
      },
      "validation_commands"= [
        "cairn validate fence-durable-remote-attempts"
      ],
      "fixture_coverage"= [
        "classification"
      ],
      "freshness_strategy"= "future classification only; owning change supplies identities",
      "freshness"= {
        "schema_blake3"= "",
        "contract_blake3"= "",
        "prelude_blake3"= "",
        "fixture_set_blake3"= "",
        "producer_identity_blake3"= "",
        "consumer_policy_blake3"= ""
      },
      "non_claims"= [
        "registration does not claim the future artifact is implemented"
      ],
      "rationale"= "Registers the active fenced-attempt surface without preempting its owner."
    },
    {
      "id"= "remote.observability-future",
      "class"= "internal",
      "rust_owner"= "src/remote_build.rs::RemoteObservabilityHealth",
      "producer"= {
        "command_or_api"= "remote build lifecycle telemetry, immutable attempt diagnostics, and health reports",
        "marker"= "remote.observability-future",
        "source_paths"= [
          "src/remote_build.rs",
          "src/remote_attempt_log_store.rs",
          "src/remote_telemetry_export.rs",
          "src/remote_trace_context.rs",
          "src/remote_transfer.rs",
          "crates/crunch-build/src/distributed/remote_telemetry.rs"
        ]
      },
      "consumers"= [
        "remote build operators",
        "build report automation"
      ],
      "artifacts"= {
        "schema"= "",
        "generated_contract"= "",
        "positive_fixtures"= [],
        "negative_fixture_set"= ""
      },
      "version_policy"= {
        "current"= "mantle-remote-observability-v1",
        "supported"= [
          "mantle-remote-observability-v1"
        ],
        "unknown_version_behavior"= "reject unknown standalone trace or telemetry frames while preserving build-result independence for diagnostics",
        "compatibility_converter"= "",
        "compatibility_fixtures"= []
      },
      "validation_commands"= [
        "cargo test -p mantle --bin mantle remote_trace_context::tests",
        "cargo test -p mantle --test remote_transfer_production production_remote_observability",
        "cairn validate persist-remote-attempt-observability"
      ],
      "fixture_coverage"= [
        "classification",
        "malformed-trace-context",
        "exporter-outage"
      ],
      "freshness_strategy"= "internal lifecycle events are embedded in the contracted build report; immutable logs and exporter state remain bounded diagnostic surfaces",
      "freshness"= {
        "schema_blake3"= "",
        "contract_blake3"= "",
        "prelude_blake3"= "",
        "fixture_set_blake3"= "",
        "producer_identity_blake3"= "",
        "consumer_policy_blake3"= ""
      },
      "non_claims"= [
        "diagnostic observability does not affect identity, authorization, scheduling, fencing, admission, output trust, build success, reproducibility, or release eligibility"
      ],
      "rationale"= "Classifies the bounded internal observability surfaces while the closed build-report schema owns their exported JSON projection."
    },
    {
      "id"= "remote.transfer-future",
      "class"= "internal",
      "rust_owner"= "src/remote_build.rs::PlannedResumableTransferArtifacts",
      "producer"= {
        "command_or_api"= "planned resumable CAS transfer checkpoints",
        "marker"= "remote.transfer-future",
        "source_paths"= [
          "src/remote_build.rs"
        ]
      },
      "consumers"= [
        "complete-resumable-remote-cas-transfer change"
      ],
      "artifacts"= {
        "schema"= "",
        "generated_contract"= "",
        "positive_fixtures"= [],
        "negative_fixture_set"= ""
      },
      "version_policy"= {
        "current"= "planned",
        "supported"= [],
        "unknown_version_behavior"= "owning change must negotiate versions",
        "compatibility_converter"= "",
        "compatibility_fixtures"= []
      },
      "validation_commands"= [
        "cairn validate complete-resumable-remote-cas-transfer"
      ],
      "fixture_coverage"= [
        "classification"
      ],
      "freshness_strategy"= "future classification only",
      "freshness"= {
        "schema_blake3"= "",
        "contract_blake3"= "",
        "prelude_blake3"= "",
        "fixture_set_blake3"= "",
        "producer_identity_blake3"= "",
        "consumer_policy_blake3"= ""
      },
      "non_claims"= [
        "registration does not claim resumable transfer exists"
      ],
      "rationale"= "Registers the active transfer surface without preempting capability design."
    },
    {
      "id"= "wasm.component-future",
      "class"= "internal",
      "rust_owner"= "src/build_correctness.rs::PlannedWasmComponentArtifacts",
      "producer"= {
        "command_or_api"= "planned and production Wasm component reports and materialization bundle",
        "marker"= "wasm.component-future",
        "source_paths"= [
          "src/build_correctness.rs",
          "src/wasm_component_cmd.rs"
        ]
      },
      "consumers"= [
        "add-wasm-component-build-pipeline change"
      ],
      "artifacts"= {
        "schema"= "",
        "generated_contract"= "",
        "positive_fixtures"= [],
        "negative_fixture_set"= ""
      },
      "version_policy"= {
        "current"= "mixed-active",
        "supported"= [],
        "unknown_version_behavior"= "active change owns versioning and authoritative admission",
        "compatibility_converter"= "",
        "compatibility_fixtures"= []
      },
      "validation_commands"= [
        "cairn validate add-wasm-component-build-pipeline",
        "cargo test -p mantle --test wasm_component_cli --offline"
      ],
      "fixture_coverage"= [
        "classification"
      ],
      "freshness_strategy"= "classification only pending authoritative contract",
      "freshness"= {
        "schema_blake3"= "",
        "contract_blake3"= "",
        "prelude_blake3"= "",
        "fixture_set_blake3"= "",
        "producer_identity_blake3"= "",
        "consumer_policy_blake3"= ""
      },
      "non_claims"= [
        "registration does not claim Octet acceptance or final materialization admission"
      ],
      "rationale"= "Classifies active component reports and bundle without replacing the owning change's authority boundary."
    },
    {
      "id"= "build.correctness-primitives",
      "class"= "internal",
      "rust_owner"= "src/build_correctness.rs::public_json_family",
      "producer"= {
        "command_or_api"= "build correctness sidecars and receipts",
        "marker"= "build.correctness-primitives",
        "source_paths"= [
          "src/build_correctness.rs"
        ]
      },
      "consumers"= [
        "build implementation",
        "proof tooling"
      ],
      "artifacts"= {
        "schema"= "",
        "generated_contract"= "",
        "positive_fixtures"= [],
        "negative_fixture_set"= ""
      },
      "version_policy"= {
        "current"= "internal",
        "supported"= [],
        "unknown_version_behavior"= "reject at each owning sidecar parser",
        "compatibility_converter"= "",
        "compatibility_fixtures"= []
      },
      "validation_commands"= [
        "cargo test -p mantle build_correctness::tests"
      ],
      "fixture_coverage"= [
        "classification"
      ],
      "freshness_strategy"= "internal unit and sidecar tests",
      "freshness"= {
        "schema_blake3"= "",
        "contract_blake3"= "",
        "prelude_blake3"= "",
        "fixture_set_blake3"= "",
        "producer_identity_blake3"= "",
        "consumer_policy_blake3"= ""
      },
      "non_claims"= [
        "internal primitives are not one aggregate public schema"
      ],
      "rationale"= "Action, sandbox, scan, and receipt types remain separate implementation boundaries."
    },
    {
      "id"= "runtime.diagnostic-fingerprint",
      "class"= "debug",
      "rust_owner"= "src/operator_diagnostics.rs::RuntimeDiagnosticFingerprint",
      "producer"= {
        "command_or_api"= "verbose runtime fingerprint",
        "marker"= "runtime.diagnostic-fingerprint",
        "source_paths"= [
          "src/operator_diagnostics.rs"
        ]
      },
      "consumers"= [
        "support diagnostics"
      ],
      "artifacts"= {
        "schema"= "",
        "generated_contract"= "",
        "positive_fixtures"= [],
        "negative_fixture_set"= ""
      },
      "version_policy"= {
        "current"= "mantle-runtime-fingerprint-v1",
        "supported"= [],
        "unknown_version_behavior"= "diagnostic readers must ignore unknown versions",
        "compatibility_converter"= "",
        "compatibility_fixtures"= []
      },
      "validation_commands"= [
        "cargo test -p mantle operator_diagnostics::tests"
      ],
      "fixture_coverage"= [
        "classification"
      ],
      "freshness_strategy"= "debug tests and redaction assertions",
      "freshness"= {
        "schema_blake3"= "",
        "contract_blake3"= "",
        "prelude_blake3"= "",
        "fixture_set_blake3"= "",
        "producer_identity_blake3"= "",
        "consumer_policy_blake3"= ""
      },
      "non_claims"= [
        "fingerprint is bounded context, not a readiness or security claim"
      ],
      "rationale"= "Explicitly debug-only and emitted only under diagnostic triggers."
    },
    {
      "id"= "source-bundle.manifest-artifacts",
      "class"= "compatibility",
      "rust_owner"= "src/source_bundle.rs::SourceBundleManifest",
      "producer"= {
        "command_or_api"= "source bundle export/import files",
        "marker"= "source-bundle.manifest-artifacts",
        "source_paths"= [
          "src/source_bundle.rs"
        ]
      },
      "consumers"= [
        "offline source transfer"
      ],
      "artifacts"= {
        "schema"= "",
        "generated_contract"= "",
        "positive_fixtures"= [],
        "negative_fixture_set"= ""
      },
      "version_policy"= {
        "current"= "mantle-source-bundle-v1",
        "supported"= [],
        "unknown_version_behavior"= "reject unsupported manifest version",
        "compatibility_converter"= "",
        "compatibility_fixtures"= []
      },
      "validation_commands"= [
        "cargo test -p mantle source_bundle::tests"
      ],
      "fixture_coverage"= [
        "classification"
      ],
      "freshness_strategy"= "canonical manifest BLAKE3 and Rust tests",
      "freshness"= {
        "schema_blake3"= "",
        "contract_blake3"= "",
        "prelude_blake3"= "",
        "fixture_set_blake3"= "",
        "producer_identity_blake3"= "",
        "consumer_policy_blake3"= ""
      },
      "non_claims"= [
        "manifest presence does not prove source trust"
      ],
      "rationale"= "File artifact is versioned but outside the report-focused initial cohort."
    },
    {
      "id"= "structured-refactor.command-reports",
      "class"= "compatibility",
      "rust_owner"= "src/structured_refactor.rs::public_json_family",
      "producer"= {
        "command_or_api"= "mantle refactor commands --json",
        "marker"= "structured-refactor.command-reports",
        "source_paths"= [
          "src/structured_refactor.rs",
          "src/main.rs"
        ]
      },
      "consumers"= [
        "refactor automation"
      ],
      "artifacts"= {
        "schema"= "",
        "generated_contract"= "",
        "positive_fixtures"= [],
        "negative_fixture_set"= ""
      },
      "version_policy"= {
        "current"= "session-specific",
        "supported"= [],
        "unknown_version_behavior"= "do not infer compatibility across refactor sessions",
        "compatibility_converter"= "",
        "compatibility_fixtures"= []
      },
      "validation_commands"= [
        "cargo test -p mantle structured_refactor::tests"
      ],
      "fixture_coverage"= [
        "classification"
      ],
      "freshness_strategy"= "classification only pending a dedicated refactor-session contract",
      "freshness"= {
        "schema_blake3"= "",
        "contract_blake3"= "",
        "prelude_blake3"= "",
        "fixture_set_blake3"= "",
        "producer_identity_blake3"= "",
        "consumer_policy_blake3"= ""
      },
      "non_claims"= [
        "refactor report shape does not prove source equivalence or applied edits"
      ],
      "rationale"= "Existing session and diagnostic JSON remains a compatibility family outside the first cohort."
    },
    {
      "id"= "transcript.evidence-reports",
      "class"= "compatibility",
      "rust_owner"= "src/transcript_cmd.rs::public_json_family",
      "producer"= {
        "command_or_api"= "mantle transcript commands",
        "marker"= "transcript.evidence-reports",
        "source_paths"= [
          "src/transcript_cmd.rs"
        ]
      },
      "consumers"= [
        "audit and evidence tooling"
      ],
      "artifacts"= {
        "schema"= "",
        "generated_contract"= "",
        "positive_fixtures"= [],
        "negative_fixture_set"= ""
      },
      "version_policy"= {
        "current"= "evidence-specific",
        "supported"= [],
        "unknown_version_behavior"= "reject only through the owning evidence schema",
        "compatibility_converter"= "",
        "compatibility_fixtures"= []
      },
      "validation_commands"= [
        "cargo test -p mantle transcript_cmd::tests"
      ],
      "fixture_coverage"= [
        "classification"
      ],
      "freshness_strategy"= "classification only; evidence files retain their own digests",
      "freshness"= {
        "schema_blake3"= "",
        "contract_blake3"= "",
        "prelude_blake3"= "",
        "fixture_set_blake3"= "",
        "producer_identity_blake3"= "",
        "consumer_policy_blake3"= ""
      },
      "non_claims"= [
        "transcript formatting does not prove command correctness or evidence trust"
      ],
      "rationale"= "Transcript JSON is public evidence plumbing but not one stable aggregate DTO."
    },
    {
      "id"= "receipt.bundle-artifacts",
      "class"= "compatibility",
      "rust_owner"= "src/portable_receipt.rs::ReceiptBundle",
      "producer"= {
        "command_or_api"= "portable receipt bundle files",
        "marker"= "receipt.bundle-artifacts",
        "source_paths"= [
          "src/portable_receipt.rs"
        ]
      },
      "consumers"= [
        "cross-machine receipt transfer"
      ],
      "artifacts"= {
        "schema"= "",
        "generated_contract"= "",
        "positive_fixtures"= [],
        "negative_fixture_set"= ""
      },
      "version_policy"= {
        "current"= "mantle-build-receipt-bundle-v1",
        "supported"= [],
        "unknown_version_behavior"= "reject unsupported bundle version",
        "compatibility_converter"= "",
        "compatibility_fixtures"= []
      },
      "validation_commands"= [
        "cargo test -p mantle portable_receipt::tests"
      ],
      "fixture_coverage"= [
        "classification"
      ],
      "freshness_strategy"= "canonical bundle digest and Rust parser tests",
      "freshness"= {
        "schema_blake3"= "",
        "contract_blake3"= "",
        "prelude_blake3"= "",
        "fixture_set_blake3"= "",
        "producer_identity_blake3"= "",
        "consumer_policy_blake3"= ""
      },
      "non_claims"= [
        "bundle syntax does not prove evidence truth"
      ],
      "rationale"= "The report projections are contracted now; the full portable bundle remains compatibility-owned."
    },
    {
      "id"= "build.evaluation-stream",
      "class"= "compatibility",
      "rust_owner"= "src/evaluation_stream_output.rs::EvaluationStreamWriter",
      "producer"= {
        "command_or_api"= "mantle build --evaluation-stream",
        "marker"= "build.evaluation-stream",
        "source_paths"= [
          "src/evaluation_stream_output.rs",
          "src/build_cmd.rs"
        ]
      },
      "consumers"= [
        "build automation",
        "selected-root monitors"
      ],
      "artifacts"= {
        "schema"= "",
        "generated_contract"= "",
        "positive_fixtures"= [],
        "negative_fixture_set"= ""
      },
      "version_policy"= {
        "current"= "mantle-evaluation-stream-v1",
        "supported"= [
          "mantle-evaluation-stream-v1"
        ],
        "unknown_version_behavior"= "reject before accepting the record",
        "compatibility_converter"= "",
        "compatibility_fixtures"= []
      },
      "validation_commands"= [
        "nix develop -c cargo -Zscript scripts/check-evaluation-stream-contract.rs",
        "cargo test -p mantle --test integration evaluation_stream"
      ],
      "fixture_coverage"= [
        "all record kinds",
        "partial outcomes",
        "unknown versions",
        "malformed and oversized records"
      ],
      "freshness_strategy"= "A dedicated bounded NDJSON checker owns record framing and schema validation",
      "freshness"= {
        "schema_blake3"= "",
        "contract_blake3"= "",
        "prelude_blake3"= "",
        "fixture_set_blake3"= "",
        "producer_identity_blake3"= "",
        "consumer_policy_blake3"= ""
      },
      "non_claims"= [
        "stream shape does not prove evaluation or build correctness"
      ],
      "rationale"= "The public stream is versioned and checked by the NDJSON-specific contract rail; the document-JSON registry cannot render multi-record framing."
    },
    {
      "id"= "evaluation.budget-reports",
      "class"= "compatibility",
      "rust_owner"= "crates/crunch-eval-budget-core/src/lib.rs::EvaluationBudgetReport",
      "producer"= {
        "command_or_api"= "bounded evaluator report files and worker protocol",
        "marker"= "evaluation.budget-reports",
        "source_paths"= [
          "crates/crunch-eval-budget-core/src/lib.rs",
          "src/evaluator_budget.rs"
        ]
      },
      "consumers"= [
        "evaluation operators",
        "resource-policy diagnostics",
        "CI evidence"
      ],
      "artifacts"= {
        "schema"= "",
        "generated_contract"= "",
        "positive_fixtures"= [],
        "negative_fixture_set"= ""
      },
      "version_policy"= {
        "current"= "mantle-evaluation-budget-report-v1",
        "supported"= [
          "mantle-evaluation-budget-report-v1"
        ],
        "unknown_version_behavior"= "reject before interpreting budget or teardown facts",
        "compatibility_converter"= "",
        "compatibility_fixtures"= []
      },
      "validation_commands"= [
        "cargo test -p crunch-eval-budget-core",
        "cargo test -p mantle --bin mantle evaluator_budget::"
      ],
      "fixture_coverage"= [
        "positive bounded completion",
        "invalid policy",
        "resource limit",
        "timeout and teardown"
      ],
      "freshness_strategy"= "The no-std budget core owns the closed report DTO and the std shell owns bounded process observations",
      "freshness"= {
        "schema_blake3"= "",
        "contract_blake3"= "",
        "prelude_blake3"= "",
        "fixture_set_blake3"= "",
        "producer_identity_blake3"= "",
        "consumer_policy_blake3"= ""
      },
      "non_claims"= [
        "budget report conformance does not prove evaluator correctness, semantic equivalence, sandboxing, or release eligibility"
      ],
      "rationale"= "The report is versioned and public, but its dedicated core tests remain authoritative until a generated document-JSON contract is adopted."
    },
    {
      "id"= "operator.command-contract-reports",
      "class"= "compatibility",
      "rust_owner"= "src/operator_contract.rs::OperatorInventory,CommandCatalog,RemediationRecord",
      "producer"= {
        "command_or_api"= "operator contract generation and remediation JSON",
        "marker"= "operator.command-contract-reports",
        "source_paths"= [
          "src/operator_contract.rs",
          "tools/generate_operator_command_contract.rs"
        ]
      },
      "consumers"= [
        "operator documentation generation",
        "CLI remediation",
        "support automation"
      ],
      "artifacts"= {
        "schema"= "",
        "generated_contract"= "",
        "positive_fixtures"= [],
        "negative_fixture_set"= ""
      },
      "version_policy"= {
        "current"= "mantle-operator-contract-v1-family",
        "supported"= [
          "mantle-operator-surface-inventory-v1",
          "mantle-operator-command-catalog-v1",
          "mantle-remediation-v1"
        ],
        "unknown_version_behavior"= "reject before generating command guidance or remediation",
        "compatibility_converter"= "",
        "compatibility_fixtures"= []
      },
      "validation_commands"= [
        "./scripts/check-operator-command-contract.sh",
        "cargo test -p mantle --bin mantle operator_contract::"
      ],
      "fixture_coverage"= [
        "inventory admission",
        "command catalog",
        "negative stale surface",
        "bounded remediation"
      ],
      "freshness_strategy"= "The repository-owned operator contract checker binds generated docs and command descriptors",
      "freshness"= {
        "schema_blake3"= "",
        "contract_blake3"= "",
        "prelude_blake3"= "",
        "fixture_set_blake3"= "",
        "producer_identity_blake3"= "",
        "consumer_policy_blake3"= ""
      },
      "non_claims"= [
        "operator contract shape does not prove command success, host readiness, authorization, or release eligibility"
      ],
      "rationale"= "The operator family has an independent generation checker and remains a compatibility family until each document-JSON projection is contracted separately."
    }
  ]
}
# registry-data-end
| Registry
