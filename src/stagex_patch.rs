use std::fs;
use std::path::Component;
use std::path::Path;

const PATCH_BYTES_MAX: usize = 512 * 1_024;
const PATCH_FILE_COUNT_MAX: usize = 64;
const PATCH_HUNK_COUNT_MAX: usize = 1_024;
const PATCH_HUNK_LINE_COUNT_MAX: usize = 100_000;
const PATCH_PATH_BYTES_MAX: usize = 256;
const PATCH_LINE_BYTES_MAX: usize = 16 * 1_024;
const HUNK_HEADER_PREFIX: &str = "@@ ";
const HUNK_HEADER_SUFFIX: &str = " @@";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct UnifiedPatch {
    files: Vec<FilePatch>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct FilePatch {
    relative_path: String,
    hunks: Vec<Hunk>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Hunk {
    old_start: u32,
    old_count: u32,
    new_start: u32,
    new_count: u32,
    lines: Vec<HunkLine>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum HunkLine {
    Context(String),
    Remove(String),
    Add(String),
}

#[derive(Debug)]
pub(crate) enum UnifiedPatchError {
    Invalid(String),
    Io { action: String, source: std::io::Error },
}

impl std::fmt::Display for UnifiedPatchError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Invalid(message) => write!(formatter, "invalid unified patch: {message}"),
            Self::Io { action, source } => write!(formatter, "{action}: {source}"),
        }
    }
}

impl std::error::Error for UnifiedPatchError {}

pub(crate) fn apply_unified_patch_tree(root: &Path, patch_bytes: &[u8]) -> Result<Vec<String>, UnifiedPatchError> {
    validate_patch_shell_inputs(root, patch_bytes)?;
    let patch_text = std::str::from_utf8(patch_bytes)
        .map_err(|error| UnifiedPatchError::Invalid(format!("patch is not UTF-8: {error}")))?;
    let patch = parse_unified_patch(patch_text)?;
    let mut applied = Vec::with_capacity(patch.files.len());
    for file_patch in &patch.files {
        let path = root.join(&file_patch.relative_path);
        let original = fs::read_to_string(&path).map_err(|source| UnifiedPatchError::Io {
            action: format!("reading patch target {}", path.display()),
            source,
        })?;
        let updated = apply_file_patch(&original, file_patch)?;
        fs::write(&path, updated.as_bytes()).map_err(|source| UnifiedPatchError::Io {
            action: format!("writing patch target {}", path.display()),
            source,
        })?;
        applied.push(file_patch.relative_path.clone());
    }
    assert_eq!(applied.len(), patch.files.len());
    assert!(applied.iter().all(|path| !path.is_empty()));
    Ok(applied)
}

fn validate_patch_shell_inputs(root: &Path, patch_bytes: &[u8]) -> Result<(), UnifiedPatchError> {
    if !root.is_absolute() || !root.is_dir() {
        return Err(UnifiedPatchError::Invalid(format!("patch root is not an absolute directory: {}", root.display())));
    }
    if patch_bytes.is_empty() || patch_bytes.len() > PATCH_BYTES_MAX {
        return Err(UnifiedPatchError::Invalid(format!(
            "patch byte count {} is outside 1..={PATCH_BYTES_MAX}",
            patch_bytes.len()
        )));
    }
    assert!(root.is_absolute());
    assert!(!patch_bytes.is_empty());
    Ok(())
}

fn parse_unified_patch(text: &str) -> Result<UnifiedPatch, UnifiedPatchError> {
    if !text.ends_with('\n') {
        return Err(UnifiedPatchError::Invalid("patch must end with a newline".to_string()));
    }
    let lines = text.lines().collect::<Vec<_>>();
    validate_patch_lines(&lines)?;
    let mut index = 0usize;
    let mut files = Vec::new();
    let mut hunk_count = 0usize;
    while index < lines.len() {
        let (file_patch, next_index) = parse_file_patch(&lines, index, &mut hunk_count)?;
        files.push(file_patch);
        if files.len() > PATCH_FILE_COUNT_MAX {
            return Err(UnifiedPatchError::Invalid(format!("patch exceeds {PATCH_FILE_COUNT_MAX} files")));
        }
        index = next_index;
    }
    if files.is_empty() {
        return Err(UnifiedPatchError::Invalid("patch contains no files".to_string()));
    }
    assert!(index == lines.len());
    assert!(!files.is_empty());
    Ok(UnifiedPatch { files })
}

