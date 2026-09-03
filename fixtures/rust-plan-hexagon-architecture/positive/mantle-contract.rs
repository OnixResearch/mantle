struct PackageFact {
    package_id: String,
    features: Vec<String>,
}

fn selected(mut packages: Vec<PackageFact>) -> Vec<PackageFact> {
    packages.sort_by(|left, right| left.package_id.cmp(&right.package_id));
    packages
}
