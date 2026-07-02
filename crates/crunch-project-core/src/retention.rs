use alloc::collections::BTreeMap;
use alloc::collections::BTreeSet;
use alloc::format;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec;
use alloc::vec::Vec;
use core::cmp::Ordering;

use serde::Deserialize;
use serde::Serialize;

use crate::LockEntry;
use crate::LockedKind;
use crate::Lockfile;
use crate::ProjectManifest;

pub const RETENTION_STATE_SCHEMA: &str = "mantle-project-retention-state-v1";
pub const RETENTION_ROOT_KIND_SOURCE_MATERIAL: &str = "source-material";
pub const MAX_RETENTION_GENERATIONS: u32 = 128;
pub const MAX_RETENTION_ROOT_RECORDS: u32 = 4096;
const INITIAL_RETENTION_GENERATION: u32 = 1;
const LOCK_ENTRY_DIGEST_DOMAIN: &str = "mantle-lock-entry-v1";
const RETENTION_ROOT_ID_DOMAIN: &str = "mantle-retention-root-v1";
const HASH_FIELD_SEPARATOR: &[u8] = b"\0";
const HASH_RECORD_SEPARATOR: &[u8] = b"\n";
const DIAGNOSTIC_RANK_INVALID_POLICY: u32 = 10;
const DIAGNOSTIC_RANK_UNKNOWN_INPUT: u32 = 20;
const DIAGNOSTIC_RANK_INTERRUPTED: u32 = 30;
const DIAGNOSTIC_RANK_MISSING_ROOT: u32 = 40;
const DIAGNOSTIC_RANK_STALE_ROOT: u32 = 50;
const DIAGNOSTIC_RANK_GC_ELIGIBLE: u32 = 60;
const ACTION_RANK_CREATE: u32 = 10;
const ACTION_RANK_QUARANTINE: u32 = 20;
const ACTION_RANK_REMOVE: u32 = 30;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(tag = "mode", rename_all = "kebab-case")]
pub enum InputRetentionPolicy {
    #[default]
    Untracked,
    Current,
    RecentGenerations {
        generations: u32,
    },
}

impl InputRetentionPolicy {
    pub fn tracked_generation_limit(self) -> u32 {
        match self {
            Self::Untracked => 0,
            Self::Current => INITIAL_RETENTION_GENERATION,
            Self::RecentGenerations { generations } => generations,
        }
    }

