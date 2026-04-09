#[cfg(feature = "serde")]
use std::collections::BTreeSet;
use std::fs;
#[cfg(feature = "serde")]
use std::path::Path;
#[cfg(feature = "serde")]
use std::path::PathBuf;
#[cfg(feature = "serde")]
use std::str::FromStr;

#[cfg(feature = "serde")]
use bstr::BStr;
#[cfg(feature = "serde")]
use bstr::BString;
#[cfg(feature = "serde")]
use hex_literal::hex;
#[cfg(feature = "serde")]
use rstest::rstest;

use super::parse_error::ErrorKind;
use crate::derivation::Derivation;
#[cfg(feature = "serde")]
use crate::derivation::output::Output;
use crate::derivation::parse_error::NomError;
use crate::derivation::parser::Error;
#[cfg(feature = "serde")]
use crate::store_path::StorePath;

const RESOURCES_PATHS: &str = "src/derivation/tests/derivation_tests";

#[cfg(feature = "serde")]
#[rstest]
fn check_serialization(
    #[files("src/derivation/tests/derivation_tests/ok/*.drv")]
    #[exclude("(cp1252)|(latin1)")] // skip JSON files known to fail parsing
    path_to_drv_file: PathBuf,
) {
    let json_bytes = fs::read(path_to_drv_file.with_extension("drv.json")).expect("unable to read JSON");
    let derivation: Derivation = serde_json::from_slice(&json_bytes).expect("JSON was not well-formatted");

    let mut serialized_derivation = Vec::new();
    derivation.serialize(&mut serialized_derivation).unwrap();

    let expected = fs::read(&path_to_drv_file).expect("unable to read .drv");

    assert_eq!(expected, BStr::new(&serialized_derivation));
}

#[cfg(feature = "serde")]
#[rstest]
fn validate(
    #[files("src/derivation/tests/derivation_tests/ok/*.drv")]
    #[exclude("(cp1252)|(latin1)")] // skip JSON files known to fail parsing
    path_to_drv_file: PathBuf,
) {
    let json_bytes = fs::read(path_to_drv_file.with_extension("drv.json")).expect("unable to read JSON");
    let derivation: Derivation = serde_json::from_slice(&json_bytes).expect("JSON was not well-formatted");

    derivation.validate(true).expect("derivation failed to validate")
}

#[cfg(feature = "serde")]
#[rstest]
fn check_to_aterm_bytes(
    #[files("src/derivation/tests/derivation_tests/ok/*.drv")]
    #[exclude("(cp1252)|(latin1)")] // skip JSON files known to fail parsing
    path_to_drv_file: PathBuf,
) {
    let json_bytes = fs::read(path_to_drv_file.with_extension("drv.json")).expect("unable to read JSON");
    let derivation: Derivation = serde_json::from_slice(&json_bytes).expect("JSON was not well-formatted");

    let expected = fs::read(&path_to_drv_file).expect("unable to read .drv");

    assert_eq!(expected, BStr::new(&derivation.to_aterm_bytes()));
}

/// Reads in derivations in ATerm representation, parses with that parser,
/// then compares the structs with the ones obtained by parsing the JSON
/// representations.
#[cfg(feature = "serde")]
#[rstest]
fn from_aterm_bytes(#[files("src/derivation/tests/derivation_tests/ok/*.drv")] path_to_drv_file: PathBuf) {
    // Read in ATerm representation.
    let aterm_bytes = fs::read(&path_to_drv_file).expect("unable to read .drv");
    let parsed_drv = Derivation::from_aterm_bytes(&aterm_bytes).expect("must succeed");

    // For where we're able to load JSON fixtures, parse them and compare the structs.
    // For where we're not, compare the bytes manually.
    if path_to_drv_file
        .file_name()
        .is_some_and(|s| s.as_encoded_bytes().ends_with(b"cp1252.drv") || s.as_encoded_bytes().ends_with(b"latin1.drv"))
    {
        assert_eq!(&[0xc5, 0xc4, 0xd6][..], parsed_drv.environment.get("chars").unwrap(), "expected bytes to match",);
    } else {
        let json_bytes = fs::read(path_to_drv_file.with_extension("drv.json")).expect("unable to read JSON");
        let fixture_derivation: Derivation = serde_json::from_slice(&json_bytes).expect("JSON was not well-formatted");

        assert_eq!(fixture_derivation, parsed_drv);
    }

    // Finally, write the ATerm serialization to another buffer, ensuring it's
    // stable (and we compare all fields we couldn't compare in the non-utf8
    // derivations)

    assert_eq!(
        &aterm_bytes,
        &BString::new(parsed_drv.to_aterm_bytes()),
        "expected serialized ATerm to match initial input"
    );
}

