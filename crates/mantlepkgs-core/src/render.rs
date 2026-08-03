use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;

use serde_json::Value;

use crate::CATALOG_SCHEMA;
use crate::CoreFailure;
use crate::Diagnostic;
use crate::MantlepkgsCatalog;

const INDENT_SPACES: usize = 2;
const MAX_RENDER_DEPTH: usize = 64;
const CATALOG_CONTRACT: &str = r#"let Catalog = {
  schema | String,
  catalog_identity_blake3 | String,
  plan_identity_blake3 | String,
  manifest_digest_blake3 | String,
  source_lock | Dyn,
  systems | Array String,
  target_store_prefix | String,
  packages | Array Dyn,
  artifacts | Array Dyn,
  producer_receipt_digest_blake3 | String,
  shared_node_count | Number,
  source_requirement_count | Number,
  max_artifact_bytes | Number,
  non_claims | Array String,
} in
"#;

pub fn render_catalog_nickel(catalog: &MantlepkgsCatalog) -> Result<String, CoreFailure> {
    if catalog.schema != CATALOG_SCHEMA {
        return Err(CoreFailure::from_diagnostic(Diagnostic::new(
            "unsupported-catalog-schema",
            "catalog.schema",
            "the catalog schema is not supported for Nickel rendering",
        )));
    }
    let value = serde_json::to_value(catalog).map_err(|_| {
        CoreFailure::from_diagnostic(Diagnostic::new(
            "catalog-render-serialization-failed",
            "catalog",
            "the catalog did not serialize for Nickel rendering",
        ))
    })?;
    let mut output = String::from("# Generated Mantlepkgs catalog. Do not edit.\n");
    output.push_str(CATALOG_CONTRACT);
    render_value(&value, 0, &mut output)?;
    output.push_str(" | Catalog\n");
    debug_assert!(output.starts_with("# Generated Mantlepkgs catalog."));
    debug_assert!(output.ends_with(" | Catalog\n"));
    Ok(output)
}

fn render_value(value: &Value, depth: usize, output: &mut String) -> Result<(), CoreFailure> {
    if depth > MAX_RENDER_DEPTH {
        return Err(CoreFailure::from_diagnostic(Diagnostic::new(
            "catalog-render-depth-exceeded",
            "catalog",
            "the catalog exceeds the Nickel render depth limit",
        )));
    }
    match value {
        Value::Null => output.push_str("null"),
        Value::Bool(value) => output.push_str(if *value { "true" } else { "false" }),
        Value::Number(value) => output.push_str(&value.to_string()),
        Value::String(value) => output.push_str(&quote(value)?),
        Value::Array(values) => render_array(values, depth, output)?,
        Value::Object(fields) => render_record(fields, depth, output)?,
    }
    Ok(())
}

fn render_array(values: &[Value], depth: usize, output: &mut String) -> Result<(), CoreFailure> {
    if values.is_empty() {
        output.push_str("[]");
        return Ok(());
    }
    output.push_str("[\n");
    let child_depth = next_depth(depth)?;
    for value in values {
        indent(child_depth, output)?;
        render_value(value, child_depth, output)?;
        output.push_str(",\n");
    }
    indent(depth, output)?;
    output.push(']');
    Ok(())
}

fn render_record(
    fields: &serde_json::Map<String, Value>,
    depth: usize,
    output: &mut String,
) -> Result<(), CoreFailure> {
    if fields.is_empty() {
        output.push_str("{}");
        return Ok(());
    }
    output.push_str("{\n");
    let mut ordered = fields.iter().collect::<Vec<_>>();
    ordered.sort_by(|left, right| left.0.cmp(right.0));
    debug_assert_eq!(ordered.len(), fields.len());
    debug_assert!(ordered.windows(2).all(|pair| pair[0].0 <= pair[1].0));
    let child_depth = next_depth(depth)?;
    for (key, value) in ordered {
        indent(child_depth, output)?;
        output.push_str(&quote(key)?);
        output.push_str(" = ");
        render_value(value, child_depth, output)?;
        output.push_str(",\n");
    }
    indent(depth, output)?;
    output.push('}');
    Ok(())
}

