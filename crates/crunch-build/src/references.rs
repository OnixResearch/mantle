//! Reference resolution: refscan needle mapping and Nix closure queries.

use std::collections::{BTreeMap, BTreeSet};

use nix_compat::derivation::Derivation;
use nix_compat::store_path::StorePath;
use snix_castore::Node;

/// Map refscan needle indices back to store path references.
///
/// The needle list is ordered as: [output paths...] ++ [input paths...].
/// Indices in `found_needles` map into that combined list.
pub(crate) fn resolve_references(
    found_needles: &BTreeSet<u64>,
    _all_needles: &[String],
    derivation: &Derivation,
    inputs: &BTreeMap<StorePath<String>, Node>,
) -> Vec<StorePath<String>> {
    let output_paths: Vec<StorePath<String>> = derivation
        .outputs
        .values()
        .filter_map(|o| o.path.clone())
        .collect();
    let input_paths: Vec<StorePath<String>> = inputs.keys().cloned().collect();

    let all_paths: Vec<StorePath<String>> = output_paths
        .into_iter()
        .chain(input_paths.into_iter())
        .collect();

    found_needles
        .iter()
        .filter_map(|&idx| all_paths.get(idx as usize).cloned())
        .collect()
}



#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_drv_for_refs() -> (Derivation, BTreeMap<StorePath<String>, Node>) {
        let mut outputs = BTreeMap::new();
        outputs.insert(
            "out".to_string(),
            nix_compat::derivation::Output {
                path: Some(
                    StorePath::from_absolute_path(
                        b"/nix/store/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa-out",
                    ).unwrap(),
                ),
                ca_hash: None,
            },
        );
        let drv = Derivation {
            arguments: vec![],
            builder: "/bin/sh".to_string(),
            environment: BTreeMap::new(),
            input_derivations: BTreeMap::new(),
            input_sources: BTreeSet::new(),
            outputs,
            system: "x86_64-linux".to_string(),
        };

        let input_path: StorePath<String> = StorePath::from_absolute_path(
            b"/nix/store/bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb-input",
        ).unwrap();
        let mut inputs = BTreeMap::new();
        inputs.insert(
            input_path,
            Node::Symlink {
                target: snix_castore::SymlinkTarget::try_from("x").unwrap(),
            },
        );
        (drv, inputs)
    }

    #[test]
    fn resolve_refs_index_zero_maps_to_output() {
        let (drv, inputs) = make_test_drv_for_refs();
        let found = BTreeSet::from([0u64]);
        let needles = vec!["a".to_string(), "b".to_string()];

        let refs = resolve_references(&found, &needles, &drv, &inputs);
        assert_eq!(refs.len(), 1);
        assert_eq!(
            refs[0].to_absolute_path(),
            "/nix/store/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa-out"
        );
    }

    #[test]
    fn resolve_refs_index_past_outputs_maps_to_input() {
        let (drv, inputs) = make_test_drv_for_refs();
        let found = BTreeSet::from([1u64]);
        let needles = vec!["a".to_string(), "b".to_string()];

        let refs = resolve_references(&found, &needles, &drv, &inputs);
        assert_eq!(refs.len(), 1);
        assert_eq!(
            refs[0].to_absolute_path(),
            "/nix/store/bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb-input"
        );
    }

    #[test]
    fn resolve_refs_out_of_range_ignored() {
        let (drv, inputs) = make_test_drv_for_refs();
        let found = BTreeSet::from([999u64]);
        let needles = vec!["a".to_string()];

        let refs = resolve_references(&found, &needles, &drv, &inputs);
        assert!(refs.is_empty());
    }

    #[test]
    fn resolve_refs_empty_needles() {
        let (drv, inputs) = make_test_drv_for_refs();
        let found = BTreeSet::new();
        let needles: Vec<String> = vec![];

        let refs = resolve_references(&found, &needles, &drv, &inputs);
        assert!(refs.is_empty());
    }
}