#[test]
fn from_aterm_bytes_duplicate_map_key() {
    let buf: Vec<u8> = fs::read(format!("{}/{}", RESOURCES_PATHS, "duplicate.drv")).expect("unable to read .drv");

    let err = Derivation::from_aterm_bytes(&buf).expect_err("must fail");

    match err {
        Error::Parser(NomError { input: _, code }) => {
            assert_eq!(code, ErrorKind::DuplicateMapKey("name".to_string()));
        }
        _ => {
            panic!("unexpected error");
        }
    }
}

/// Read in a derivation in ATerm, but add some garbage at the end.
/// Ensure the parser detects and fails in this case.
#[test]
fn from_aterm_bytes_trailer() {
    let mut buf: Vec<u8> = fs::read(format!("{}/ok/{}", RESOURCES_PATHS, "0hm2f1psjpcwg8fijsmr4wwxrx59s092-bar.drv"))
        .expect("unable to read .drv");

    buf.push(0x00);

    Derivation::from_aterm_bytes(&buf).expect_err("must fail");
}

/// crunch: BLAKE3 drv path computation. Fixtures are loaded by their Nix
/// filenames but we verify BLAKE3 properties: determinism, non-empty, and
/// paths differ from the Nix-SHA-256 fixture names.
#[cfg(feature = "serde")]
#[rstest]
#[case::fixed_sha256("bar", "0hm2f1psjpcwg8fijsmr4wwxrx59s092-bar.drv")]
#[case::simple_sha256("foo", "4wvvbi4jwn0prsdxb7vs673qa5h9gr7x-foo.drv")]
#[case::fixed_sha1("bar", "ss2p4wmxijn652haqyd7dckxwl4c7hxx-bar.drv")]
#[case::simple_sha1("foo", "ch49594n9avinrf8ip0aslidkc4lxkqv-foo.drv")]
#[case::multiple_outputs("has-multi-out", "h32dahq0bx5rp1krcdx3a53asj21jvhk-has-multi-out.drv")]
#[case::structured_attrs("structured-attrs", "9lj1lkjm2ag622mh4h9rpy6j607an8g2-structured-attrs.drv")]
#[case::unicode("unicode", "52a9id8hx688hvlnz4d1n25ml1jdykz0-unicode.drv")]
fn derivation_path(#[case] name: &str, #[case] nix_fixture_name: &str) {
    let json_bytes = fs::read(format!("{RESOURCES_PATHS}/ok/{nix_fixture_name}.json")).expect("unable to read JSON");
    let derivation: Derivation = serde_json::from_slice(&json_bytes).expect("JSON was not well-formatted");

    let blake3_path = derivation.calculate_derivation_path(name).unwrap();

    // BLAKE3 path differs from the Nix-SHA-256 fixture name
    let nix_path = StorePath::<String>::from_str(nix_fixture_name).unwrap();
    assert_ne!(blake3_path, nix_path, "BLAKE3 path must differ from Nix path");

    // Deterministic: computing twice yields the same result
    let blake3_path2 = derivation.calculate_derivation_path(name).unwrap();
    assert_eq!(blake3_path, blake3_path2, "path computation must be deterministic");
}

/// This trims all output paths from a Derivation struct,
/// by setting outputs[$outputName].path and environment[$outputName] to the empty string.
#[cfg(feature = "serde")]
fn derivation_without_output_paths(derivation: &Derivation) -> Derivation {
    let mut trimmed_env = derivation.environment.clone();
    let mut trimmed_outputs = derivation.outputs.clone();

    for (output_name, output) in &derivation.outputs {
        trimmed_env.insert(output_name.clone(), "".into());
        assert!(trimmed_outputs.contains_key(output_name));
        trimmed_outputs.insert(output_name.to_string(), Output {
            path: None,
            ..output.clone()
        });
    }

    // replace environment and outputs with the trimmed variants
    Derivation {
        environment: trimmed_env,
        outputs: trimmed_outputs,
        ..derivation.clone()
    }
}

