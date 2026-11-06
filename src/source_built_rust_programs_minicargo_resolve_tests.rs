use super::*;

fn sources() -> Sources {
    Sources {
        top: "fixture".into(),
        manifests: BTreeMap::new(),
        locks: BTreeMap::new(),
        members: BTreeSet::new(),
        policy: BTreeMap::new(),
    }
}

fn package(s: &mut Sources, path: &str, name: &str, version: &str, dependencies: &str) {
    let manifest = format!("[package]\nname = {name:?}\nversion = {version:?}\n[dependencies]\n{dependencies}")
        .parse()
        .unwrap();
    s.manifests.insert(path.into(), manifest);
    s.members.insert(format!("{}/src/lib.rs", directory(path)));
}

fn vendor(s: &mut Sources, name: &str, version: &str, dependencies: &str) {
    package(s, &format!("vendor/{name}-{version}/Cargo.toml"), name, version, dependencies);
}

fn selection(s: &Sources, limit: u32) -> Result<Option<Selection>, RunError> {
    let mut state = Selection::default();
    push(s, &mut state, "root/Cargo.toml")?;
    choose(s, state, None, limit)
}

#[test]
fn late_exact_pin_skips_unrelated_version_choices() {
    let mut s = sources();
    let mut deps = "a = { package = 'tracing-core', version = '0.1' }\n".to_string();
    for index in 0..24 {
        let name = format!("noise{index:02}");
        deps.push_str(&format!("{name} = '1'\n"));
        vendor(&mut s, &name, "1.0.0", "");
        vendor(&mut s, &name, "1.1.0", "");
    }
    deps.push_str("zpin = { path = '../pin' }\n");
    package(&mut s, "root/Cargo.toml", "root", "1.0.0", &deps);
    package(&mut s, "pin/Cargo.toml", "pin", "1.0.0", "tracing-core = '=0.1.30'");
    vendor(&mut s, "tracing-core", "0.1.30", "");
    vendor(&mut s, "tracing-core", "0.1.32", "");
    // Chronological search explores 2^24 unrelated combinations before lowering
    // tracing-core. Dependency-directed rollback must solve this within 128 pops.
    let selected = selection(&s, 128).unwrap().unwrap();
    for (owner, alias) in [("root/Cargo.toml", "a"), ("pin/Cargo.toml", "tracing-core")] {
        assert_eq!(selected.edges[&(owner.into(), alias.into())], "vendor/tracing-core-0.1.30/Cargo.toml");
    }
    for index in 0..24 {
        let name = format!("noise{index:02}");
        assert_eq!(
            selected.edges[&("root/Cargo.toml".into(), name.clone())],
            format!("vendor/{name}-1.1.0/Cargo.toml")
        );
    }
}

#[test]
fn conflict_can_change_requiring_package_through_path_dependencies() {
    let mut s = sources();
    package(&mut s, "root/Cargo.toml", "root", "1.0.0", "a = '1'\nz = '=1.0.0'");
    vendor(&mut s, "a", "1.0.0", "");
    vendor(&mut s, "a", "1.1.0", "nested = { path = '../../nested' }");
    package(&mut s, "nested/Cargo.toml", "nested", "1.0.0", "z = '=1.1.0'");
    vendor(&mut s, "z", "1.0.0", "");
    vendor(&mut s, "z", "1.1.0", "");
    let topology = resolve(&s, &[("root/Cargo.toml", &[])], false).unwrap();
    let versions = topology
        .packages
        .values()
        .map(|p| (p.name.as_str(), p.version.as_str()))
        .collect::<BTreeMap<_, _>>();
    assert_eq!(versions, BTreeMap::from([("root", "1.0.0"), ("a", "1.0.0"), ("z", "1.0.0")]));
    assert!(!topology.packages.keys().any(|key| key.manifest == "nested/Cargo.toml"));
}

#[test]
fn filtered_alternatives_reappear_after_their_blocker_changes() {
    let mut s = sources();
    package(
        &mut s,
        "root/Cargo.toml",
        "root",
        "1.0.0",
        "a = { package = 'x', version = '<=1.1.0' }\nb = { package = 'x', version = '>1.1.0' }\nz = '=1.0.0'",
    );
    for version in ["0.9.0", "1.1.0", "1.2.0"] {
        vendor(&mut s, "x", version, "");
    }
    vendor(&mut s, "x", "2.0.0", "z = '=1.1.0'");
    vendor(&mut s, "z", "1.0.0", "");
    vendor(&mut s, "z", "1.1.0", "");
    let selected = selection(&s, 32).unwrap().unwrap();
    assert_eq!(selected.edges[&("root/Cargo.toml".into(), "a".into())], "vendor/x-0.9.0/Cargo.toml");
    assert_eq!(selected.edges[&("root/Cargo.toml".into(), "b".into())], "vendor/x-1.2.0/Cargo.toml");
    assert!(!selected.visited.contains("vendor/x-2.0.0/Cargo.toml"));
}