fn validate_patch_lines(lines: &[&str]) -> Result<(), UnifiedPatchError> {
    if lines.len() > PATCH_HUNK_LINE_COUNT_MAX {
        return Err(UnifiedPatchError::Invalid(format!("patch exceeds {PATCH_HUNK_LINE_COUNT_MAX} lines")));
    }
    for line in lines {
        if line.len() > PATCH_LINE_BYTES_MAX {
            return Err(UnifiedPatchError::Invalid(format!("patch line exceeds {PATCH_LINE_BYTES_MAX} bytes")));
        }
    }
    assert!(!lines.is_empty());
    assert!(lines.len() <= PATCH_HUNK_LINE_COUNT_MAX);
    Ok(())
}

fn parse_file_patch(
    lines: &[&str],
    start_index: usize,
    hunk_count: &mut usize,
) -> Result<(FilePatch, usize), UnifiedPatchError> {
    let diff_line = required_line(lines, start_index, "diff header")?;
    if !diff_line.starts_with("diff --git ") {
        return Err(UnifiedPatchError::Invalid(format!(
            "expected diff header at line {}, observed {diff_line:?}",
            start_index.saturating_add(1)
        )));
    }
    let mut index = start_index.saturating_add(1);
    if required_line(lines, index, "index line")?.starts_with("index ") {
        index = index.saturating_add(1);
    }
    let old_line = required_line(lines, index, "old path")?;
    index = index.saturating_add(1);
    let new_line = required_line(lines, index, "new path")?;
    index = index.saturating_add(1);
    let relative_path = parse_patch_paths(old_line, new_line)?;
    let mut hunks = Vec::new();
    while index < lines.len() && !lines[index].starts_with("diff --git ") {
        let (hunk, next_index) = parse_hunk(lines, index)?;
        hunks.push(hunk);
        *hunk_count = hunk_count
            .checked_add(1)
            .ok_or_else(|| UnifiedPatchError::Invalid("patch hunk count overflow".to_string()))?;
        if *hunk_count > PATCH_HUNK_COUNT_MAX {
            return Err(UnifiedPatchError::Invalid(format!("patch exceeds {PATCH_HUNK_COUNT_MAX} hunks")));
        }
        index = next_index;
    }
    if hunks.is_empty() {
        return Err(UnifiedPatchError::Invalid(format!("patch file {relative_path} has no hunks")));
    }
    assert!(!relative_path.is_empty());
    assert!(!hunks.is_empty());
    Ok((FilePatch { relative_path, hunks }, index))
}

fn parse_patch_paths(old_line: &str, new_line: &str) -> Result<String, UnifiedPatchError> {
    let old_path = old_line
        .strip_prefix("--- a/raw/")
        .ok_or_else(|| UnifiedPatchError::Invalid(format!("unsupported old path line {old_line:?}")))?;
    let new_path = new_line
        .strip_prefix("+++ b/patched/")
        .ok_or_else(|| UnifiedPatchError::Invalid(format!("unsupported new path line {new_line:?}")))?;
    if old_path != new_path || old_path.is_empty() || old_path.len() > PATCH_PATH_BYTES_MAX {
        return Err(UnifiedPatchError::Invalid(format!(
            "patch path pair is substituted or outside limits: {old_path:?} and {new_path:?}"
        )));
    }
    let path = Path::new(old_path);
    let component_count = path.components().count();
    let all_normal = path.components().all(|component| matches!(component, Component::Normal(_)));
    if component_count != 1 || !all_normal {
        return Err(UnifiedPatchError::Invalid(format!("patch path must be one relative filename: {old_path}")));
    }
    assert_eq!(old_path, new_path);
    assert_eq!(component_count, 1);
    Ok(old_path.to_string())
}

fn parse_hunk(lines: &[&str], start_index: usize) -> Result<(Hunk, usize), UnifiedPatchError> {
    let header = required_line(lines, start_index, "hunk header")?;
    let (old_start, old_count, new_start, new_count) = parse_hunk_header(header)?;
    let mut index = start_index.saturating_add(1);
    let mut hunk_lines = Vec::new();
    while index < lines.len() {
        let line = lines[index];
        if line.starts_with(HUNK_HEADER_PREFIX) || line.starts_with("diff --git ") {
            break;
        }
        let hunk_line = parse_hunk_line(line, index)?;
        hunk_lines.push(hunk_line);
        index = index.saturating_add(1);
    }
    validate_hunk_counts(old_count, new_count, &hunk_lines)?;
    assert!(!hunk_lines.is_empty());
    assert!(index > start_index);
    Ok((
        Hunk {
            old_start,
            old_count,
            new_start,
            new_count,
            lines: hunk_lines,
        },
        index,
    ))
}

