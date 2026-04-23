use std::ffi::OsString;

use crunch_eval::Expr;

pub fn evaluate_against_system_module_contract(
    module_source: &str,
    import_paths: &[OsString],
) -> Result<Expr, crunch_eval::Error> {
    let wrapped_source = wrap_with_contract("SystemModule", module_source);
    crunch_eval::evaluate_str(&wrapped_source, import_paths)
}

pub fn evaluate_against_inventory_contract(
    inventory_source: &str,
    import_paths: &[OsString],
) -> Result<Expr, crunch_eval::Error> {
    let wrapped_source = wrap_with_contract("Inventory", inventory_source);
    crunch_eval::evaluate_str(&wrapped_source, import_paths)
}

fn wrap_with_contract(contract_name: &str, body_source: &str) -> String {
    assert!(!contract_name.is_empty(), "contract name must not be empty");
    assert!(!body_source.is_empty(), "body source must not be empty");
    format!(
        "let crunch = import \"lib.ncl\" in\n({body_source}) | crunch.{contract_name}\n"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stdlib_import_paths() -> Vec<OsString> {
        vec![crunch_eval::stdlib::stdlib_import_path().unwrap().into_os_string()]
    }

    #[test]
    fn valid_system_module_contract_passes() {
        let source = r#"
          {
            interface = {
              roles = {
                server = { port = 22 },
              },
            },
            impl = fun args => {
              output = {
                exports = { ssh = args.settings.port },
                providers = { firewall = { port = args.settings.port } },
                nixos = { services = { sshd = true } },
              },
            },
          }
        "#;

        let result = evaluate_against_system_module_contract(source, &stdlib_import_paths());
        let error_text = result.as_ref().err().map(ToString::to_string).unwrap_or_default();

        assert!(result.is_ok(), "valid module should pass contract: {error_text}");
    }

    #[test]
    fn missing_interface_contract_fails() {
        let source = r#"
          {
            impl = fun _args => { output = {} },
          }
        "#;

        let error_text = evaluate_against_system_module_contract(source, &stdlib_import_paths())
            .err()
            .expect("invalid module should fail")
            .to_string();

        assert!(error_text.contains("interface"), "error should mention missing interface: {error_text}");
        assert!(
            error_text.contains("missing definition") || error_text.contains("blame") || error_text.contains("contract"),
            "error should be contract-related: {error_text}"
        );
    }
}