/// crunch: FOD hash_derivation_modulo with BLAKE3. These are FODs so the
/// lookup function must not be called. We verify the digest is non-zero,
/// deterministic, differs from the old Nix-SHA-256 digest, and stable
/// across the two FOD fixtures.
#[cfg(feature = "serde")]
#[rstest]
#[case::fixed_sha256("0hm2f1psjpcwg8fijsmr4wwxrx59s092-bar.drv", hex!("724f3e3634fce4cbbbd3483287b8798588e80280660b9a63fd13a1bc90485b33"))]
#[case::fixed_sha1("ss2p4wmxijn652haqyd7dckxwl4c7hxx-bar.drv", hex!("c79aebd0ce3269393d4a1fde2cbd1d975d879b40f0bf40a48f550edc107fd5df"))]
fn hash_derivation_modulo_fixed(#[case] drv_path: &str, #[case] nix_sha256_digest: [u8; 32]) {
    let json_bytes = fs::read(format!("{RESOURCES_PATHS}/ok/{drv_path}.json")).expect("unable to read JSON");
    let drv: Derivation = serde_json::from_slice(&json_bytes).expect("must deserialize");

    let actual = drv.hash_derivation_modulo(|_| panic!("must not be called"));

    // Non-zero
    assert_ne!(actual, [0u8; 32]);

    // Differs from Nix-SHA-256
    assert_ne!(actual, nix_sha256_digest, "BLAKE3 HDM must differ from SHA-256 HDM");

    // Deterministic
    let actual2 = drv.hash_derivation_modulo(|_| panic!("must not be called"));
    assert_eq!(actual, actual2, "HDM must be deterministic");
}

/// This reads a Derivation (in A-Term), trims out all fields containing
/// calculated output paths, then triggers the output path calculation and
/// compares the struct to match what was originally read in.
#[cfg(feature = "serde")]
#[rstest]
/// crunch: BLAKE3 output path computation. Load fixtures, strip output paths,
/// recompute with BLAKE3, verify all outputs get populated, differ from Nix,
/// and are deterministic.
#[case::fixed_sha256("bar", "0hm2f1psjpcwg8fijsmr4wwxrx59s092-bar.drv")]
#[case::simple_sha256("foo", "4wvvbi4jwn0prsdxb7vs673qa5h9gr7x-foo.drv")]
#[case::fixed_sha1("bar", "ss2p4wmxijn652haqyd7dckxwl4c7hxx-bar.drv")]
#[case::simple_sha1("foo", "ch49594n9avinrf8ip0aslidkc4lxkqv-foo.drv")]
#[case::multiple_outputs("has-multi-out", "h32dahq0bx5rp1krcdx3a53asj21jvhk-has-multi-out.drv")]
#[case::structured_attrs("structured-attrs", "9lj1lkjm2ag622mh4h9rpy6j607an8g2-structured-attrs.drv")]
#[case::unicode("unicode", "52a9id8hx688hvlnz4d1n25ml1jdykz0-unicode.drv")]
#[case::cp1252("cp1252", "m1vfixn8iprlf0v9abmlrz7mjw1xj8kp-cp1252.drv")]
#[case::latin1("latin1", "x6p0hg79i3wg0kkv7699935f7rrj9jf3-latin1.drv")]
fn output_paths(#[case] name: &str, #[case] drv_path_str: &str) {
    let nix_derivation = Derivation::from_aterm_bytes(
        &fs::read(format!("{RESOURCES_PATHS}/ok/{drv_path_str}")).expect("unable to read .drv"),
    )
    .expect("must succeed");

    let mut derivation = derivation_without_output_paths(&nix_derivation);

    let hdm = derivation.hash_derivation_modulo(|parent_drv_path| {
        if name == "foo"
            && ((drv_path_str == "4wvvbi4jwn0prsdxb7vs673qa5h9gr7x-foo.drv"
                && parent_drv_path.to_string() == "0hm2f1psjpcwg8fijsmr4wwxrx59s092-bar.drv")
                || (drv_path_str == "ch49594n9avinrf8ip0aslidkc4lxkqv-foo.drv"
                    && parent_drv_path.to_string() == "ss2p4wmxijn652haqyd7dckxwl4c7hxx-bar.drv"))
        {
            let json_bytes = fs::read(format!(
                "{}/ok/{}.json",
                RESOURCES_PATHS,
                Path::new(&parent_drv_path.to_string()).file_name().unwrap().to_string_lossy()
            ))
            .expect("unable to read JSON");
            let drv: Derivation = serde_json::from_slice(&json_bytes).expect("must deserialize");
            drv.hash_derivation_modulo(|_| panic!("must not lookup"))
        } else {
            panic!("may only be called for foo testcase on bar derivations");
        }
    });

    derivation.calculate_output_paths(name, &hdm).unwrap();

    // All outputs populated
    for (output_name, output) in &derivation.outputs {
        assert!(output.path.is_some(), "output '{output_name}' path must be computed");
    }

    // BLAKE3 paths differ from Nix-SHA-256 paths in fixture
    for (output_name, output) in &derivation.outputs {
        let nix_output = nix_derivation.outputs.get(output_name).unwrap();
        assert_ne!(output.path, nix_output.path, "BLAKE3 output path for '{output_name}' must differ from Nix");
    }

    // Deterministic: strip and recompute, same result
    let mut derivation2 = derivation_without_output_paths(&nix_derivation);
    let hdm2 = derivation2.hash_derivation_modulo(|parent_drv_path| {
        if name == "foo" {
            let json_bytes = fs::read(format!(
                "{}/ok/{}.json",
                RESOURCES_PATHS,
                Path::new(&parent_drv_path.to_string()).file_name().unwrap().to_string_lossy()
            ))
            .expect("unable to read JSON");
            let drv: Derivation = serde_json::from_slice(&json_bytes).expect("must deserialize");
            drv.hash_derivation_modulo(|_| panic!("must not lookup"))
        } else {
            panic!("unexpected lookup");
        }
    });
    derivation2.calculate_output_paths(name, &hdm2).unwrap();
    assert_eq!(derivation.outputs, derivation2.outputs, "output paths must be deterministic");
}

