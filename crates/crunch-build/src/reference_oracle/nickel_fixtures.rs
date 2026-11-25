//! `#40` Comment 1 Nickel graph construction & subgraph composition fixtures.
//!
//! Exercises `crunch_eval` -> `crunch_glue::convert` -> `populate_registry` ->
//! `Worker` + `Builder` + `DeterministicFakeBuildService` across inline records,
//! imported constructor modules, composed higher-order graph functions,
//! `examples/diamond-dependency.ncl`, `mantle.select`, provenance preservation,
//! order-permutation invariance, and boundary-attributed negative tests.

use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::ffi::OsString;
use std::path::PathBuf;

use crunch_eval::Error as EvalError;
use crunch_eval::evaluate_and_extract_named_roots;
use crunch_eval::evaluate_str;
use crunch_eval::evaluate_str_and_extract_named_roots;
use crunch_eval::evaluate_str_to_json;
use crunch_eval::session::EvaluationSession;
use crunch_eval::stdlib::stdlib_import_path;
use crunch_glue::ConversionCache;
use crunch_glue::CrunchDerivation;
use crunch_glue::Error as GlueError;
use crunch_glue::convert;
use nix_compat::derivation::Derivation;
use nix_compat::store_path::StorePath;
use snix_castore::Node;
use tokio::sync::mpsc;

use super::harness::InstrumentedEnv;
use crate::DerivationRegistry;
use crate::EvalMessage;
use crate::Worker;
use crate::populate_registry;
use crate::test_support::FakeBuildProgram;
use crate::test_support::FakeOutputProgram;
use crate::test_support::MANTLE_REF_PROG_ARG_PREFIX;
use crate::worker::WatchWorkerEvent;
use crate::worker::WatchWorkerUpdate;

fn stdlib_imports() -> Vec<OsString> {
    vec![stdlib_import_path().expect("stdlib import path").into_os_string()]
}

fn prog_arg(task_name: &str, salt: u64, outputs: &[&str], expected_inputs: &[&str]) -> String {
    let mut output_programs = BTreeMap::new();
    for out_name in outputs {
        output_programs.insert((*out_name).to_string(), FakeOutputProgram {
            salt,
            ca_content_override: None,
            embed_self_output_ref: false,
            ref_sibling_outputs: Vec::new(),
            ref_input_names: Vec::new(),
            ref_inputs: Vec::new(),
            dynamic_plan_branches: None,
        });
    }
    let program = FakeBuildProgram {
        task_name: task_name.to_string(),
        expected_input_names: expected_inputs.iter().map(|s| (*s).to_string()).collect(),
        expected_inputs: Vec::new(),
        ignore_plan_inputs: false,
        output_programs,
        fail_message: None,
        omit_outputs: Vec::new(),
    };
    let json = serde_json::to_string(&program).expect("serialize FakeBuildProgram");
    format!("{MANTLE_REF_PROG_ARG_PREFIX}{json}")
}

type ConvertedRoot = (StorePath<String>, Derivation, [u8; 32]);
type ExecutedGraph = (Vec<ConvertedRoot>, BTreeMap<String, Vec<u8>>, u32);

fn convert_and_execute_roots(roots: &[(String, CrunchDerivation)], store_dir: &str) -> ExecutedGraph {
    let mut cache = ConversionCache::new(store_dir);
    let mut registry = DerivationRegistry::new(store_dir);
    let mut converted = Vec::with_capacity(roots.len());
    for (_, drv) in roots {
        let (sp, nix_drv) = convert(drv, &mut cache).expect("convert nickel root");
        let hdm = cache.get_hdm_by_drv_path(&sp.to_absolute_path_with_prefix(store_dir)).expect("hdm present");
        populate_registry(&mut registry, cache.drain_pending());
        converted.push((sp, nix_drv, hdm));
    }

    let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().expect("tokio rt");
    let (outputs_by_name, executor_calls) = rt.block_on(async {
        let env = InstrumentedEnv::new(store_dir);
        let mut builder = env.make_builder(false);
        let mut worker = Worker::new(2);
        for (sp, _, _) in &converted {
            worker.want(sp, &registry, true).expect("worker.want");
        }
        let res = worker.run(&mut builder, &mut registry).await.expect("worker.run");
        assert!(res.failed.is_empty(), "unexpected failures: {:?}", res.failed);

        let mut bytes_map = BTreeMap::new();
        for outcome in &res.outcomes {
            for (out_name, pi) in &outcome.outputs {
                let Node::File { digest, .. } = &pi.node else {
                    panic!("expected Node::File");
                };
                let bytes = env.blobs.read_all_bytes_uncounted(digest).await.expect("read blob").expect("blob present");
                bytes_map.insert(format!("{}:{out_name}", outcome.drv_path.name()), bytes);
            }
        }
        (bytes_map, env.metrics().executor_calls)
    });

    (converted, outputs_by_name, executor_calls)
}

