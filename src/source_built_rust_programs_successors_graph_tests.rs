use super::*;
use crate::source_built_recipe::RecipePreparation;
use crate::source_built_recipe::SCHEMA;
use crate::source_built_recipe::SourceRecipe;
use crate::source_built_recipe::validate_source_recipe;

fn marker_recipe(ids: &[String]) -> SourceRecipe {
    SourceRecipe {
        schema: SCHEMA.into(),
        stage_id: STAGE.into(),
        preparation: RecipePreparation {
            tool: "successor-label-fixture".into(),
            provenance: "source-only identity admission regression".into(),
        },
        steps: ids
            .iter()
            .map(|id| RecipeStep {
                id: id.clone(),
                inputs: BTreeMap::new(),
                outputs: BTreeMap::from([("text".into(), super::super::spec(ArtifactKind::Text))]),
                operation: RecipeOperation::WriteText {
                    output: "text".into(),
                    text: "source-only marker".into(),
                },
            })
            .collect(),
        timeout_groups: Vec::new(),
        roots: ids.iter().map(|id| product(id, "text")).collect(),
        exports: BTreeMap::new(),
        semantic_roots: BTreeMap::new(),
    }
}

#[test]
fn build_metadata_and_prerelease_versions_keep_distinct_admitted_producers() {
    let raw = marker_recipe(&["compiler.native-sys.1.0.0+backend.2.target".into()]);
    assert!(validate_source_recipe(&raw).is_err());
    let ids = ["1.0.0+backend.2", "1.0.0-backend.2"]
        .map(|version| format!("compiler.native-sys.{}.target", label_component(version).unwrap()));
    let recipe = marker_recipe(&ids);
    validate_source_recipe(&recipe).unwrap();
    assert_ne!(recipe.roots[0], recipe.roots[1]);
}

#[test]
fn combined_crate_types_and_literal_escape_tokens_do_not_alias() {
    let raw = marker_recipe(&["std.std.0.0.0.target.lib.rlib,dylib".into()]);
    assert!(validate_source_recipe(&raw).is_err());
    let ids = ["rlib,dylib", "rlib_2cdylib"]
        .map(|kind| format!("std.std.0.0.0.target.lib.{}", label_component(kind).unwrap()));
    let recipe = marker_recipe(&ids);
    validate_source_recipe(&recipe).unwrap();
    assert_ne!(recipe.roots[0], recipe.roots[1]);
}