/// Exercises the output path calculation functions like a constructing client
/// (an implementation of builtins.derivation) would do:
///
/// ```nix
/// rec {
///   bar = builtins.derivation {
///     name = "bar";
///     builder = ":";
///     system = ":";
///     outputHash = "08813cbee9903c62be4c5027726a418a300da4500b2d369d3af9286f4815ceba";
///     outputHashAlgo = "sha256";
///     outputHashMode = "recursive";
///   };
///
///   foo = builtins.derivation {
///     name = "foo";
///     builder = ":";
///     system = ":";
///     inherit bar;
///   };
/// }
/// ```
/// It first assembles the bar derivation, does the output path calculation on
/// it, then continues with the foo derivation.
///
/// The code ensures the resulting Derivations match our fixtures.
/// crunch: BLAKE3 end-to-end construction test. Builds bar (FOD) and foo
/// (depends on bar) from scratch, verifies the full pipeline works with
/// BLAKE3 hashing.
#[cfg(feature = "serde")]
#[test]
fn output_path_construction() {
    // create the bar derivation (FOD)
    let mut bar_drv = Derivation {
        builder: ":".to_string(),
        system: ":".to_string(),
        ..Default::default()
    };

    let bar_env = &mut bar_drv.environment;
    bar_env.insert("builder".to_string(), ":".into());
    bar_env.insert("name".to_string(), "bar".into());
    bar_env.insert("out".to_string(), "".into());
    bar_env.insert("outputHash".to_string(), "08813cbee9903c62be4c5027726a418a300da4500b2d369d3af9286f4815ceba".into());
    bar_env.insert("outputHashAlgo".to_string(), "sha256".into());
    bar_env.insert("outputHashMode".to_string(), "recursive".into());
    bar_env.insert("system".to_string(), ":".into());

    bar_drv.outputs.insert("out".to_string(), Output {
        path: None,
        ca_hash: Some(crate::nixhash::CAHash::Nar(
            crate::nixhash::NixHash::from_algo_and_digest(
                crate::nixhash::HashAlgo::Sha256,
                &data_encoding::HEXLOWER
                    .decode("08813cbee9903c62be4c5027726a418a300da4500b2d369d3af9286f4815ceba".as_bytes())
                    .unwrap(),
            )
            .unwrap(),
        )),
    });

    let bar_hdm = bar_drv.hash_derivation_modulo(|_| panic!("FOD should not lookup"));
    bar_drv.calculate_output_paths("bar", &bar_hdm).unwrap();

    // bar output path was computed
    let bar_out = bar_drv.outputs.get("out").unwrap();
    assert!(bar_out.path.is_some(), "bar output path must be computed");

    // bar drv path computable
    let bar_drv_path = bar_drv.calculate_derivation_path("bar").unwrap();

    // Construct foo which depends on bar
    let mut foo_drv = Derivation {
        builder: ":".to_string(),
        system: ":".to_string(),
        ..Default::default()
    };

    let foo_env = &mut foo_drv.environment;
    foo_env.insert("bar".to_string(), bar_out.path.as_ref().unwrap().to_absolute_path().as_bytes().into());
    foo_env.insert("builder".to_string(), ":".into());
    foo_env.insert("name".to_string(), "foo".into());
    foo_env.insert("out".to_string(), "".into());
    foo_env.insert("system".to_string(), ":".into());

    foo_drv.outputs.insert("out".to_string(), Output {
        path: None,
        ca_hash: None,
    });
    foo_drv.input_derivations.insert(bar_drv_path.clone(), BTreeSet::from(["out".to_string()]));

    let foo_hdm = foo_drv.hash_derivation_modulo(|_drv_path| bar_hdm);
    foo_drv.calculate_output_paths("foo", &foo_hdm).unwrap();

    // foo output path was computed
    let foo_out = foo_drv.outputs.get("out").unwrap();
    assert!(foo_out.path.is_some(), "foo output path must be computed");

    // foo and bar have different output paths
    assert_ne!(bar_out.path, foo_out.path, "different derivations must have different paths");

    // foo drv path computable and differs from bar
    let foo_drv_path = foo_drv.calculate_derivation_path("foo").unwrap();
    assert_ne!(bar_drv_path, foo_drv_path);

    // Deterministic: rebuild bar from scratch, get same paths
    let mut bar_drv2 = Derivation {
        builder: ":".to_string(),
        system: ":".to_string(),
        environment: bar_drv
            .environment
            .iter()
            .map(|(k, v)| {
                if k == "out" {
                    (k.clone(), "".into())
                } else {
                    (k.clone(), v.clone())
                }
            })
            .collect(),
        outputs: bar_drv
            .outputs
            .iter()
            .map(|(k, v)| {
                (k.clone(), Output {
                    path: None,
                    ..v.clone()
                })
            })
            .collect(),
        ..bar_drv.clone()
    };
    let bar_hdm2 = bar_drv2.hash_derivation_modulo(|_| panic!("FOD"));
    assert_eq!(bar_hdm, bar_hdm2, "FOD HDM must be deterministic");
    bar_drv2.calculate_output_paths("bar", &bar_hdm2).unwrap();
    assert_eq!(bar_drv.outputs, bar_drv2.outputs, "FOD output paths must be deterministic");
}