fn collect_all_entries_sorted(
    roots: &[(String, CrunchDerivation)],
    store_dir: &str,
) -> Vec<(String, Vec<u8>, [u8; 32])> {
    let mut cache = ConversionCache::new(store_dir);
    for (_, drv) in roots {
        let _ = convert(drv, &mut cache).expect("convert root");
    }
    let mut entries: Vec<_> = cache
        .iter_entries()
        .map(|(sp, hdm, drv, _, _, _)| {
            (
                sp.name().to_string(),
                sp.to_absolute_path_with_prefix(store_dir),
                drv.to_aterm_bytes_with_store_dir(store_dir),
                hdm,
            )
        })
        .collect();
    entries.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)));
    entries.into_iter().map(|(_, path, aterm, hdm)| (path, aterm, hdm)).collect()
}

#[test]
fn nickel_inline_imported_and_composed_diamond_equivalence() {
    let shared_arg = prog_arg("diamond-shared", 10, &["out"], &[]);
    let left_arg = prog_arg("diamond-left", 20, &["out"], &["diamond-shared"]);
    let right_arg = prog_arg("diamond-right", 30, &["out"], &["diamond-shared"]);
    let top_arg = prog_arg("diamond-result", 40, &["out"], &["diamond-left", "diamond-right"]);

    // (a) Inline nested record graph (no helper functions).
    let inline_ncl = format!(
        r#"
let mantle = import "lib.ncl" in
let shared = {{
  name = "diamond-shared",
  builder = "/bin/sh",
  args = [m%"{shared_arg}"%],
}} | mantle.Derivation in
let left = {{
  name = "diamond-left",
  builder = "/bin/sh",
  inputs = [shared],
  args = [m%"{left_arg}"%],
}} | mantle.Derivation in
let right = {{
  name = "diamond-right",
  builder = "/bin/sh",
  inputs = [shared],
  args = [m%"{right_arg}"%],
}} | mantle.Derivation in
{{
  name = "diamond-result",
  builder = "/bin/sh",
  inputs = [left, right],
  args = [m%"{top_arg}"%],
}} | mantle.Derivation
"#
    );

    // (b) Imported constructor module in a TempDir.
    let tmp = tempfile::tempdir().expect("tempdir");
    let ctor_path = tmp.path().join("constructors.ncl");
    let ctor_ncl = r#"
let mantle = import "lib.ncl" in
{
  mk_leaf = fun pkg_name prog => {
    name = pkg_name,
    builder = "/bin/sh",
    args = [prog],
  } | mantle.Derivation,

  mk_branch = fun pkg_name dep prog => {
    name = pkg_name,
    builder = "/bin/sh",
    inputs = [dep],
    args = [prog],
  } | mantle.Derivation,

  mk_join = fun pkg_name left_dep right_dep prog => {
    name = pkg_name,
    builder = "/bin/sh",
    inputs = [left_dep, right_dep],
    args = [prog],
  } | mantle.Derivation,
}
"#;
    std::fs::write(&ctor_path, ctor_ncl).expect("write constructors.ncl");

    let imported_main_path = tmp.path().join("main.ncl");
    let imported_main_ncl = format!(
        r#"
let c = import "constructors.ncl" in
let shared = c.mk_leaf "diamond-shared" m%"{shared_arg}"% in
let left = c.mk_branch "diamond-left" shared m%"{left_arg}"% in
let right = c.mk_branch "diamond-right" shared m%"{right_arg}"% in
c.mk_join "diamond-result" left right m%"{top_arg}"%
"#
    );
    std::fs::write(&imported_main_path, imported_main_ncl).expect("write main.ncl");

    // (c) Composed higher-order graph combinators.
    let composed_ncl = format!(
        r#"
let mantle = import "lib.ncl" in
let compose_diamond = fun mk_base mk_left mk_right mk_top =>
  let base = mk_base {{}} in
  let l = mk_left base in
  let r = mk_right base in
  mk_top l r
in
let step = fun pkg_name prog deps => {{
  name = pkg_name,
  builder = "/bin/sh",
  inputs = deps,
  args = [prog],
}} | mantle.Derivation in
compose_diamond
  (fun _ => step "diamond-shared" m%"{shared_arg}"% [])
  (fun base => step "diamond-left" m%"{left_arg}"% [base])
  (fun base => step "diamond-right" m%"{right_arg}"% [base])
  (fun l r => step "diamond-result" m%"{top_arg}"% [l, r])
"#
    );

    let imports = stdlib_imports();
    let inline_roots =
        evaluate_str_and_extract_named_roots::<CrunchDerivation>(&inline_ncl, &imports).expect("eval inline");
    let imported_roots =
        evaluate_and_extract_named_roots::<CrunchDerivation>(&imported_main_path, &imports).expect("eval imported");
    let composed_roots =
        evaluate_str_and_extract_named_roots::<CrunchDerivation>(&composed_ncl, &imports).expect("eval composed");

    let (inline_conv, inline_out, inline_calls) = convert_and_execute_roots(&inline_roots, "/nix/store");
    let (imported_conv, imported_out, imported_calls) = convert_and_execute_roots(&imported_roots, "/nix/store");
    let (composed_conv, composed_out, composed_calls) = convert_and_execute_roots(&composed_roots, "/nix/store");

    assert_eq!(inline_conv[0].0, imported_conv[0].0);
    assert_eq!(inline_conv[0].0, composed_conv[0].0);
    assert_eq!(inline_conv[0].1, imported_conv[0].1);
    assert_eq!(inline_conv[0].1, composed_conv[0].1);
    assert_eq!(inline_out, imported_out);
    assert_eq!(inline_out, composed_out);
    assert_eq!(inline_calls, 4, "shared node must execute exactly once");
    assert_eq!(imported_calls, 4);
    assert_eq!(composed_calls, 4);

    // Full ConversionCache entries & ATerm bytes match identically across all 3 forms.
    let entries_a = collect_all_entries_sorted(&inline_roots, "/nix/store");
    let entries_b = collect_all_entries_sorted(&imported_roots, "/nix/store");
    let entries_c = collect_all_entries_sorted(&composed_roots, "/nix/store");
    assert_eq!(entries_a.len(), 4);
    assert_eq!(entries_a, entries_b);
    assert_eq!(entries_a, entries_c);

    // Store-prefix sensitivity: same graph under /opt/mantle/store yields distinct drv_path, same HDM!
    let alt_entries = collect_all_entries_sorted(&inline_roots, "/opt/mantle/store");
    assert_eq!(alt_entries.len(), 4);
    for (nix_entry, opt_entry) in entries_a.iter().zip(&alt_entries) {
        assert_ne!(nix_entry.0, opt_entry.0, "drv_path must reflect store_prefix");
        assert_eq!(nix_entry.2, opt_entry.2, "HDM must be store-prefix invariant");
    }
}