fn parse_hunk_header(header: &str) -> Result<(u32, u32, u32, u32), UnifiedPatchError> {
    let body = header
        .strip_prefix(HUNK_HEADER_PREFIX)
        .and_then(|value| value.split_once(HUNK_HEADER_SUFFIX).map(|pair| pair.0))
        .ok_or_else(|| UnifiedPatchError::Invalid(format!("malformed hunk header {header:?}")))?;
    let mut parts = body.split_ascii_whitespace();
    let old = parts
        .next()
        .ok_or_else(|| UnifiedPatchError::Invalid(format!("hunk header lacks old range: {header}")))?;
    let new = parts
        .next()
        .ok_or_else(|| UnifiedPatchError::Invalid(format!("hunk header lacks new range: {header}")))?;
    if parts.next().is_some() {
        return Err(UnifiedPatchError::Invalid(format!("hunk header has extra range fields: {header}")));
    }
    let (old_start, old_count) = parse_hunk_range(old, '-')?;
    let (new_start, new_count) = parse_hunk_range(new, '+')?;
    assert!(old_start > 0);
    assert!(new_start > 0);
    Ok((old_start, old_count, new_start, new_count))
}

fn parse_hunk_range(text: &str, prefix: char) -> Result<(u32, u32), UnifiedPatchError> {
    let range = text
        .strip_prefix(prefix)
        .ok_or_else(|| UnifiedPatchError::Invalid(format!("hunk range {text:?} lacks {prefix}")))?;
    let (start, count) = range.split_once(',').unwrap_or((range, "1"));
    let start = start
        .parse::<u32>()
        .map_err(|error| UnifiedPatchError::Invalid(format!("invalid hunk start {start:?}: {error}")))?;
    let count = count
        .parse::<u32>()
        .map_err(|error| UnifiedPatchError::Invalid(format!("invalid hunk count {count:?}: {error}")))?;
    if start == 0 {
        return Err(UnifiedPatchError::Invalid("new-file hunks are not supported".to_string()));
    }
    assert!(start > 0);
    assert!(text.starts_with(prefix));
    Ok((start, count))
}

fn parse_hunk_line(line: &str, index: usize) -> Result<HunkLine, UnifiedPatchError> {
    let (prefix, content) = line.split_at_checked(1).ok_or_else(|| {
        UnifiedPatchError::Invalid(format!("empty hunk line at patch line {}", index.saturating_add(1)))
    })?;
    match prefix {
        " " => Ok(HunkLine::Context(content.to_string())),
        "-" => Ok(HunkLine::Remove(content.to_string())),
        "+" => Ok(HunkLine::Add(content.to_string())),
        _ => Err(UnifiedPatchError::Invalid(format!(
            "unsupported hunk line prefix {prefix:?} at line {}",
            index.saturating_add(1)
        ))),
    }
}

fn validate_hunk_counts(old_count: u32, new_count: u32, lines: &[HunkLine]) -> Result<(), UnifiedPatchError> {
    let observed_old = lines.iter().filter(|line| !matches!(line, HunkLine::Add(_))).count();
    let observed_new = lines.iter().filter(|line| !matches!(line, HunkLine::Remove(_))).count();
    if observed_old != usize::try_from(old_count).unwrap_or(usize::MAX)
        || observed_new != usize::try_from(new_count).unwrap_or(usize::MAX)
    {
        return Err(UnifiedPatchError::Invalid(format!(
            "hunk count mismatch: header old/new {old_count}/{new_count}, observed {observed_old}/{observed_new}"
        )));
    }
    assert!(!lines.is_empty());
    assert!(observed_old > 0 || observed_new > 0);
    Ok(())
}

