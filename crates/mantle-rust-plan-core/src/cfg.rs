//! Bounded Cargo target-cfg admission for selected native dependencies.

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;

use crate::features::classify_target_cfg_dependency;
use crate::features::native_optional_dependency_selected;
use crate::model::MAX_NATIVE_SCALAR_BYTES;
use crate::model::classify_native_target_triple;

/// Preserve the accepted native target-cfg interpreter's bounded evaluation.
pub const MAX_CFG_EVALUATION_STEPS: usize = 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NativeDependencyEligibility {
    pub is_cfg_selected: bool,
    pub is_optional_selected: bool,
    pub is_dependency_selected: bool,
    pub decision: &'static str,
}

/// Classify each dependency from the cfg result already selected once for its
/// target table; avoids reinterpreting a predicate for every declared edge.
pub fn classify_native_dependency_from_selected_cfg(
    is_cfg_selected: bool,
    dependency_name: &str,
    is_optional: bool,
    feature_defs: &BTreeMap<String, Vec<String>>,
    selected_features: &[String],
) -> NativeDependencyEligibility {
    let is_optional_selected = native_optional_dependency_selected(dependency_name, feature_defs, selected_features);
    let is_dependency_selected = is_cfg_selected && (!is_optional || is_optional_selected);
    NativeDependencyEligibility {
        is_cfg_selected,
        is_optional_selected,
        is_dependency_selected,
        decision: classify_target_cfg_dependency(is_dependency_selected, is_cfg_selected),
    }
}

/// Literal target triples, and the accepted `cfg(...)` grammar (`not`, `any`,
/// `all`, and bounded target atoms) are evaluated without host access.
pub fn evaluate_supported_target_cfg(expression: &str, active_target: &str) -> Option<bool> {
    if expression.len() > MAX_NATIVE_SCALAR_BYTES || active_target.len() > MAX_NATIVE_SCALAR_BYTES {
        return None;
    }
    let trimmed = expression.trim();
    if !trimmed.starts_with("cfg(") {
        return Some(trimmed == active_target);
    }
    let inner = trimmed.strip_prefix("cfg(")?.strip_suffix(')')?.trim();
    evaluate_cfg_inner(inner, active_target)
}

enum Frame<'a> {
    Evaluate(&'a str),
    Not,
    Any { arguments: Vec<&'a str>, next_index: usize },
    All { arguments: Vec<&'a str>, next_index: usize },
}

fn evaluate_cfg_inner(expression: &str, active_target: &str) -> Option<bool> {
    let mut frames = Vec::with_capacity(8);
    let mut values = Vec::with_capacity(8);
    frames.push(Frame::Evaluate(expression));
    for _ in 0..MAX_CFG_EVALUATION_STEPS {
        let Some(frame) = frames.pop() else {
            return (values.len() == 1).then(|| values.pop()).flatten();
        };
        match frame {
            Frame::Evaluate(candidate) => {
                let trimmed = candidate.trim();
                if let Some(inner) = call_arg(trimmed, "not") {
                    frames.push(Frame::Not);
                    frames.push(Frame::Evaluate(inner));
                } else if let Some(inner) = call_arg(trimmed, "any") {
                    queue_arguments(split_args(inner)?, false, &mut frames, &mut values)?;
                } else if let Some(inner) = call_arg(trimmed, "all") {
                    queue_arguments(split_args(inner)?, true, &mut frames, &mut values)?;
                } else {
                    values.push(evaluate_atom(trimmed, active_target)?);
                }
            }
            Frame::Not => {
                let value = values.pop()?;
                values.push(!value);
            }
            Frame::Any { arguments, next_index } => {
                if values.pop()? {
                    values.push(true);
                } else {
                    continue_arguments(arguments, next_index, false, &mut frames, &mut values)?;
                }
            }
            Frame::All { arguments, next_index } => {
                if !values.pop()? {
                    values.push(false);
                } else {
                    continue_arguments(arguments, next_index, true, &mut frames, &mut values)?;
                }
            }
        }
    }
    None
}

fn call_arg<'a>(expression: &'a str, name: &str) -> Option<&'a str> {
    expression.strip_prefix(name)?.strip_prefix('(')?.strip_suffix(')')
}