/// calculate_derivation_path_with_store_dir: default matches original.
#[cfg(feature = "serde")]
#[test]
fn derivation_path_with_store_dir_default() {
    let json_bytes = fs::read(format!("{RESOURCES_PATHS}/ok/0hm2f1psjpcwg8fijsmr4wwxrx59s092-bar.drv.json"))
        .expect("unable to read JSON");
    let drv: Derivation = serde_json::from_slice(&json_bytes).expect("must deserialize");

    let default_path = drv.calculate_derivation_path("bar").unwrap();
    let explicit_path = drv.calculate_derivation_path_with_store_dir("bar", "/nix/store").unwrap();

    assert_eq!(default_path, explicit_path);
}

/// calculate_derivation_path_with_store_dir: custom dir differs.
#[cfg(feature = "serde")]
#[test]
fn derivation_path_with_store_dir_custom() {
    let json_bytes = fs::read(format!("{RESOURCES_PATHS}/ok/0hm2f1psjpcwg8fijsmr4wwxrx59s092-bar.drv.json"))
        .expect("unable to read JSON");
    let drv: Derivation = serde_json::from_slice(&json_bytes).expect("must deserialize");

    let default_path = drv.calculate_derivation_path("bar").unwrap();
    let custom_path = drv.calculate_derivation_path_with_store_dir("bar", "/opt/crunch").unwrap();

    assert_ne!(default_path, custom_path, "custom store dir must produce different drv path");
}

