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
    "source-bundle.offline-preflight-report",
    "nickel-export.report",
    "nickel-export.receipt",
    "attestation.release-envelope"
  ],
  "surfaces"= [
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
        "contract_blake3"= "c0021386177a6791a848a95a1740926cb3c44e9fb345a8e70cdff361f26da667",
        "prelude_blake3"= "496f0c79d7918f1761ccb04a0ce3643954aa2653eeae115068ea67092fda4db5",
        "fixture_set_blake3"= "b2c0eee0dfbf01929d3f4b3aca13cab3695a693053e5249ca5d79c186966b586",
        "producer_identity_blake3"= "c16c10e8d1cee02eff583fed1a5baa898be17a7088bcb9cd7252a231637a85a0",
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
          "src/frontend_artifact_spec.rs"
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
        "unknown-field"
      ],
      "freshness_strategy"= "BLAKE3 binds the full report projection and all directly serialized owner modules",
      "freshness"= {
        "schema_blake3"= "ed523975017580b2b79790073efc745cce361efd088966ab1054dd12e1e7a2b8",
        "contract_blake3"= "bdfc3d491025bbda7615332f8290420fe6bb50ce15b7d54434286de7020a3dda",
        "prelude_blake3"= "496f0c79d7918f1761ccb04a0ce3643954aa2653eeae115068ea67092fda4db5",
        "fixture_set_blake3"= "a61a5899057e4eecf8164a0b97c3b355cbfcebd58468743fe04d68d41454747b",
        "producer_identity_blake3"= "98e640e678b93acf1cb4e5d04447d6f0adfbd3a8100781ea49128b57d57d715d",
        "consumer_policy_blake3"= "7d4455f3467f3be7d648f7b5f2db114d776215cfc4c89187da0fa2c07a5520dd"
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
        "contract_blake3"= "fa172e4f74cf9605bd211ac683c628a83917b623554554b38538b58f49d56829",
        "prelude_blake3"= "496f0c79d7918f1761ccb04a0ce3643954aa2653eeae115068ea67092fda4db5",
        "fixture_set_blake3"= "576b0cc4dbed5d3c850a408c032411c180e9a0446507ec333606c25c4b15f719",
        "producer_identity_blake3"= "0f6cd8b4b7cae8dd640acf86d40f89937a937195fb517dfa8b6e9d128bc78b24",
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
      "rust_owner"= "src/realization_routing.rs::RoutePlanReport",
      "producer"= {
        "command_or_api"= "plan_realization_route",
        "marker"= "realization.route-plan-report",
        "source_paths"= [
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
        "schema_blake3"= "6264c981a69074be3666b86282a1e9dcc8e7d87132e39cb4d2595cea6940d308",
        "contract_blake3"= "cd20cb1f42f9c0b91267ba44ba449dfab772996cc5f7efc98c643b7134676546",
        "prelude_blake3"= "496f0c79d7918f1761ccb04a0ce3643954aa2653eeae115068ea67092fda4db5",
        "fixture_set_blake3"= "326f83bf601b7aaf442cadcde8c09e8274e606b510e83a7b327cb8e1debc45a0",
        "producer_identity_blake3"= "660917bd30d3d28571b6f522822866d8d613d119ee15bfe2bea0cffe038e864e",
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
        "contract_blake3"= "2dbee0e6faf2eaa07243e7be7b1ad4f8e415efd0bf60bc3db72348ee1115d1fa",
        "prelude_blake3"= "496f0c79d7918f1761ccb04a0ce3643954aa2653eeae115068ea67092fda4db5",
        "fixture_set_blake3"= "f8efd884abc9b6beffd9527db139ab53356b198331c91adb4b41259ca6684c88",
        "producer_identity_blake3"= "b26610924a3c05af55c432d2a3a9377ce2edd3e89cd38fe37a3b723d95c685c1",
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
        "contract_blake3"= "eaab54bca79c41b817f96c198ebb511a4d1710602395e5cd704ca02e73363d35",
        "prelude_blake3"= "496f0c79d7918f1761ccb04a0ce3643954aa2653eeae115068ea67092fda4db5",
        "fixture_set_blake3"= "a9cd57d937da9107de66d87888629c51f19a07dc585beda5192baa817bf77ec9",
        "producer_identity_blake3"= "5a2e8649734f8d0ccaa3d293beff4741426d17e9c20e7f9040d9e45d86c7bfff",
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
        "contract_blake3"= "54fbc926003092124a7a4a88eb1fc29c43a274dab9e72fe082230138163e570a",
        "prelude_blake3"= "496f0c79d7918f1761ccb04a0ce3643954aa2653eeae115068ea67092fda4db5",
        "fixture_set_blake3"= "6788977d15737936e07f67f8cc3ee1c5fd7acbe275762801e54e680681de19d2",
        "producer_identity_blake3"= "22dd4da2bf0f447d98e3ddc9b66da79e684f89aa3e2f2e55266e526a56e9339e",
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
        "contract_blake3"= "1b298a1327e9ab47c6bdb31c1b0efe373bcc77dbb868e94b004e956593e80bbd",
        "prelude_blake3"= "496f0c79d7918f1761ccb04a0ce3643954aa2653eeae115068ea67092fda4db5",
        "fixture_set_blake3"= "996d0ea5521663d8180e90e955fcba9c92b4570b8935e13b2e57af45c81260e2",
        "producer_identity_blake3"= "53dfeaa242f411594a037e984610c6751f84d077a01e95a6ca72b53140300a17",
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
        "contract_blake3"= "82d26b857396ae116cc3d245dc6f3f6edd37fcad4417a02e65f69f11b4a86339",
        "prelude_blake3"= "496f0c79d7918f1761ccb04a0ce3643954aa2653eeae115068ea67092fda4db5",
        "fixture_set_blake3"= "1ad119357b7d7217396fe2f47598b14cb8d87bc105982a428e3924e7c3213c11",
        "producer_identity_blake3"= "9f44162362c6a2b01f67d2b84d96b9170130330edd3ed78189a8d9aea1e7786c",
        "consumer_policy_blake3"= "edc96af15dd9073d7cd0178ed0d617acd41c8cf91c6719a674dd1422a7e285e4"
      },
      "non_claims"= [
        "readiness shape does not prove source provenance or downstream success"
      ],
      "rationale"= "Stable source verification decision consumed by offline workflows."
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
        "contract_blake3"= "b3cbddb94afb0b1d25aeacee4dc02932548be98c2d32d551a2579ea17d18938b",
        "prelude_blake3"= "496f0c79d7918f1761ccb04a0ce3643954aa2653eeae115068ea67092fda4db5",
        "fixture_set_blake3"= "74b65baffbfcfb59d5557a99ea0ed63c02711ee4c865c5f7be1eb14ad57071c8",
        "producer_identity_blake3"= "6467468a25dd31623a415dffc06c6e3ce7d559c0310a17436b3fbce8744fc6af",
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
        "contract_blake3"= "4ff8d00ce8180be6207e7a87bb53b210b4eb8420826c75cbf1977ebd5a33c5c4",
        "prelude_blake3"= "496f0c79d7918f1761ccb04a0ce3643954aa2653eeae115068ea67092fda4db5",
        "fixture_set_blake3"= "3cf4824049a7e2294fe7aa79115d67f79739889171e3e022cb7c07da622af610",
        "producer_identity_blake3"= "73b784a1b0c2b3408ddebc585bbf41357ae177d109455ed1cd799d47c635d570",
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
          "src/nickel_export.rs"
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
        "contract_blake3"= "0ebfa8dc10228f19a76f97e5afd406b55c2c5df5ba8358a02d58bc77f1485e7d",
        "prelude_blake3"= "496f0c79d7918f1761ccb04a0ce3643954aa2653eeae115068ea67092fda4db5",
        "fixture_set_blake3"= "e39044713ad33d6d6153018742b25fa6b1da7c3b5b013537854684078292a2c8",
        "producer_identity_blake3"= "28e141869781254076884f49dbab779120a2c004b5b4289080868aaf71c7064c",
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
        "schema_blake3"= "24541155258a9ec67af44b95848176c35e9823fa977ebd64fa24bbe762c9f271",
        "contract_blake3"= "4f12c53d77531fcac374d86b0bf5df1e1c9d6b854b22d1a73a5bbd177abae980",
        "prelude_blake3"= "496f0c79d7918f1761ccb04a0ce3643954aa2653eeae115068ea67092fda4db5",
        "fixture_set_blake3"= "67d9e9232706f73f157c68abea93383ffa5af8ef83a924ecc2fb8fadce10a357",
        "producer_identity_blake3"= "da3cb30f7be5aeba2634b3f6b3afc0aa73063722b9c26b97bded402723839045",
        "consumer_policy_blake3"= "fce9cfc511ed3931f6804f5e99cd0ae3d6d325a2a6e47899b8f027ccaa6787ec"
      },
      "non_claims"= [
        "shape and digest syntax do not prove attestation truth, signer trust, reproducibility, or release eligibility"
      ],
      "rationale"= "Selected stable release handoff envelope crosses publisher, witness, and verifier boundaries."
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
          "src/foreign_derivation_import.rs"
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
        "cargo test -p mantle pin_import::tests"
      ],
      "fixture_coverage"= [
        "classification"
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
        "classification"
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
      "rationale"= "Policy and witness reports remain compatibility surfaces under their owning release work."
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
          "src/release_source.rs"
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
          "src/source_root_provider.rs",
          "src/rust_source_provider.rs",
          "src/rust_bootstrap_patch_plan.rs",
          "src/source_toolchain_closure.rs",
          "src/native_toolchain_closure.rs",
          "src/vendor_source_manifest.rs"
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
      "rust_owner"= "src/remote_build.rs::public_json_family",
      "producer"= {
        "command_or_api"= "mantle remote build/status/serve --json",
        "marker"= "remote.execution-reports",
        "source_paths"= [
          "src/remote_build.rs"
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
        "cargo test -p mantle remote_build::tests"
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
        "remote status does not prove output admission or exactly-once execution"
      ],
      "rationale"= "Current remote reports stay under the active remote execution family."
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
      "rust_owner"= "src/remote_build.rs::PlannedRemoteObservabilityArtifacts",
      "producer"= {
        "command_or_api"= "planned attempt logs and telemetry events",
        "marker"= "remote.observability-future",
        "source_paths"= [
          "src/remote_build.rs"
        ]
      },
      "consumers"= [
        "persist-remote-attempt-observability change"
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
        "cairn validate persist-remote-attempt-observability"
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
        "registration does not define telemetry authority or metric labels"
      ],
      "rationale"= "Registers the active observability surface without freezing its design."
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
        "command_or_api"= "planned Wasm component reports and materialization bundle",
        "marker"= "wasm.component-future",
        "source_paths"= [
          "src/build_correctness.rs"
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
        "current"= "planned",
        "supported"= [],
        "unknown_version_behavior"= "owning change must version before exposure",
        "compatibility_converter"= "",
        "compatibility_fixtures"= []
      },
      "validation_commands"= [
        "cairn validate add-wasm-component-build-pipeline"
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
        "registration does not claim component pipeline implementation or Octet acceptance"
      ],
      "rationale"= "Registers future component reports and bundle without constraining the active owner."
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
    }
  ]
}
# registry-data-end
| Registry