#[test]
fn nickel_examples_diamond_dependency_and_evaluation_session() {
    let example_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/diamond-dependency.ncl");
    let imports = stdlib_imports();
    let roots = evaluate_and_extract_named_roots::<CrunchDerivation>(&example_path, &imports)
        .expect("eval examples/diamond-dependency.ncl");
    assert_eq!(roots.len(), 1);
    assert_eq!(roots[0].0, "diamond-result");

    let mut cache = ConversionCache::new("/nix/store");
    let (drv_path, nix_drv) = convert(&roots[0].1, &mut cache).expect("convert example");
    assert_eq!(nix_drv.input_derivations.len(), 2);
    let entries = collect_all_entries_sorted(&roots, "/nix/store");
    assert_eq!(entries.len(), 4, "shared leaf + left + right + result");

    // Multi-root EvaluationSession parity check with parameterized constructors & sharing.
    let shared_arg = prog_arg("shared-base", 1, &["out"], &[]);
    let alpha_arg = prog_arg("pkg-alpha", 2, &["out"], &["shared-base"]);
    let beta_arg = prog_arg("pkg-beta", 3, &["out"], &["shared-base"]);
    let multi_ncl = format!(
        r#"
let mantle = import "lib.ncl" in
let mk_shared = fun _ => {{
  name = "shared-base",
  builder = "/bin/sh",
  args = [m%"{shared_arg}"%],
}} | mantle.Derivation in
let mk_pkg = fun pkg_name prog => {{
  name = pkg_name,
  builder = "/bin/sh",
  inputs = [mk_shared {{}}],
  args = [prog],
}} | mantle.Derivation in
{{
  alpha = mk_pkg "pkg-alpha" m%"{alpha_arg}"%,
  beta = mk_pkg "pkg-beta" m%"{beta_arg}"%,
}}
"#
    );
    let mut session = EvaluationSession::open_str(&multi_ncl, &imports).expect("open session");
    let session_roots = session.force_all_roots::<CrunchDerivation>().expect("force_all_roots");
    let direct_roots =
        evaluate_str_and_extract_named_roots::<CrunchDerivation>(&multi_ncl, &imports).expect("direct extract");

    let (conv_s, out_s, calls_s) = convert_and_execute_roots(&session_roots, "/nix/store");
    let (conv_d, out_d, calls_d) = convert_and_execute_roots(&direct_roots, "/nix/store");
    assert_eq!(conv_s.len(), 2);
    assert_ne!(conv_s[0].0, conv_s[1].0);
    assert_eq!(conv_s, conv_d);
    assert_eq!(out_s, out_d);
    assert_eq!(calls_s, 3, "repeated identical constructor mk_shared {{}} must share once (total 3 builds)");
    assert_eq!(calls_d, 3);
    let _ = drv_path;
}