fn queue_arguments<'a>(
    arguments: Vec<&'a str>,
    identity: bool,
    frames: &mut Vec<Frame<'a>>,
    values: &mut Vec<bool>,
) -> Option<()> {
    let Some(first) = arguments.first().copied() else {
        values.push(identity);
        return Some(());
    };
    frames.push(if identity {
        Frame::All {
            arguments,
            next_index: 1,
        }
    } else {
        Frame::Any {
            arguments,
            next_index: 1,
        }
    });
    frames.push(Frame::Evaluate(first));
    Some(())
}

fn continue_arguments<'a>(
    arguments: Vec<&'a str>,
    next_index: usize,
    identity: bool,
    frames: &mut Vec<Frame<'a>>,
    values: &mut Vec<bool>,
) -> Option<()> {
    let Some(argument) = arguments.get(next_index).copied() else {
        values.push(identity);
        return Some(());
    };
    let next_index = next_index.checked_add(1)?;
    frames.push(if identity {
        Frame::All { arguments, next_index }
    } else {
        Frame::Any { arguments, next_index }
    });
    frames.push(Frame::Evaluate(argument));
    Some(())
}

fn split_args(args: &str) -> Option<Vec<&str>> {
    let mut parts = Vec::with_capacity(args.matches(',').count().saturating_add(1));
    let mut depth = 0usize;
    let mut in_string = false;
    let mut start = 0usize;
    for (index, ch) in args.char_indices() {
        match ch {
            '"' => in_string = !in_string,
            '(' if !in_string => depth = depth.checked_add(1)?,
            ')' if !in_string => depth = depth.checked_sub(1)?,
            ',' if !in_string && depth == 0 => {
                let part = args[start..index].trim();
                if !part.is_empty() {
                    parts.push(part);
                }
                start = index.saturating_add(ch.len_utf8());
            }
            _ => {}
        }
    }
    if in_string || depth != 0 {
        return None;
    }
    let tail = args[start..].trim();
    if !tail.is_empty() {
        parts.push(tail);
    }
    Some(parts)
}

fn evaluate_atom(atom: &str, active_target: &str) -> Option<bool> {
    let target = classify_native_target_triple(active_target);
    if atom == "unix" {
        return Some(target.is_unix);
    }
    if atom == "windows" {
        return Some(target.os == "windows");
    }
    if let Some((key, raw_value)) = atom.split_once('=') {
        let value = raw_value.trim().trim_matches('"');
        let matched = match key.trim() {
            "target_os" => value == target.os,
            "target_arch" => value == target.arch,
            "target_family" => value == target.family,
            "target_vendor" => value == target.vendor,
            "target_env" => value == target.env,
            "target_abi" => value == target.abi,
            "target_endian" => value == target.endian,
            "target_pointer_width" => value == target.pointer_width,
            "target_has_atomic" => {
                let pointer_width = target.pointer_width.parse::<u32>().unwrap_or(0);
                if value == "ptr" {
                    pointer_width > 0
                } else {
                    value.parse::<u32>().is_ok_and(|width| width <= pointer_width)
                }
            }
            "feature" => false,
            key if key.ends_with("_backend") || key.starts_with("rustix_") || key == "getrandom_backend" => false,
            _ => return None,
        };
        return Some(matched);
    }
    matches!(
        atom,
        "loom"
            | "miri"
            | "criterion"
            | "compiletests"
            | "crossbeam_loom"
            | "diatomic_waker_loom"
            | "tokio_unstable"
            | "tracing_unstable"
            | "valgrind"
            | "windows_raw_dylib"
            | "rustix_use_libc"
            | "rustix_use_experimental_asm"
    )
    .then_some(false)
}
