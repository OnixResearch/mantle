# Build a cohort review subject from that cohort's own measured artifacts.
#
#   jq -n \
#     --slurpfile tpl  <template-subject.json> \
#     --slurpfile facts  <review-subject-facts-<cycle>.json> \
#     --slurpfile prep   <cohort>/source-program-preparation.json \
#     --slurpfile plan   <cohort>/source-program-preparation-input-plan.json \
#     --slurpfile complete <complete-source-<cycle>.json> \
#     --arg revision <full revision> --arg name <cycle name> --arg run <run root> \
#     --arg evidence_root <repo-relative cycle evidence dir> --arg stream <ssh stream hash> \
#     -f build-subject.jq
#
# `--arg stream` must be the BLAKE3 of the profile bytes as received over the
# ssh stream from the execution host, not a local read of the remote file.
# The template supplies only cycle-independent constants and historical
# reference paths; every measured identity comes from this cohort's artifacts.
#
# Validated by reproduction: feeding the 70db0868 template and that cycle's own
# facts/preparation/plan/complete artifacts yields the reviewed subject exactly
# (byte BLAKE3 2081b08f4933319c236800f1b709144c2581d1423c6b1d8a9a8077f50a33652d).
$tpl[0] as $template
| $facts[0] as $f
| $prep[0] as $p
| $plan[0] as $pl
| $complete[0] as $c
| {
    host: $template.host,
    run: $run,
    frozen_product_revision: $revision,
    frozen_source: ($run + "/runner-source-" + $name),
    frozen_complete_source_blake3: $c.complete_archive_blake3,
    runner: ($run + "/bin/mantle-" + $name),
    runner_blake3: $f.runner_blake3,
    cohort: ($run + "/prepared-cohort-" + $name),
    evidence_root: $evidence_root,
    input_manifest_blake3: $p.input_profile_blake3,
    input_plan_blake3: $p.input_plan_blake3,
    prepared_manifest_blake3: $p.output_profile_blake3,
    profile_file: ($run + "/prepared-cohort-" + $name + "/source-profile.json"),
    profile_file_bytes: $f.profile_file_bytes,
    profile_file_raw_blake3: $f.profile_file_raw_blake3,
    full_profile_ssh_stream_blake3: $stream,
    profile_transport_equal: ($stream == $f.profile_file_raw_blake3),
    preparation_report: ($run + "/prepared-cohort-" + $name + "/source-program-preparation.json"),
    preparation_report_raw_blake3: $f.preparation_report_raw_blake3,
    input_plan: ($run + "/prepared-cohort-" + $name + "/source-program-preparation-input-plan.json"),
    program_records: $p.program_records,
    source_inputs: $pl.source_inputs,
    resource_bounds: $pl.resource_bounds,
    expected_stagex_lineage_blake3: ($pl.source_inputs[] | select(.id == "stagex-lineage") | .digest_blake3),
    expected_native_provider_blake3: $pl.policies.expected_native_provider_digest_blake3,
    original_aws: {
        capture: ($run + "/aws-original-source-02561621.TGISLN/aws-lc.git"),
        commit: $template.original_aws.commit,
        git_tree_sha1: $template.original_aws.git_tree_sha1,
        normalized_tree_blake3: $template.original_aws.normalized_tree_blake3
    },
    inert_tools: $template.inert_tools,
    prior_reference_inspector: $template.prior_reference_inspector,
    prior_reference_identity_result: $template.prior_reference_identity_result,
    authority: ($template.authority
        | sub("70db08685c0c5fa5c326bfe32db78576d7e2e182"; $revision)
        | sub("28fa8ae91ce978216fba288452219bc0c5aa07a7b57fc6f92a7168863b1b1aab"; $p.output_profile_blake3)),
    prepared_native_source_manifest_raw_blake3: $f.prepared_native_source_manifest_raw_blake3
}