#[test]
fn nickel_selected_outputs_and_constructor_elimination() {
    let sdk_arg = prog_arg("sdk-multi", 55, &["out", "dev"], &[]);
    let use_dev_arg = prog_arg("use-dev", 56, &["out"], &["sdk-multi-dev"]);
    let use_both_arg = prog_arg("use-both", 57, &["out"], &["sdk-multi", "sdk-multi-dev"]);

    let ncl_src = format!(
        r#"
let mantle = import "lib.ncl" in
let helper_pipe = fun x => x in
let sdk = {{
  name = "sdk-multi",
  builder = "/bin/sh",
  outputs = ["out", "dev"],
  args = [m%"{sdk_arg}"%],
}} | mantle.Derivation in
let mk_consumer = fun pkg_name sel_list prog =>
  ({{
    name = pkg_name,
    builder = "/bin/sh",
    inputs = sel_list,
    args = [prog],
  }} |> helper_pipe) | mantle.Derivation
in
{{
  dev_only = mk_consumer "use-dev" [mantle.select sdk "dev"] m%"{use_dev_arg}"%,
  both = mk_consumer "use-both" [mantle.select sdk "out", mantle.select sdk "dev"] m%"{use_both_arg}"%,
}}
"#
    );

    let imports = stdlib_imports();
    // Constructor elimination: evaluate_str_to_json succeeds with zero functions and strips
    // not_exported.
    let elim_check_ncl = format!(
        r#"
let helper_pipe = fun x => x in
{{
  pkg = ({{
    name = "elim-check",
    builder = "/bin/sh",
    args = [m%"{sdk_arg}"%],
    internal_hint | not_exported = "never-seen",
  }} |> helper_pipe),
}}
"#
    );
    let elim_json = evaluate_str_to_json(&elim_check_ncl, &imports).expect("not_exported stripped");
    assert!(!elim_json.contains("internal_hint"), "not_exported field must be stripped");

    let json_text = evaluate_str_to_json(&ncl_src, &imports).expect("constructor elimination to JSON");
    assert!(json_text.contains("use-dev"));

    let roots = evaluate_str_and_extract_named_roots::<CrunchDerivation>(&ncl_src, &imports)
        .expect("extract selected-output roots");
    let (conv, out, calls) = convert_and_execute_roots(&roots, "/nix/store");
    assert_eq!(conv.len(), 2);
    // dev_only has single input_derivation entry with {"dev"}
    let dev_only_sets: Vec<&BTreeSet<String>> = conv[0].1.input_derivations.values().collect();
    assert_eq!(dev_only_sets.len(), 1);
    assert_eq!(dev_only_sets[0], &BTreeSet::from(["dev".to_string()]));
    // both coalesces repeated selections on same drv into {"out", "dev"}
    let both_sets: Vec<&BTreeSet<String>> = conv[1].1.input_derivations.values().collect();
    assert_eq!(both_sets.len(), 1);
    assert_eq!(both_sets[0], &BTreeSet::from(["out".to_string(), "dev".to_string()]));
    assert_eq!(calls, 3, "shared sdk-multi executes once + 2 consumers");
    assert!(out.contains_key("use-dev.drv:out"), "got keys {:?}", out.keys().collect::<Vec<_>>());
    assert!(out.contains_key("use-both.drv:out"), "got keys {:?}", out.keys().collect::<Vec<_>>());
}