fn apply_file_patch(original: &str, patch: &FilePatch) -> Result<String, UnifiedPatchError> {
    if !original.ends_with('\n') {
        return Err(UnifiedPatchError::Invalid(format!(
            "patch target {} must end with a newline",
            patch.relative_path
        )));
    }
    let original_lines = original.lines().collect::<Vec<_>>();
    let mut output = Vec::new();
    let mut cursor = 0usize;
    for hunk in &patch.hunks {
        let target = usize::try_from(hunk.old_start.saturating_sub(1))
            .map_err(|_| UnifiedPatchError::Invalid("hunk start does not fit usize".to_string()))?;
        if target < cursor || target > original_lines.len() {
            return Err(UnifiedPatchError::Invalid(format!(
                "hunk for {} starts outside ordered target lines",
                patch.relative_path
            )));
        }
        output.extend(original_lines[cursor..target].iter().map(|line| (*line).to_string()));
        cursor = target;
        apply_hunk_lines(&original_lines, &mut cursor, &mut output, hunk, &patch.relative_path)?;
    }
    output.extend(original_lines[cursor..].iter().map(|line| (*line).to_string()));
    let mut updated = output.join("\n");
    updated.push('\n');
    assert!(updated.ends_with('\n'));
    assert!(!updated.is_empty());
    Ok(updated)
}

fn apply_hunk_lines(
    original_lines: &[&str],
    cursor: &mut usize,
    output: &mut Vec<String>,
    hunk: &Hunk,
    relative_path: &str,
) -> Result<(), UnifiedPatchError> {
    for line in &hunk.lines {
        match line {
            HunkLine::Context(expected) => {
                require_original_line(original_lines, *cursor, expected, relative_path)?;
                output.push(expected.clone());
                *cursor = cursor.saturating_add(1);
            }
            HunkLine::Remove(expected) => {
                require_original_line(original_lines, *cursor, expected, relative_path)?;
                *cursor = cursor.saturating_add(1);
            }
            HunkLine::Add(value) => output.push(value.clone()),
        }
    }
    assert!(*cursor <= original_lines.len());
    assert!(output.len() <= original_lines.len().saturating_add(PATCH_HUNK_LINE_COUNT_MAX));
    Ok(())
}

fn require_original_line(
    original_lines: &[&str],
    index: usize,
    expected: &str,
    relative_path: &str,
) -> Result<(), UnifiedPatchError> {
    let observed = original_lines.get(index).copied().ok_or_else(|| {
        UnifiedPatchError::Invalid(format!(
            "patch context for {relative_path} reaches beyond source at line {}",
            index.saturating_add(1)
        ))
    })?;
    if observed != expected {
        return Err(UnifiedPatchError::Invalid(format!(
            "patch context mismatch for {relative_path} at line {}: expected {expected:?}, observed {observed:?}",
            index.saturating_add(1)
        )));
    }
    assert_eq!(observed, expected);
    assert!(index < original_lines.len());
    Ok(())
}

fn required_line<'a>(lines: &'a [&str], index: usize, label: &str) -> Result<&'a str, UnifiedPatchError> {
    lines
        .get(index)
        .copied()
        .ok_or_else(|| UnifiedPatchError::Invalid(format!("patch ended before required {label}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    const PATCH: &str = "diff --git a/raw/sample.c b/patched/sample.c\nindex 11111111..22222222 100644\n--- a/raw/sample.c\n+++ b/patched/sample.c\n@@ -1,3 +1,4 @@\n first\n-old\n+new\n+extra\n last\n";

    #[test]
    fn applies_exact_patch_and_rejects_context_substitution() {
        let patch = parse_unified_patch(PATCH).unwrap();
        let updated = apply_file_patch("first\nold\nlast\n", &patch.files[0]).unwrap();
        assert_eq!(updated, "first\nnew\nextra\nlast\n");
        assert!(!updated.contains("\nold\n"));

        let error = apply_file_patch("first\nsubstituted\nlast\n", &patch.files[0]).unwrap_err();
        assert!(error.to_string().contains("patch context mismatch"));
        assert!(!error.to_string().is_empty());
    }

    #[test]
    fn rejects_path_escape_and_hunk_count_substitution() {
        let escaped = PATCH.replace("sample.c", "../sample.c");
        let error = parse_unified_patch(&escaped).unwrap_err();
        assert!(error.to_string().contains("one relative filename"));
        assert!(!error.to_string().is_empty());

        let substituted_count = PATCH.replace("@@ -1,3 +1,4 @@", "@@ -1,2 +1,4 @@");
        let error = parse_unified_patch(&substituted_count).unwrap_err();
        assert!(error.to_string().contains("hunk count mismatch"));
        assert!(!error.to_string().is_empty());
    }
}
