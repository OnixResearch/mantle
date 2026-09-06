fn record(identity: &str, root: bool, references: &[&str]) -> crunch_repair_core::legacy_archive::ClosureRecord {
    crunch_repair_core::legacy_archive::ClosureRecord {
        identity: identity.to_string(),
        root,
        references: references.iter().map(|value| (*value).to_string()).collect(),
    }
}

#[test]
fn closed_root_and_reachable_child_pass_but_unreachable_member_fails() {
    let roots = vec!["root".to_string()];
    assert_eq!(
        crunch_repair_core::legacy_archive::validate_closed_archive(roots.clone(), vec![
            record("root", true, &["child"]),
            record("child", false, &[])
        ]),
        Ok(())
    );
    assert_eq!(
        crunch_repair_core::legacy_archive::validate_closed_archive(roots, vec![
            record("root", true, &[]),
            record("child", false, &[])
        ]),
        Err(crunch_repair_core::legacy_archive::ClosureRejection::UnreachableMember)
    );
}

#[test]
fn empty_duplicate_and_excessive_root_guards_keep_the_same_rejection() {
    assert_eq!(
        crunch_repair_core::legacy_archive::validate_closed_archive(vec![], vec![]),
        Err(crunch_repair_core::legacy_archive::ClosureRejection::Bounds)
    );
    assert_eq!(
        crunch_repair_core::legacy_archive::validate_closed_archive(
            vec!["root".to_string(), "root".to_string()],
            vec![record("root", true, &[])]
        ),
        Err(crunch_repair_core::legacy_archive::ClosureRejection::Bounds)
    );
    let roots = (0..65).map(|index| format!("root-{index}")).collect();
    assert_eq!(
        crunch_repair_core::legacy_archive::validate_closed_archive(roots, vec![]),
        Err(crunch_repair_core::legacy_archive::ClosureRejection::Bounds)
    );
}

#[test]
fn exact_record_ceiling_passes_and_excess_is_rejected_before_reachability() {
    let roots: Vec<_> = (0..64).map(|index| format!("root-{index}")).collect();
    let records = roots.iter().map(|name| record(name, true, &[])).collect();
    assert_eq!(crunch_repair_core::legacy_archive::validate_closed_archive(roots, records), Ok(()));
    let records = (0..65).map(|index| record(&format!("root-{index}"), index == 0, &[])).collect();
    assert_eq!(
        crunch_repair_core::legacy_archive::validate_closed_archive(vec!["root-0".to_string()], records),
        Err(crunch_repair_core::legacy_archive::ClosureRejection::Bounds)
    );
}

#[test]
fn malformed_roles_references_and_members_keep_distinct_rejections() {
    let cases = [
        (
            vec![record("root", true, &["child"])],
            crunch_repair_core::legacy_archive::ClosureRejection::MissingMember,
        ),
        (vec![record("root", false, &[])], crunch_repair_core::legacy_archive::ClosureRejection::RootRole),
        (
            vec![record("root", true, &[]), record("root", true, &[])],
            crunch_repair_core::legacy_archive::ClosureRejection::DuplicateMember,
        ),
        (
            vec![record("root", true, &["root", "root"])],
            crunch_repair_core::legacy_archive::ClosureRejection::DuplicateReference,
        ),
    ];
    assert_eq!(cases.len(), 4);
    for (records, rejection) in cases {
        assert_eq!(
            crunch_repair_core::legacy_archive::validate_closed_archive(vec!["root".to_string()], records),
            Err(rejection)
        );
    }
}