#[test]
fn nickel_order_permutation_invariance() {
    let a_arg = prog_arg("ord-a", 101, &["out"], &[]);
    let b_arg = prog_arg("ord-b", 102, &["out"], &[]);
    let join_arg = prog_arg("ord-join", 103, &["out"], &["ord-a", "ord-b"]);

    let order_1 = format!(
        r#"
let mantle = import "lib.ncl" in
let a = {{ name = "ord-a", builder = "/bin/sh", args = [m%"{a_arg}"%] }} | mantle.Derivation in
let b = {{ name = "ord-b", builder = "/bin/sh", args = [m%"{b_arg}"%] }} | mantle.Derivation in
{{
  first = a,
  second = {{ name = "ord-join", builder = "/bin/sh", inputs = [a, b], args = [m%"{join_arg}"%] }} | mantle.Derivation,
}}
"#
    );
    let order_2 = format!(
        r#"
let mantle = import "lib.ncl" in
let a = {{ name = "ord-a", builder = "/bin/sh", args = [m%"{a_arg}"%] }} | mantle.Derivation in
let b = {{ name = "ord-b", builder = "/bin/sh", args = [m%"{b_arg}"%] }} | mantle.Derivation in
{{
  second = {{ name = "ord-join", builder = "/bin/sh", inputs = [b, a], args = [m%"{join_arg}"%] }} | mantle.Derivation,
  first = a,
}}
"#
    );

    let imports = stdlib_imports();
    let roots_1 = evaluate_str_and_extract_named_roots::<CrunchDerivation>(&order_1, &imports).expect("order 1");
    let roots_2 = evaluate_str_and_extract_named_roots::<CrunchDerivation>(&order_2, &imports).expect("order 2");

    let entries_1 = collect_all_entries_sorted(&roots_1, "/nix/store");
    let entries_2 = collect_all_entries_sorted(&roots_2, "/nix/store");
    assert_eq!(entries_1, entries_2, "root/input ordering must preserve converted derivations");

    let (_, out_1, _) = convert_and_execute_roots(&roots_1, "/nix/store");
    let (_, out_2, _) = convert_and_execute_roots(&roots_2, "/nix/store");
    assert_eq!(out_1, out_2, "root/input ordering must preserve realized outputs");
}