    pub fn validate(self, owner: &str) -> Vec<String> {
        let mut problems = Vec::new();
        if let Self::RecentGenerations { generations } = self {
            if generations == 0 {
                problems.push(format!("{owner}: retention recent-generations must keep at least one generation"));
            }
            if generations > MAX_RETENTION_GENERATIONS {
                problems.push(format!(
                    "{owner}: retention recent-generations keeps {generations} generations (max {MAX_RETENTION_GENERATIONS})"
                ));
            }
        }
        problems
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RetentionRootKind {
    SourceMaterial,
}

impl RetentionRootKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::SourceMaterial => RETENTION_ROOT_KIND_SOURCE_MATERIAL,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RetentionRootRecord {
    pub input_name: String,
    pub lock_digest: String,
    pub source_identity: String,
    pub content_digest: String,
    pub generation: u32,
    pub root_kind: RetentionRootKind,
    pub root_id: String,
    pub committed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectRetentionState {
    pub schema: String,
    pub records: Vec<RetentionRootRecord>,
}

impl ProjectRetentionState {
    pub fn empty() -> Self {
        Self {
            schema: RETENTION_STATE_SCHEMA.into(),
            records: Vec::new(),
        }
    }

    pub fn validate(&self) -> Vec<String> {
        let mut problems = Vec::new();
        if self.schema != RETENTION_STATE_SCHEMA {
            problems.push(format!("retention state schema '{}' is unsupported", self.schema));
        }
        if self.records.len() as u64 > MAX_RETENTION_ROOT_RECORDS as u64 {
            problems.push(format!(
                "too many retention root records: {} (max {MAX_RETENTION_ROOT_RECORDS})",
                self.records.len()
            ));
        }
        for record in &self.records {
            push_record_validation(record, &mut problems);
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RetentionRootFact {
    pub record: RetentionRootRecord,
    pub root_exists: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionPlanRequest {
    pub manifest: ProjectManifest,
    pub lock: Lockfile,
    pub existing_roots: Vec<RetentionRootFact>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RetentionPlan {
    pub statuses: Vec<RetentionInputStatus>,
    pub diagnostics: Vec<RetentionDiagnostic>,
    pub actions: Vec<RetentionRootAction>,
    pub retained_records: Vec<RetentionRootRecord>,
    pub next_generation: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RetentionInputStatus {
    pub input_name: String,
    pub policy: InputRetentionPolicy,
    pub state: RetentionInputState,
    pub gc_eligible: bool,
    pub lock_digest: Option<String>,
    pub root_id: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum RetentionInputState {
    Pinned,
    Unpinned,
    MissingRoot,
    StaleRoot,
    GcEligible,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RetentionDiagnostic {
    pub input_name: String,
    pub kind: RetentionDiagnosticKind,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum RetentionDiagnosticKind {
    InvalidPolicy,
    RootForUnknownInput,
    InterruptedUpdate,
    MissingRoot,
    StaleRoot,
    GcEligible,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RetentionRootAction {
    pub input_name: String,
    pub kind: RetentionRootActionKind,
    pub record: RetentionRootRecord,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum RetentionRootActionKind {
    CreateRoot,
    RemoveRoot,
    QuarantineInterrupted,
}

struct PlannedInputRetention {
    status: RetentionInputStatus,
    diagnostics: Vec<RetentionDiagnostic>,
    actions: Vec<RetentionRootAction>,
    retained_records: Vec<RetentionRootRecord>,
}

pub fn plan_retention_roots(request: RetentionPlanRequest) -> RetentionPlan {
    assert!(request.manifest.inputs.len() as u64 <= crate::MAX_INPUTS as u64, "manifest input limit exceeded");
    assert!(
        request.existing_roots.len() as u64 <= MAX_RETENTION_ROOT_RECORDS as u64,
        "retention root fact limit exceeded"
    );
    let declared_names = declared_input_names(&request.manifest);
    let next_generation = next_generation(&request.existing_roots);
    let roots_by_input = roots_by_input(&request.existing_roots);
    let mut plan = empty_plan(next_generation);

    push_unknown_root_actions(&request.existing_roots, &declared_names, &mut plan);
    for input in &request.manifest.inputs {
        let policy = input.retention.unwrap_or(request.manifest.retention);
        let roots = roots_by_input.get(input.name.as_str()).cloned().unwrap_or_default();
        let planned = plan_one_input(&input.name, policy, request.lock.inputs.get(&input.name), roots, next_generation);
        merge_planned_input(&mut plan, planned);
    }

    sort_plan(&mut plan);
    plan
}

pub fn retention_record_for_input(input_name: &str, entry: &LockEntry, generation: u32) -> RetentionRootRecord {
    assert!(!input_name.is_empty(), "input name must not be empty");
    assert!(!entry.hash.value.is_empty(), "lock entry hash must not be empty");
    assert!(generation > 0, "retention generation must be nonzero");
    let lock_digest = lock_entry_digest(input_name, entry);
    let source_identity = locked_source_identity(&entry.kind);
    let content_digest = entry.hash.value.clone();
    let root_id = retention_root_id(input_name, &lock_digest, &source_identity, &content_digest);
    RetentionRootRecord {
        input_name: input_name.into(),
        lock_digest,
        source_identity,
        content_digest,
        generation,
        root_kind: RetentionRootKind::SourceMaterial,
        root_id,
        committed: true,
    }
}

pub fn lock_entry_digest(input_name: &str, entry: &LockEntry) -> String {
    assert!(!input_name.is_empty(), "input name must not be empty");
    let mut hasher = blake3::Hasher::new();
    hash_field(&mut hasher, "domain", LOCK_ENTRY_DIGEST_DOMAIN);
    hash_field(&mut hasher, "input", input_name);
    hash_locked_kind(&mut hasher, &entry.kind);
    hash_field(&mut hasher, "hash_algo", &entry.hash.algo.to_string());
    hash_field(&mut hasher, "hash_value", &entry.hash.value);
    hash_string_vec(&mut hasher, "patch", &entry.patches);
    hash_string_vec(&mut hasher, "mirror", &entry.mirrors);
    hash_field(&mut hasher, "fetch_policy", entry.fetch_policy.as_str());
    if let Some(freshness) = &entry.freshness {
        hash_field(&mut hasher, "freshness_input", &freshness.input_name);
        hash_field(&mut hasher, "freshness_digest", &freshness.value_digest);
    }
    hasher.finalize().to_hex().to_string()
}

pub fn locked_source_identity(kind: &LockedKind) -> String {
    match kind {
        LockedKind::File { url } => format!("file:{url}"),
        LockedKind::Tarball { url } => format!("tarball:{url}"),
        LockedKind::Git {
            repository,
            rev,
            ref_name,
        } => format!("git:{repository}@{}:{rev}", ref_name.clone().unwrap_or_else(|| "<detached>".into())),
        LockedKind::Darcs {
            repository,
            selector,
            context,
            weak_hash,
        } => format!(
            "darcs:{repository}@{}:{}",
            selector.identity_fragment(),
            context.clone().or_else(|| weak_hash.clone()).unwrap_or_else(|| "<unresolved>".into())
        ),
        LockedKind::Pijul {
            repository,
            selector,
            state,
            change,
        } => format!(
            "pijul:{repository}@{}:{state}:{}",
            selector.identity_fragment(),
            change.clone().unwrap_or_else(|| "<no-change>".into())
        ),
        LockedKind::Fossil {
            repository,
            selector,
            checkin,
        } => format!("fossil:{repository}@{}:{checkin}", selector.identity_fragment()),
    }
}

fn plan_one_input(
    input_name: &str,
    policy: InputRetentionPolicy,
    entry: Option<&LockEntry>,
    roots: Vec<&RetentionRootFact>,
    next_generation: u32,
) -> PlannedInputRetention {
    assert!(!input_name.is_empty(), "input name must not be empty");
    let mut planned = empty_planned_input(input_name, policy);
    planned.diagnostics.extend(policy_diagnostics(input_name, policy));
    if policy == InputRetentionPolicy::Untracked {
        plan_untracked_input(input_name, roots, &mut planned);
        return planned;
    }
    let Some(entry) = entry else {
        plan_unpinned_input(input_name, roots, &mut planned);
        return planned;
    };
    plan_tracked_input(input_name, policy, entry, roots, next_generation, &mut planned);
    planned
}

fn plan_tracked_input(
    input_name: &str,
    policy: InputRetentionPolicy,
    entry: &LockEntry,
    roots: Vec<&RetentionRootFact>,
    next_generation: u32,
    planned: &mut PlannedInputRetention,
) {
    let desired = desired_current_record(input_name, entry, &roots, next_generation);
    let current_root = roots.iter().find(|fact| root_satisfies_current(fact, &desired));
    let retained = retained_records_for_policy(policy, desired.clone(), &roots);
    let current_pinned = current_root.is_some();
    let stale_roots = stale_roots(&roots, &retained);

    planned.retained_records = retained;
    planned.status = tracked_status(input_name, policy, &desired, current_pinned, !stale_roots.is_empty());
    if !current_pinned {
        planned.actions.push(create_action(input_name, desired.clone()));
        planned.diagnostics.push(missing_root_diagnostic(input_name, &desired));
    }
    push_stale_root_diagnostics(input_name, stale_roots, planned);
}

fn plan_untracked_input(input_name: &str, roots: Vec<&RetentionRootFact>, planned: &mut PlannedInputRetention) {
    planned.status.state = RetentionInputState::GcEligible;
    planned.status.gc_eligible = true;
    planned.diagnostics.push(RetentionDiagnostic {
        input_name: input_name.into(),
        kind: RetentionDiagnosticKind::GcEligible,
        message: format!(
            "input '{input_name}' is untracked by retention policy and remains garbage-collection eligible"
        ),
    });
    for root in roots {
        planned.actions.push(remove_action(input_name, root.record.clone()));
    }
}

fn plan_unpinned_input(input_name: &str, roots: Vec<&RetentionRootFact>, planned: &mut PlannedInputRetention) {
    planned.status.state = RetentionInputState::Unpinned;
    planned.status.gc_eligible = true;
    planned.diagnostics.push(RetentionDiagnostic {
        input_name: input_name.into(),
        kind: RetentionDiagnosticKind::MissingRoot,
        message: format!("input '{input_name}' has retention policy but no lock entry to pin"),
    });
    for root in roots {
        planned.actions.push(remove_action(input_name, root.record.clone()));
    }
}

fn desired_current_record(
    input_name: &str,
    entry: &LockEntry,
    roots: &[&RetentionRootFact],
    next_generation: u32,
) -> RetentionRootRecord {
    let probe = retention_record_for_input(input_name, entry, next_generation);
    if let Some(existing) = roots.iter().find(|fact| record_matches_digest(&fact.record, &probe)) {
        retention_record_for_input(input_name, entry, existing.record.generation)
    } else {
        probe
    }
}

fn retained_records_for_policy(
    policy: InputRetentionPolicy,
    desired: RetentionRootRecord,
    roots: &[&RetentionRootFact],
) -> Vec<RetentionRootRecord> {
    let limit = policy.tracked_generation_limit();
    if limit == 0 {
        return Vec::new();
    }
    let mut retained = vec![desired.clone()];
    let mut previous = roots
        .iter()
        .filter(|fact| previous_root_is_eligible(fact, &desired))
        .map(|fact| fact.record.clone())
        .collect::<Vec<_>>();
    previous.sort_by(compare_records_newest_first);
    for record in previous.into_iter().take(limit.saturating_sub(1) as usize) {
        retained.push(record);
    }
    retained.sort_by(compare_records_oldest_first);
    retained
}

fn tracked_status(
    input_name: &str,
    policy: InputRetentionPolicy,
    desired: &RetentionRootRecord,
    current_pinned: bool,
    has_stale_roots: bool,
) -> RetentionInputStatus {
    let state = match (current_pinned, has_stale_roots) {
        (true, false) => RetentionInputState::Pinned,
        (true, true) => RetentionInputState::StaleRoot,
        (false, _) => RetentionInputState::MissingRoot,
    };
    RetentionInputStatus {
        input_name: input_name.into(),
        policy,
        state,
        gc_eligible: false,
        lock_digest: Some(desired.lock_digest.clone()),
        root_id: current_pinned.then(|| desired.root_id.clone()),
    }
}

fn push_stale_root_diagnostics(
    input_name: &str,
    stale_roots: Vec<&RetentionRootFact>,
    planned: &mut PlannedInputRetention,
) {
    for root in stale_roots {
        let kind = if root.record.committed {
            RetentionDiagnosticKind::StaleRoot
        } else {
            RetentionDiagnosticKind::InterruptedUpdate
        };
        planned.diagnostics.push(RetentionDiagnostic {
            input_name: input_name.into(),
            kind,
            message: stale_root_message(input_name, root),
        });
        planned.actions.push(action_for_stale_root(input_name, root));
    }
}

fn push_unknown_root_actions(facts: &[RetentionRootFact], declared_names: &BTreeSet<&str>, plan: &mut RetentionPlan) {
    for fact in facts {
        let input_name = fact.record.input_name.as_str();
        if declared_names.contains(input_name) {
            continue;
        }
        plan.diagnostics.push(RetentionDiagnostic {
            input_name: fact.record.input_name.clone(),
            kind: RetentionDiagnosticKind::RootForUnknownInput,
            message: format!("retention root '{}' belongs to undeclared input '{input_name}'", fact.record.root_id),
        });
        plan.actions.push(remove_action(input_name, fact.record.clone()));
    }
}

fn policy_diagnostics(input_name: &str, policy: InputRetentionPolicy) -> Vec<RetentionDiagnostic> {
    policy
        .validate(&format!("input '{input_name}'"))
        .into_iter()
        .map(|message| RetentionDiagnostic {
            input_name: input_name.into(),
            kind: RetentionDiagnosticKind::InvalidPolicy,
            message,
        })
        .collect()
}

fn empty_plan(next_generation: u32) -> RetentionPlan {
    RetentionPlan {
        statuses: Vec::new(),
        diagnostics: Vec::new(),
        actions: Vec::new(),
        retained_records: Vec::new(),
        next_generation,
    }
}

fn empty_planned_input(input_name: &str, policy: InputRetentionPolicy) -> PlannedInputRetention {
    PlannedInputRetention {
        status: RetentionInputStatus {
            input_name: input_name.into(),
            policy,
            state: RetentionInputState::Unpinned,
            gc_eligible: false,
            lock_digest: None,
            root_id: None,
        },
        diagnostics: Vec::new(),
        actions: Vec::new(),
        retained_records: Vec::new(),
    }
}

fn merge_planned_input(plan: &mut RetentionPlan, planned: PlannedInputRetention) {
    plan.statuses.push(planned.status);
    plan.diagnostics.extend(planned.diagnostics);
    plan.actions.extend(planned.actions);
    plan.retained_records.extend(planned.retained_records);
}

fn roots_by_input(facts: &[RetentionRootFact]) -> BTreeMap<&str, Vec<&RetentionRootFact>> {
    let mut by_input: BTreeMap<&str, Vec<&RetentionRootFact>> = BTreeMap::new();
    for fact in facts {
        by_input.entry(fact.record.input_name.as_str()).or_default().push(fact);
    }
    by_input
}

fn declared_input_names(manifest: &ProjectManifest) -> BTreeSet<&str> {
    manifest.inputs.iter().map(|input| input.name.as_str()).collect()
}

fn next_generation(facts: &[RetentionRootFact]) -> u32 {
    facts
        .iter()
        .map(|fact| fact.record.generation)
        .max()
        .unwrap_or(0)
        .saturating_add(INITIAL_RETENTION_GENERATION)
}

fn stale_roots<'a>(roots: &'a [&RetentionRootFact], retained: &[RetentionRootRecord]) -> Vec<&'a RetentionRootFact> {
    roots.iter().copied().filter(|fact| !retained_record_satisfies_fact(fact, retained)).collect()
}

fn retained_record_satisfies_fact(fact: &RetentionRootFact, retained: &[RetentionRootRecord]) -> bool {
    root_is_durable(fact) && retained.iter().any(|record| record_matches_digest(&fact.record, record))
}

fn root_satisfies_current(fact: &RetentionRootFact, desired: &RetentionRootRecord) -> bool {
    root_is_durable(fact) && record_matches_digest(&fact.record, desired)
}

fn previous_root_is_eligible(fact: &RetentionRootFact, desired: &RetentionRootRecord) -> bool {
    root_is_durable(fact)
        && fact.record.input_name == desired.input_name
        && fact.record.root_id != desired.root_id
        && fact.record.root_kind == RetentionRootKind::SourceMaterial
}

fn root_is_durable(fact: &RetentionRootFact) -> bool {
    fact.root_exists && fact.record.committed
}

fn record_matches_digest(left: &RetentionRootRecord, right: &RetentionRootRecord) -> bool {
    left.input_name == right.input_name
        && left.lock_digest == right.lock_digest
        && left.source_identity == right.source_identity
        && left.content_digest == right.content_digest
        && left.root_kind == right.root_kind
}

fn missing_root_diagnostic(input_name: &str, desired: &RetentionRootRecord) -> RetentionDiagnostic {
    RetentionDiagnostic {
        input_name: input_name.into(),
        kind: RetentionDiagnosticKind::MissingRoot,
        message: format!(
            "input '{input_name}' requires retention root '{}' for lock digest {} but it is not committed",
            desired.root_id, desired.lock_digest
        ),
    }
}

fn stale_root_message(input_name: &str, fact: &RetentionRootFact) -> String {
    if !fact.record.committed {
        return format!(
            "input '{input_name}' has interrupted retention root '{}' that is not committed",
            fact.record.root_id
        );
    }
    if !fact.root_exists {
        return format!("input '{input_name}' has retention state for missing root '{}'", fact.record.root_id);
    }
    format!(
        "input '{input_name}' has stale retention root '{}' at generation {}",
        fact.record.root_id, fact.record.generation
    )
}

fn create_action(input_name: &str, record: RetentionRootRecord) -> RetentionRootAction {
    RetentionRootAction {
        input_name: input_name.into(),
        kind: RetentionRootActionKind::CreateRoot,
        record,
    }
}

fn remove_action(input_name: &str, record: RetentionRootRecord) -> RetentionRootAction {
    RetentionRootAction {
        input_name: input_name.into(),
        kind: RetentionRootActionKind::RemoveRoot,
        record,
    }
}

fn action_for_stale_root(input_name: &str, fact: &RetentionRootFact) -> RetentionRootAction {
    if fact.record.committed {
        remove_action(input_name, fact.record.clone())
    } else {
        RetentionRootAction {
            input_name: input_name.into(),
            kind: RetentionRootActionKind::QuarantineInterrupted,
            record: fact.record.clone(),
        }
    }
}

fn push_record_validation(record: &RetentionRootRecord, problems: &mut Vec<String>) {
    if record.input_name.is_empty() {
        problems.push("retention record input name must not be empty".into());
    }
    if record.lock_digest.is_empty() {
        problems.push(format!("retention record '{}': lock digest is empty", record.input_name));
    }
    if record.source_identity.is_empty() {
        problems.push(format!("retention record '{}': source identity is empty", record.input_name));
    }
    if record.content_digest.is_empty() {
        problems.push(format!("retention record '{}': content digest is empty", record.input_name));
    }
    if record.generation == 0 {
        problems.push(format!("retention record '{}': generation must be nonzero", record.input_name));
    }
    if record.root_id.is_empty() {
        problems.push(format!("retention record '{}': root id is empty", record.input_name));
    }
}

fn sort_plan(plan: &mut RetentionPlan) {
    plan.statuses.sort_by(|left, right| left.input_name.cmp(&right.input_name));
    plan.diagnostics.sort_by(compare_diagnostics);
    plan.actions.sort_by(compare_actions);
    plan.retained_records.sort_by(compare_records_oldest_first);
    plan.retained_records.dedup_by(|left, right| left.root_id == right.root_id);
}

fn compare_diagnostics(left: &RetentionDiagnostic, right: &RetentionDiagnostic) -> Ordering {
    left.input_name
        .cmp(&right.input_name)
        .then_with(|| diagnostic_rank(left.kind).cmp(&diagnostic_rank(right.kind)))
        .then_with(|| left.message.cmp(&right.message))
}

fn compare_actions(left: &RetentionRootAction, right: &RetentionRootAction) -> Ordering {
    action_rank(left.kind)
        .cmp(&action_rank(right.kind))
        .then_with(|| left.input_name.cmp(&right.input_name))
        .then_with(|| left.record.generation.cmp(&right.record.generation))
        .then_with(|| left.record.root_id.cmp(&right.record.root_id))
}

fn compare_records_newest_first(left: &RetentionRootRecord, right: &RetentionRootRecord) -> Ordering {
    right
        .generation
        .cmp(&left.generation)
        .then_with(|| left.input_name.cmp(&right.input_name))
        .then_with(|| left.root_id.cmp(&right.root_id))
}

fn compare_records_oldest_first(left: &RetentionRootRecord, right: &RetentionRootRecord) -> Ordering {
    left.generation
        .cmp(&right.generation)
        .then_with(|| left.input_name.cmp(&right.input_name))
        .then_with(|| left.root_id.cmp(&right.root_id))
}

fn diagnostic_rank(kind: RetentionDiagnosticKind) -> u32 {
    match kind {
        RetentionDiagnosticKind::InvalidPolicy => DIAGNOSTIC_RANK_INVALID_POLICY,
        RetentionDiagnosticKind::RootForUnknownInput => DIAGNOSTIC_RANK_UNKNOWN_INPUT,
        RetentionDiagnosticKind::InterruptedUpdate => DIAGNOSTIC_RANK_INTERRUPTED,
        RetentionDiagnosticKind::MissingRoot => DIAGNOSTIC_RANK_MISSING_ROOT,
        RetentionDiagnosticKind::StaleRoot => DIAGNOSTIC_RANK_STALE_ROOT,
        RetentionDiagnosticKind::GcEligible => DIAGNOSTIC_RANK_GC_ELIGIBLE,
    }
}

fn action_rank(kind: RetentionRootActionKind) -> u32 {
    match kind {
        RetentionRootActionKind::CreateRoot => ACTION_RANK_CREATE,
        RetentionRootActionKind::QuarantineInterrupted => ACTION_RANK_QUARANTINE,
        RetentionRootActionKind::RemoveRoot => ACTION_RANK_REMOVE,
    }
}

fn retention_root_id(input_name: &str, lock_digest: &str, source_identity: &str, content_digest: &str) -> String {
    let mut hasher = blake3::Hasher::new();
    hash_field(&mut hasher, "domain", RETENTION_ROOT_ID_DOMAIN);
    hash_field(&mut hasher, "input", input_name);
    hash_field(&mut hasher, "lock_digest", lock_digest);
    hash_field(&mut hasher, "source_identity", source_identity);
    hash_field(&mut hasher, "content_digest", content_digest);
    hasher.finalize().to_hex().to_string()
}

fn hash_locked_kind(hasher: &mut blake3::Hasher, kind: &LockedKind) {
    match kind {
        LockedKind::File { url } => {
            hash_field(hasher, "kind", "file");
            hash_field(hasher, "url", url);
        }
        LockedKind::Tarball { url } => {
            hash_field(hasher, "kind", "tarball");
            hash_field(hasher, "url", url);
        }
        LockedKind::Git {
            repository,
            rev,
            ref_name,
        } => {
            hash_field(hasher, "kind", "git");
            hash_field(hasher, "repository", repository);
            hash_field(hasher, "rev", rev);
            hash_field(hasher, "ref_name", ref_name.as_deref().unwrap_or(""));
        }
        LockedKind::Darcs {
            repository,
            selector,
            context,
            weak_hash,
        } => {
            hash_field(hasher, "kind", "darcs");
            hash_field(hasher, "repository", repository);
            hash_field(hasher, "selector", &selector.identity_fragment());
            hash_field(hasher, "context", context.as_deref().unwrap_or(""));
            hash_field(hasher, "weak_hash", weak_hash.as_deref().unwrap_or(""));
        }
        LockedKind::Pijul {
            repository,
            selector,
            state,
            change,
        } => {
            hash_field(hasher, "kind", "pijul");
            hash_field(hasher, "repository", repository);
            hash_field(hasher, "selector", &selector.identity_fragment());
            hash_field(hasher, "state", state);
            hash_field(hasher, "change", change.as_deref().unwrap_or(""));
        }
        LockedKind::Fossil {
            repository,
            selector,
            checkin,
        } => {
            hash_field(hasher, "kind", "fossil");
            hash_field(hasher, "repository", repository);
            hash_field(hasher, "selector", &selector.identity_fragment());
            hash_field(hasher, "checkin", checkin);
        }
    }
}

fn hash_string_vec(hasher: &mut blake3::Hasher, label: &str, values: &[String]) {
    hash_field(hasher, &format!("{label}_count"), &values.len().to_string());
    for value in values {
        hash_field(hasher, label, value);
    }
}

fn hash_field(hasher: &mut blake3::Hasher, label: &str, value: &str) {
    hasher.update(label.as_bytes());
    hasher.update(HASH_FIELD_SEPARATOR);
    hasher.update(value.len().to_string().as_bytes());
    hasher.update(HASH_FIELD_SEPARATOR);
    hasher.update(value.as_bytes());
    hasher.update(HASH_RECORD_SEPARATOR);
}

#[cfg(test)]
mod tests {
    use alloc::collections::BTreeMap;
    use alloc::vec;

    use super::*;
    use crate::HashAlgo;
    use crate::LockedHash;
    use crate::ManifestInput;
    use crate::manifest::HashSpec;
    use crate::manifest::InputKind;
    use crate::version::SchemaVersion;

    const HASH_A: &str = "sha256-AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=";
    const HASH_B: &str = "sha256-BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB=";
    const GENERATION_ONE: u32 = 1;
    const GENERATION_TWO: u32 = 2;
    const GENERATION_THREE: u32 = 3;

    fn manifest_with(policy: InputRetentionPolicy) -> ProjectManifest {
        ProjectManifest {
            version: "1.0.0".into(),
            retention: policy,
            inputs: vec![ManifestInput {
                name: "pkg".into(),
                kind: InputKind::File {
                    url: "https://example.com/pkg".into(),
                },
                hash: HashSpec::default(),
                frozen: false,
                mirrors: vec![],
                patches: vec![],
                fetch_policy: crate::InputFetchPolicy::GenerationMaterial,
                retention: None,
                freshness: None,
                trust: None,
            }],
            patches: vec![],
        }
    }

    fn lock_with(hash: &str) -> Lockfile {
        let mut inputs = BTreeMap::new();
        inputs.insert("pkg".into(), LockEntry {
            kind: LockedKind::File {
                url: "https://example.com/pkg".into(),
            },
            hash: LockedHash {
                algo: HashAlgo::Sha256,
                value: hash.into(),
            },
            patches: vec![],
            mirrors: vec![],
            fetch_policy: crate::InputFetchPolicy::GenerationMaterial,
            freshness: None,
            trust: None,
        });
        Lockfile {
            version: SchemaVersion::CURRENT,
            inputs,
            patches: BTreeMap::new(),
        }
    }

    fn fact(record: RetentionRootRecord, root_exists: bool) -> RetentionRootFact {
        RetentionRootFact { record, root_exists }
    }

    fn record(hash: &str, generation: u32) -> RetentionRootRecord {
        let lock = lock_with(hash);
        retention_record_for_input("pkg", lock.inputs.get("pkg").unwrap(), generation)
    }

    fn plan(policy: InputRetentionPolicy, lock: Lockfile, existing_roots: Vec<RetentionRootFact>) -> RetentionPlan {
        plan_retention_roots(RetentionPlanRequest {
            manifest: manifest_with(policy),
            lock,
            existing_roots,
        })
    }

    // r[verify project_workflows.input_retention_roots]
    #[test]
    fn current_policy_plans_root_until_marker_exists() {
        let lock = lock_with(HASH_A);
        let missing = plan(InputRetentionPolicy::Current, lock.clone(), Vec::new());
        assert_eq!(missing.statuses[0].state, RetentionInputState::MissingRoot);
        assert_eq!(missing.actions[0].kind, RetentionRootActionKind::CreateRoot);
        assert_eq!(missing.diagnostics[0].kind, RetentionDiagnosticKind::MissingRoot);

        let current = retention_record_for_input("pkg", lock.inputs.get("pkg").unwrap(), GENERATION_ONE);
        let pinned = plan(InputRetentionPolicy::Current, lock, vec![fact(current.clone(), true)]);
        assert_eq!(pinned.statuses[0].state, RetentionInputState::Pinned);
        assert_eq!(pinned.statuses[0].root_id.as_deref(), Some(current.root_id.as_str()));
        assert!(pinned.actions.is_empty());
    }

    // r[verify project_workflows.input_retention_roots]
    #[test]
    fn input_override_beats_project_default() {
        let mut manifest = manifest_with(InputRetentionPolicy::Untracked);
        manifest.inputs[0].retention = Some(InputRetentionPolicy::Current);
        let lock = lock_with(HASH_A);
        let plan = plan_retention_roots(RetentionPlanRequest {
            manifest,
            lock,
            existing_roots: Vec::new(),
        });
        assert_eq!(plan.statuses[0].policy, InputRetentionPolicy::Current);
        assert_eq!(plan.actions[0].kind, RetentionRootActionKind::CreateRoot);
    }

    // r[verify project_workflows.input_retention_roots]
    #[test]
    fn generation_policy_keeps_bounded_newest_history() {
        let current_lock = lock_with(HASH_A);
        let older = record(HASH_B, GENERATION_ONE);
        let current = retention_record_for_input("pkg", current_lock.inputs.get("pkg").unwrap(), GENERATION_THREE);
        let plan = plan(
            InputRetentionPolicy::RecentGenerations {
                generations: GENERATION_TWO,
            },
            current_lock,
            vec![fact(older.clone(), true), fact(current.clone(), true)],
        );
        let ids = plan.retained_records.iter().map(|record| record.root_id.as_str()).collect::<Vec<_>>();
        assert_eq!(ids.len(), GENERATION_TWO as usize);
        assert!(ids.contains(&older.root_id.as_str()));
        assert!(ids.contains(&current.root_id.as_str()));
        assert!(plan.actions.is_empty());
    }

    // r[verify project_workflows.input_retention_roots]
    #[test]
    fn stale_root_outside_generation_window_is_removed_after_create() {
        let current_lock = lock_with(HASH_A);
        let stale = record(HASH_B, GENERATION_ONE);
        let plan = plan(
            InputRetentionPolicy::RecentGenerations {
                generations: GENERATION_ONE,
            },
            current_lock,
            vec![fact(stale.clone(), true)],
        );
        assert_eq!(plan.actions[0].kind, RetentionRootActionKind::CreateRoot);
        assert_eq!(plan.actions[1].kind, RetentionRootActionKind::RemoveRoot);
        assert_eq!(plan.actions[1].record.root_id, stale.root_id);
        assert!(plan.diagnostics.iter().any(|diag| diag.kind == RetentionDiagnosticKind::StaleRoot));
    }

    // r[verify project_workflows.input_retention_roots]
    #[test]
    fn untracked_input_is_gc_eligible_and_does_not_retain_root() {
        let root = record(HASH_A, GENERATION_ONE);
        let plan = plan(InputRetentionPolicy::Untracked, lock_with(HASH_A), vec![fact(root.clone(), true)]);
        assert_eq!(plan.statuses[0].state, RetentionInputState::GcEligible);
        assert!(plan.statuses[0].gc_eligible);
        assert_eq!(plan.actions[0].kind, RetentionRootActionKind::RemoveRoot);
        assert!(plan.retained_records.is_empty());
    }

    // r[verify project_workflows.input_retention_roots]
    #[test]
    fn lock_digest_binding_changes_when_hash_changes() {
        let first = lock_with(HASH_A);
        let second = lock_with(HASH_B);
        let first_digest = lock_entry_digest("pkg", first.inputs.get("pkg").unwrap());
        let second_digest = lock_entry_digest("pkg", second.inputs.get("pkg").unwrap());
        assert_ne!(first_digest, second_digest);
        assert_eq!(first_digest.len(), second_digest.len());
    }

    // r[verify project_workflows.input_retention_roots]
    // r[verify project_workflows.input_retention_atomicity]
    #[test]
    fn invalid_generation_limit_and_interrupted_root_are_not_durable() {
        let mut interrupted = record(HASH_A, GENERATION_ONE);
        interrupted.committed = false;
        let plan = plan(InputRetentionPolicy::RecentGenerations { generations: 0 }, lock_with(HASH_A), vec![fact(
            interrupted,
            true,
        )]);
        assert_eq!(plan.statuses[0].state, RetentionInputState::MissingRoot);
        assert!(plan.diagnostics.iter().any(|diag| diag.kind == RetentionDiagnosticKind::InvalidPolicy));
        assert!(plan.diagnostics.iter().any(|diag| diag.kind == RetentionDiagnosticKind::InterruptedUpdate));
    }

    // r[verify project_workflows.input_retention_roots]
    #[test]
    fn root_for_unknown_input_is_rejected() {
        let mut unknown = record(HASH_A, GENERATION_ONE);
        unknown.input_name = "ghost".into();
        let plan = plan(InputRetentionPolicy::Current, lock_with(HASH_A), vec![fact(unknown.clone(), true)]);
        assert!(plan.diagnostics.iter().any(|diag| diag.kind == RetentionDiagnosticKind::RootForUnknownInput));
        assert!(plan.actions.iter().any(|action| action.record.input_name == "ghost"));
    }

    // r[verify project_workflows.input_retention_roots]
    #[test]
    fn mismatched_lock_digest_does_not_satisfy_current_policy() {
        let mut bad = record(HASH_A, GENERATION_ONE);
        bad.lock_digest = "wrong-lock-digest".into();
        let plan = plan(InputRetentionPolicy::Current, lock_with(HASH_A), vec![fact(bad, true)]);
        assert_eq!(plan.statuses[0].state, RetentionInputState::MissingRoot);
        assert!(plan.diagnostics.iter().any(|diag| diag.kind == RetentionDiagnosticKind::StaleRoot));
        assert!(plan.actions.iter().any(|action| action.kind == RetentionRootActionKind::CreateRoot));
    }

    // r[verify project_workflows.input_retention_atomicity]
    #[test]
    fn missing_root_marker_is_reported_as_absent() {
        let current = record(HASH_A, GENERATION_ONE);
        let plan = plan(InputRetentionPolicy::Current, lock_with(HASH_A), vec![fact(current, false)]);
        assert_eq!(plan.statuses[0].state, RetentionInputState::MissingRoot);
        assert!(plan.diagnostics.iter().any(|diag| diag.kind == RetentionDiagnosticKind::MissingRoot));
        assert!(plan.diagnostics.iter().any(|diag| diag.message.contains("missing root")));
    }
}