#[test]
fn contradictory_pins_fail_instead_of_dropping_a_required_edge() {
    let mut s = sources();
    package(
        &mut s,
        "root/Cargo.toml",
        "root",
        "1.0.0",
        "a = { package = 'x', version = '=1.0.0' }\nb = { package = 'x', version = '=1.1.0' }",
    );
    vendor(&mut s, "x", "1.0.0", "");
    vendor(&mut s, "x", "1.1.0", "");
    assert!(selection(&s, 16).unwrap().is_none());
    let Err(error) = resolve(&s, &[("root/Cargo.toml", &[])], false) else {
        panic!("contradictory pins were admitted")
    };
    assert!(error.to_string().contains("version unification has no solution"));
}

#[test]
fn incompatible_families_coexist_and_satisfying_versions_are_reused() {
    let mut s = sources();
    package(
        &mut s,
        "root/Cargo.toml",
        "root",
        "1.0.0",
        "a = { package = 'x', version = '1' }\nb = { package = 'x', version = '2' }\nc = { package = 'x', version = '>=1' }",
    );
    vendor(&mut s, "x", "1.0.0", "");
    vendor(&mut s, "x", "1.1.0", "");
    vendor(&mut s, "x", "2.0.0", "");
    let selected = selection(&s, 16).unwrap().unwrap();
    assert_eq!(selected.edges[&("root/Cargo.toml".into(), "a".into())], "vendor/x-1.1.0/Cargo.toml");
    assert_eq!(selected.edges[&("root/Cargo.toml".into(), "b".into())], "vendor/x-2.0.0/Cargo.toml");
    assert_eq!(selected.edges[&("root/Cargo.toml".into(), "c".into())], "vendor/x-1.1.0/Cargo.toml");
}

#[test]
fn missing_vendor_is_allowed_only_until_a_dependency_is_active() {
    let mut s = sources();
    package(&mut s, "root/Cargo.toml", "root", "1.0.0", "missing = { version = '1', optional = true }");
    let topology = resolve(&s, &[("root/Cargo.toml", &[])], false).unwrap();
    assert!(topology.packages.values().all(|p| p.dependencies.is_empty()));
    let Err(error) = resolve(&s, &[("root/Cargo.toml", &["missing"])], false) else {
        panic!("active missing dependency was admitted")
    };
    assert!(error.to_string().contains("root -> missing has no original vendored version"));
}

#[test]
fn work_bound_is_a_failure_not_a_partial_solution() {
    let mut s = sources();
    package(&mut s, "root/Cargo.toml", "root", "1.0.0", "a = '1'\nb = '1'");
    vendor(&mut s, "a", "1.0.0", "");
    vendor(&mut s, "b", "1.0.0", "");
    let Err(error) = selection(&s, 1) else {
        panic!("work limit was ignored")
    };
    assert!(error.to_string().contains("finite work bound"));
    assert_eq!(selection(&s, 2).unwrap().unwrap().edges.len(), 2);
}

#[test]
fn backjumping_agrees_with_exhaustive_version_assignments() {
    // Every acyclic three-package graph with two compatible versions and each
    // dependency either broad, pinned low, or pinned high: 3^6 constraint sets.
    for encoding in 0u32..729 {
        let mut digits = encoding;
        let mut requirements = [0u32; 6];
        for requirement in &mut requirements {
            *requirement = digits % 3;
            digits /= 3;
        }
        let requirement = |index: usize| ["1", "=1.0.0", "=1.1.0"][requirements[index] as usize];
        let mut s = sources();
        package(&mut s, "root/Cargo.toml", "root", "1.0.0", "a = '1'\nb = '1'\nc = '1'");
        for (index, version) in ["1.0.0", "1.1.0"].into_iter().enumerate() {
            vendor(
                &mut s,
                "a",
                version,
                &format!("b = {:?}\nc = {:?}", requirement(index * 2), requirement(index * 2 + 1)),
            );
            vendor(&mut s, "b", version, &format!("c = {:?}", requirement(4 + index)));
            vendor(&mut s, "c", version, "");
        }
        let permits = |constraint: u32, version: u32| constraint == 0 || constraint == version + 1;
        let expected = (0u32..8).any(|assignment| {
            let a = (assignment & 1) as usize;
            let b = (assignment >> 1) & 1;
            let c = (assignment >> 2) & 1;
            permits(requirements[a * 2], b)
                && permits(requirements[a * 2 + 1], c)
                && permits(requirements[4 + b as usize], c)
        });
        let selected = selection(&s, 128).unwrap();
        assert_eq!(selected.is_some(), expected, "constraint set {encoding}");
        if let Some(selected) = selected {
            let path = |name: &str| &selected.edges[&("root/Cargo.toml".into(), name.into())];
            let version = |name: &str| u32::from(path(name).contains("1.1.0"));
            let a = version("a") as usize;
            assert!(permits(requirements[a * 2], version("b")), "a -> b: {encoding}");
            assert!(permits(requirements[a * 2 + 1], version("c")), "a -> c: {encoding}");
            assert!(permits(requirements[4 + version("b") as usize], version("c")), "b -> c: {encoding}");
        }
    }
}