#[test]
fn nickel_distinct_provenance_preservation_and_contract_boundary() {
    let leaf_arg = prog_arg("prov-leaf", 700, &["out"], &[]);
    let top_arg = prog_arg("prov-top", 701, &["out"], &["prov-leaf"]);

    // Note: lib/derivation.ncl Shape contract does not include `provenance`, so
    // Nickel derivations carrying inline `provenance` omit `| mantle.Derivation`.
    let mk_prov_ncl = |supplier: &str, alias: &str| {
        format!(
            r#"
let leaf = {{
  name = "prov-leaf",
  builder = "/bin/sh",
  addressing_mode = "input-addressed",
  args = [m%"{leaf_arg}"%],
  provenance = {{
    supplier = "{supplier}-leaf",
    source_aliases = ["{alias}-leaf"],
  }},
}} in
{{
  name = "prov-top",
  builder = "/bin/sh",
  addressing_mode = "input-addressed",
  inputs = [leaf],
  args = [m%"{top_arg}"%],
  provenance = {{
    supplier = "{supplier}-top",
    homepage = "https://example.invalid/{supplier}",
    source_aliases = ["{alias}-top"],
  }},
}}
"#
        )
    };

    let imports = stdlib_imports();
    let roots_a = evaluate_str_and_extract_named_roots::<CrunchDerivation>(&mk_prov_ncl("sup-a", "origin"), &imports)
        .expect("eval prov A");
    let roots_b = evaluate_str_and_extract_named_roots::<CrunchDerivation>(&mk_prov_ncl("sup-b", "mirror"), &imports)
        .expect("eval prov B");

    let mut cache_a = ConversionCache::new("/nix/store");
    let (sp_a, nix_a) = convert(&roots_a[0].1, &mut cache_a).expect("convert prov A");
    let mut reg_a = DerivationRegistry::new("/nix/store");
    populate_registry(&mut reg_a, cache_a.drain_pending());

    let mut cache_b = ConversionCache::new("/nix/store");
    let (sp_b, nix_b) = convert(&roots_b[0].1, &mut cache_b).expect("convert prov B");
    let mut reg_b = DerivationRegistry::new("/nix/store");
    populate_registry(&mut reg_b, cache_b.drain_pending());

    assert_eq!(sp_a, sp_b, "provenance must not alter drv_path");
    assert_eq!(nix_a, nix_b, "provenance must not alter nix_compat::Derivation");

    let entry_a = reg_a.get_by_drv_path(&sp_a.to_absolute_path()).expect("entry A");
    let entry_b = reg_b.get_by_drv_path(&sp_b.to_absolute_path()).expect("entry B");
    assert_ne!(entry_a.provenance_claims, entry_b.provenance_claims, "distinct provenance claims must survive");
    assert_eq!(entry_a.provenance_claims.as_ref().and_then(|c| c.supplier.as_deref()), Some("sup-a-top"));
    assert_eq!(entry_b.provenance_claims.as_ref().and_then(|c| c.supplier.as_deref()), Some("sup-b-top"));

    // Verify Builder::build_artifact_provenance preserves distinct claims.
    let env_a = InstrumentedEnv::new("/nix/store");
    let builder_a = env_a.make_builder(false);
    let art_a = builder_a
        .build_artifact_provenance(&entry_a.derivation, &sp_a, &[], &reg_a)
        .expect("artifact provenance A");
    let env_b = InstrumentedEnv::new("/nix/store");
    let builder_b = env_b.make_builder(false);
    let art_b = builder_b
        .build_artifact_provenance(&entry_b.derivation, &sp_b, &[], &reg_b)
        .expect("artifact provenance B");
    assert_ne!(art_a.claims, art_b.claims);

    // Verify lib/derivation.ncl closed Shape rejects unknown `provenance` when `| mantle.Derivation` is
    // applied.
    let contracted_with_prov = r#"
let mantle = import "lib.ncl" in
{
  name = "prov-contracted",
  builder = "/bin/sh",
  provenance = { supplier = "x" },
} | mantle.Derivation
"#;
    let contract_err = match evaluate_str(contracted_with_prov, &imports) {
        Ok(_) => panic!("closed mantle.Derivation Shape rejects undeclared provenance field"),
        Err(err) => err,
    };
    assert!(matches!(contract_err, EvalError::Eval(_)));
}