fn quote(value: &str) -> Result<String, CoreFailure> {
    serde_json::to_string(value).map_err(|_| {
        CoreFailure::from_diagnostic(Diagnostic::new(
            "catalog-render-string-failed",
            "catalog",
            "a catalog string did not serialize",
        ))
    })
}

fn next_depth(depth: usize) -> Result<usize, CoreFailure> {
    depth.checked_add(1).ok_or_else(|| {
        CoreFailure::from_diagnostic(Diagnostic::new(
            "catalog-render-depth-overflow",
            "catalog",
            "the Nickel render depth exceeded the integer range",
        ))
    })
}

fn indent(depth: usize, output: &mut String) -> Result<(), CoreFailure> {
    let width = depth.checked_mul(INDENT_SPACES).ok_or_else(|| {
        CoreFailure::from_diagnostic(Diagnostic::new(
            "catalog-render-indent-overflow",
            "catalog",
            "the Nickel indentation width exceeded the integer range",
        ))
    })?;
    output.push_str(&" ".repeat(width));
    Ok(())
}

#[cfg(test)]
mod tests {
    use alloc::format;

    use super::*;
    use crate::ArtifactBinding;
    use crate::NixpkgsSourceLock;

    const DIGEST: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    const REVISION: &str = "0123456789abcdef0123456789abcdef01234567";
    const ARTIFACT_BYTES: u64 = 16;

    fn catalog() -> MantlepkgsCatalog {
        MantlepkgsCatalog {
            schema: CATALOG_SCHEMA.into(),
            catalog_identity_blake3: DIGEST.into(),
            plan_identity_blake3: DIGEST.into(),
            manifest_digest_blake3: DIGEST.into(),
            source_lock: NixpkgsSourceLock {
                reference: format!("github:NixOS/nixpkgs/{REVISION}"),
                revision: REVISION.into(),
                lock_digest_blake3: DIGEST.into(),
            },
            systems: alloc::vec!["x86_64-linux".into()],
            target_store_prefix: "/mantle/store".into(),
            packages: Vec::new(),
            artifacts: alloc::vec![ArtifactBinding {
                role: "shared-foreign-graph".into(),
                path: "artifacts/shared.graph.json".into(),
                digest_blake3: DIGEST.into(),
                bytes: ARTIFACT_BYTES,
            }],
            producer_receipt_digest_blake3: DIGEST.into(),
            shared_node_count: 0,
            source_requirement_count: 0,
            max_artifact_bytes: ARTIFACT_BYTES,
            non_claims: alloc::vec!["not-package-correctness".into()],
        }
    }

    #[test]
    fn catalog_renders_as_typed_deterministic_nickel() {
        let first = render_catalog_nickel(&catalog()).expect("catalog renders");
        let second = render_catalog_nickel(&catalog()).expect("catalog renders again");
        assert_eq!(first, second);
        assert!(first.contains("let Catalog ="));
        assert!(first.contains("\"catalog_identity_blake3\" ="));
        assert!(first.ends_with(" | Catalog\n"));
    }

    #[test]
    fn unsupported_catalog_schema_does_not_render() {
        let mut invalid = catalog();
        invalid.schema = "future".into();
        let failure = render_catalog_nickel(&invalid).expect_err("unsupported schema fails");
        assert_eq!(failure.diagnostics[0].code, "unsupported-catalog-schema");
        assert!(!failure.diagnostics[0].message.is_empty());
    }

    #[test]
    fn render_depth_overflow_is_an_error() {
        let failure = next_depth(usize::MAX).expect_err("depth overflow must fail");
        assert_eq!(failure.diagnostics[0].code, "catalog-render-depth-overflow");
        assert!(!failure.diagnostics[0].message.is_empty());
    }
}
