//! Rendering for structured-refactor plans.

use mantle_application_contract::OutputFormat;

use crate::RunError;
use crate::structured_refactor;

/// Render one refactor plan.
pub(crate) fn render_refactor_plan(
    plan: &structured_refactor::RefactorPlan,
    format: OutputFormat,
) -> Result<String, RunError> {
    debug_assert!(!plan.session_id.is_empty());
    debug_assert!(!plan.root.is_empty());
    if format.is_machine_readable() {
        let rendered = serde_json::to_string_pretty(plan).map_err(|err| RunError::Internal(err.to_string()))?;
        debug_assert!(rendered.starts_with('{'));
        return Ok(rendered);
    }
    let mut lines = vec![
        format!("refactor session: {}", plan.session_id),
        format!("root: {}", plan.root),
        format!("dry-run: {}", plan.dry_run),
    ];
    if plan.operations.is_empty() {
        lines.push(String::from("operations: none"));
    } else {
        lines.push(String::from("operations:"));
        for operation in &plan.operations {
            let suffix = if operation.apply_supported { "" } else { " (plan-only)" };
            lines.push(format!("  {}: {} -> {}{}", operation.kind, operation.from, operation.to, suffix));
        }
    }
    for diagnostic in &plan.diagnostics {
        lines.push(format!("diagnostic {}: {}", diagnostic.code, diagnostic.message));
        lines.push(format!("  remediation: {}", diagnostic.remediation));
    }
    debug_assert!(lines.len() >= 4);
    debug_assert!(lines.iter().all(|line| !line.is_empty()));
    Ok(lines.join("\n"))
}

/// Write one refactor plan to standard output.
pub(crate) fn print_refactor_plan(
    plan: &structured_refactor::RefactorPlan,
    format: OutputFormat,
) -> Result<(), RunError> {
    let rendered = render_refactor_plan(plan, format)?;
    println!("{rendered}");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn plan(
        operations: Vec<structured_refactor::RefactorOperation>,
        diagnostics: Vec<structured_refactor::RefactorDiagnostic>,
    ) -> structured_refactor::RefactorPlan {
        structured_refactor::RefactorPlan {
            schema: structured_refactor::REFACTOR_SESSION_SCHEMA,
            session_id: "session-1",
            root: "src".to_string(),
            dry_run: true,
            operations,
            diagnostics,
        }
    }

    fn operation(apply_supported: bool) -> structured_refactor::RefactorOperation {
        structured_refactor::RefactorOperation {
            kind: "rename",
            from: "a.ncl".to_string(),
            to: "b.ncl".to_string(),
            apply_supported,
        }
    }

    #[test]
    fn the_json_branch_emits_the_plan_payload() {
        let rendered =
            render_refactor_plan(&plan(vec![operation(true)], Vec::new()), OutputFormat::Json).expect("json renders");
        let parsed: serde_json::Value = serde_json::from_str(&rendered).expect("output is JSON");
        assert_eq!(parsed["session_id"], "session-1");
        assert_eq!(parsed["operations"][0]["kind"], "rename");
    }

    #[test]
    fn the_human_branch_lists_operations_and_marks_plan_only_entries() {
        let rendered =
            render_refactor_plan(&plan(vec![operation(true), operation(false)], Vec::new()), OutputFormat::Human)
                .expect("human renders");
        assert!(rendered.contains("refactor session: session-1"));
        assert!(rendered.contains("root: src"));
        assert!(rendered.contains("dry-run: true"));
        assert!(rendered.contains("  rename: a.ncl -> b.ncl\n"));
        assert!(rendered.contains("  rename: a.ncl -> b.ncl (plan-only)"));
    }

    #[test]
    fn an_empty_operation_list_is_reported_as_none() {
        let rendered = render_refactor_plan(&plan(Vec::new(), Vec::new()), OutputFormat::Human).expect("human renders");
        assert!(rendered.contains("operations: none"));
        assert!(!rendered.contains("operations:\n"));
    }

    #[test]
    fn diagnostics_render_their_remediation_line() {
        let diagnostic = structured_refactor::RefactorDiagnostic {
            code: "refactor-conflict",
            message: "target exists".to_string(),
            remediation: "pick another name".to_string(),
        };
        let rendered =
            render_refactor_plan(&plan(Vec::new(), vec![diagnostic]), OutputFormat::Human).expect("human renders");
        assert!(rendered.contains("diagnostic refactor-conflict: target exists"));
        assert!(rendered.contains("  remediation: pick another name"));
    }
}