#[test]
fn nickel_boundary_attributed_negative_rejections() {
    let imports = stdlib_imports();

    // 1. Missing output selection -> passes Nickel contract, rejected at crunch_glue::convert!
    let missing_out_ncl = r#"
let mantle = import "lib.ncl" in
let dep = {
  name = "multi-dep",
  builder = "/bin/sh",
  outputs = ["out", "dev"],
} | mantle.Derivation in
{
  name = "bad-consumer",
  builder = "/bin/sh",
  inputs = [mantle.select dep "headers"],
} | mantle.Derivation
"#;
    let roots = evaluate_str_and_extract_named_roots::<CrunchDerivation>(missing_out_ncl, &imports)
        .expect("Nickel select passes contract");
    let mut cache = ConversionCache::new("/nix/store");
    let err = convert(&roots[0].1, &mut cache).expect_err("missing output rejected at crunch-glue");
    assert!(
        matches!(err, GlueError::InvalidOutputSelection { ref output, .. } if output == "headers"),
        "unexpected error: {err:?}"
    );

    // 2. Accidental collisions:
    // (a) Same name/builder/system with different args from parameterized constructor -> NO collision!
    let diff_args_ncl = r#"
let mantle = import "lib.ncl" in
let mk = fun msg => {
  name = "same-name",
  builder = "/bin/sh",
  args = ["-c", msg],
} | mantle.Derivation in
{ a = mk "one", b = mk "two" }
"#;
    let diff_roots =
        evaluate_str_and_extract_named_roots::<CrunchDerivation>(diff_args_ncl, &imports).expect("eval diff args");
    let mut cache2 = ConversionCache::new("/nix/store");
    let (sp_1, _) = convert(&diff_roots[0].1, &mut cache2).expect("conv 1");
    let (sp_2, _) = convert(&diff_roots[1].1, &mut cache2).expect("conv 2");
    assert_ne!(sp_1, sp_2, "differing constructor args must produce distinct drv_paths");

    // (b) Conflicting dynamic_plan_outputs on identical structural recipe ->
    // crunch_glue::InvalidDynamicPlanOutputs!
    let conflict_dyn_ncl = r#"
let mantle = import "lib.ncl" in
{
  a = {
    name = "dyn-conflict",
    builder = "/bin/sh",
    outputs = ["out", "plan"],
    dynamic_plan_outputs = [],
  } | mantle.Derivation,
  b = {
    name = "dyn-conflict",
    builder = "/bin/sh",
    outputs = ["out", "plan"],
    dynamic_plan_outputs = ["plan"],
  } | mantle.Derivation,
}
"#;
    let conflict_roots = evaluate_str_and_extract_named_roots::<CrunchDerivation>(conflict_dyn_ncl, &imports)
        .expect("eval dyn conflict");
    let mut cache3 = ConversionCache::new("/nix/store");
    let _ = convert(&conflict_roots[0].1, &mut cache3).expect("first succeeds");
    let dyn_err = convert(&conflict_roots[1].1, &mut cache3).expect_err("conflicting dynamic_plan_outputs rejected");
    assert!(matches!(dyn_err, GlueError::InvalidDynamicPlanOutputs(_)), "got {dyn_err:?}");

    // (c) Watch collision: same drv_path with differing derivation in WatchWorkerUpdate ->
    // WatchWorkerEvent::Rejected!
    let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().expect("rt");
    rt.block_on(async {
        let env = InstrumentedEnv::new("/nix/store");
        let mut builder = env.make_builder(false);
        let mut worker = Worker::new(1);
        let mut reg = DerivationRegistry::new("/nix/store");
        let (utx, mut urx) = mpsc::channel(2);
        let (etx, mut erx) = mpsc::channel(4);
        let (_, real_nix) = convert(&diff_roots[0].1, &mut ConversionCache::new("/nix/store")).expect("real");
        let mut bad_nix = real_nix.clone();
        bad_nix.arguments.push("mutated".to_string());
        reg.insert(sp_1.clone(), [1u8; 32], real_nix, false, None);
        utx.send(WatchWorkerUpdate {
            generation: 1,
            roots: vec![EvalMessage {
                label: "collision".to_string(),
                drv_path: sp_1.clone(),
                new_entries: vec![(sp_1, [2u8; 32], bad_nix, false, Vec::new(), None)],
            }],
        })
        .await
        .expect("send collision");
        drop(utx);
        let (w_res, ev) = tokio::join!(worker.run_watch(&mut builder, &mut reg, &mut urx, &etx), erx.recv());
        w_res.expect("run_watch handles rejection cleanly");
        assert!(matches!(ev, Some(WatchWorkerEvent::Rejected { generation: 1 })), "got {ev:?}");
    });

    // 3. Cycles:
    // (a) CrunchDerivation identity cycle -> crunch_glue::CircularDependency
    let cyc_drv = CrunchDerivation {
        name: "nickel-cyc".to_string(),
        builder: "/bin/sh".to_string(),
        system: "x86_64-linux".to_string(),
        args: vec!["-c".to_string(), "outer".to_string()],
        outputs: vec!["out".to_string()],
        dynamic_plan_outputs: Vec::new(),
        env: std::collections::HashMap::new(),
        inputs: vec![crunch_glue::Input::Derivation(Box::new(CrunchDerivation {
            name: "nickel-cyc".to_string(),
            builder: "/bin/sh".to_string(),
            system: "x86_64-linux".to_string(),
            args: vec!["-c".to_string(), "inner".to_string()],
            outputs: vec!["out".to_string()],
            dynamic_plan_outputs: Vec::new(),
            env: std::collections::HashMap::new(),
            inputs: Vec::new(),
            fixed_output: None,
            addressing_mode: "input-addressed".to_string(),
            provenance: None,
        }))],
        fixed_output: None,
        addressing_mode: "input-addressed".to_string(),
        provenance: None,
    };
    let cyc_err = convert(&cyc_drv, &mut ConversionCache::new("/nix/store")).expect_err("identity cycle");
    assert!(matches!(cyc_err, GlueError::CircularDependency(_)));

    // (b) Nickel mutual field blackhole -> crunch_eval::Error::Eval
    let blackhole_ncl = r#"
let rec r = {
  a = { name = r.b.name, builder = "/bin/sh" },
  b = { name = r.a.name, builder = "/bin/sh" },
} in r
"#;
    let bh_err = match evaluate_str(blackhole_ncl, &imports) {
        Ok(_) => panic!("Nickel blackhole rejected"),
        Err(err) => err,
    };
    assert!(matches!(bh_err, EvalError::Eval(_)), "got {bh_err:?}");

    // 4. Residual uneliminated functions -> rejected at crunch-eval export/extraction boundary!
    let top_fn = r#"fun x => { name = x, builder = "/bin/sh" }"#;
    assert!(matches!(evaluate_str_to_json(top_fn, &imports), Err(EvalError::Eval(_))));
    assert!(matches!(
        evaluate_str_and_extract_named_roots::<CrunchDerivation>(top_fn, &imports),
        Err(EvalError::Boundary(_))
    ));

    let record_fn = r#"{ pkg = fun x => { name = x, builder = "/bin/sh" } }"#;
    assert!(matches!(
        evaluate_str_and_extract_named_roots::<CrunchDerivation>(record_fn, &imports),
        Err(EvalError::Serde(_))
    ));

    let array_fn = r#"[fun x => { name = x, builder = "/bin/sh" }]"#;
    assert!(matches!(
        evaluate_str_and_extract_named_roots::<CrunchDerivation>(array_fn, &imports),
        Err(EvalError::Boundary(_))
    ));

    let field_fn_contracted = r#"
let mantle = import "lib.ncl" in
{
  name = "bad-fn-field",
  builder = "/bin/sh",
  env = { F = fun x => x },
} | mantle.Derivation
"#;
    assert!(matches!(
        evaluate_str_and_extract_named_roots::<CrunchDerivation>(field_fn_contracted, &imports),
        Err(EvalError::Eval(_))
    ));
}