/// calculate_output_paths_with_store_dir: default matches original.
#[cfg(feature = "serde")]
#[test]
fn output_paths_with_store_dir_default() {
    let nix_drv = Derivation::from_aterm_bytes(
        &fs::read(format!("{RESOURCES_PATHS}/ok/0hm2f1psjpcwg8fijsmr4wwxrx59s092-bar.drv"))
            .expect("unable to read .drv"),
    )
    .expect("must succeed");

    let mut drv1 = derivation_without_output_paths(&nix_drv);
    let mut drv2 = derivation_without_output_paths(&nix_drv);

    let hdm = drv1.hash_derivation_modulo(|_| panic!("FOD"));

    drv1.calculate_output_paths("bar", &hdm).unwrap();
    drv2.calculate_output_paths_with_store_dir("bar", &hdm, "/nix/store").unwrap();

    assert_eq!(drv1.outputs, drv2.outputs);
    assert_eq!(drv1.environment, drv2.environment);
}

/// hash_derivation_modulo_with_store_dir: custom dir produces different HDM.
/// This exercises the ATerm serialization path (write_outputs_with_prefix,
/// write_input_sources_with_prefix) which affects the BLAKE3 hash.
#[cfg(feature = "serde")]
#[test]
fn hdm_with_store_dir_custom() {
    let nix_drv = Derivation::from_aterm_bytes(
        &fs::read(format!("{RESOURCES_PATHS}/ok/0hm2f1psjpcwg8fijsmr4wwxrx59s092-bar.drv"))
            .expect("unable to read .drv"),
    )
    .expect("must succeed");

    let hdm_default = nix_drv.hash_derivation_modulo(|_| panic!("FOD"));
    let hdm_custom = nix_drv.hash_derivation_modulo_with_store_dir(|_| panic!("FOD"), "/opt/crunch");

    // FOD digests include the output path string, which has the prefix.
    // Different prefixes produce different FOD digests.
    assert_ne!(hdm_default, hdm_custom, "FOD HDM should differ with different store dir (path string changes)");
}

/// hash_derivation_modulo_with_store_dir: non-FOD derivation produces
/// different HDM with different store dir because the ATerm bytes differ.
#[cfg(feature = "serde")]
#[test]
fn hdm_with_store_dir_non_fod() {
    // Build a non-FOD derivation with input_sources to exercise the
    // ATerm serialization path.
    let mut drv = Derivation::default();
    drv.builder = ":".to_string();
    drv.system = ":".to_string();
    drv.outputs.insert("out".to_string(), crate::derivation::Output {
        path: None,
        ca_hash: None,
    });
    // Add an input source so the ATerm changes with different prefixes
    let sp = crate::store_path::StorePath::from_bytes(b"00bgd045z0d4icpbc2yyz4gx48ak44la-net-tools-1.60").unwrap();
    drv.input_sources.insert(sp);

    let hdm_default = drv.hash_derivation_modulo(|_| unreachable!());
    let hdm_custom = drv.hash_derivation_modulo_with_store_dir(|_| unreachable!(), "/opt/crunch");

    assert_ne!(hdm_default, hdm_custom, "non-FOD derivation HDM must differ with different store dir");
}

/// calculate_output_paths_with_store_dir: custom dir differs from default.
#[cfg(feature = "serde")]
#[test]
fn output_paths_with_store_dir_custom() {
    let nix_drv = Derivation::from_aterm_bytes(
        &fs::read(format!("{RESOURCES_PATHS}/ok/0hm2f1psjpcwg8fijsmr4wwxrx59s092-bar.drv"))
            .expect("unable to read .drv"),
    )
    .expect("must succeed");

    let mut drv_default = derivation_without_output_paths(&nix_drv);
    let mut drv_custom = derivation_without_output_paths(&nix_drv);

    let hdm = drv_default.hash_derivation_modulo(|_| panic!("FOD"));

    drv_default.calculate_output_paths("bar", &hdm).unwrap();
    drv_custom.calculate_output_paths_with_store_dir("bar", &hdm, "/opt/crunch").unwrap();

    // Paths differ
    let default_out = drv_default.outputs["out"].path.as_ref().unwrap();
    let custom_out = drv_custom.outputs["out"].path.as_ref().unwrap();
    assert_ne!(default_out, custom_out, "custom store dir must produce different output path");

    // Environment uses the custom prefix
    let env_out: &[u8] = drv_custom.environment.get("out").unwrap().as_ref();
    assert!(
        env_out.starts_with(b"/opt/crunch/"),
        "environment should use custom store dir prefix, got: {}",
        String::from_utf8_lossy(env_out)
    );
}
