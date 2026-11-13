//! Synthetic supplied facts only: no command in this fixture is launched, and
//! acceptance here is not execution, producer authentication, or proof authority.
use super::*;
use crate::source_built_fixed_point_action_trust::ActionExecutable;
use crate::source_built_fixed_point_action_trust::ActionInput;
use crate::source_built_fixed_point_action_trust::ActionSpec;
use crate::source_built_fixed_point_action_trust::StageActionAdapter;
use crate::source_built_fixed_point_action_trust::reconcile_actions;
use crate::source_built_fixed_point_planning::InitialOutputAuthorityState;
use crate::source_built_fixed_point_planning::SourceBuiltFixedPointPlanInput;
use crate::source_built_fixed_point_planning::plan_actions;
use crate::source_built_fixed_point_planning::plan_source_built_fixed_point;
use crate::source_built_fixed_point_receipt::ColdProviderObservation;
use crate::source_built_fixed_point_receipt::ColdStageObservation;
use crate::source_built_fixed_point_receipt::write_cold_source_built_fixed_point_receipt;

pub(super) struct SyntheticReceiptFixture {
    pub(super) root: tempfile::TempDir,
    pub(super) proof: SourceBuiltFixedPointPlan,
    pub(super) actions: ActionTrustPlan,
    pub(super) events: Vec<ActionExecutionEvent>,
}

fn write_bytes(root: &Path, relative: &str, bytes: &[u8]) {
    let path = root.join(relative);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, bytes).unwrap();
}

pub(super) fn write_json(root: &Path, relative: &str, value: &impl Serialize) {
    write_bytes(root, relative, &serde_json::to_vec(value).unwrap());
}

fn journal_path(index: usize, stage: &str) -> String {
    match index {
        4 => "cargo-free-fixed-point/stage1/recipe/execution.jsonl".into(),
        5 => "cargo-free-fixed-point/stage2/recipe/execution.jsonl".into(),
        _ => format!("synthetic/{stage}/recipe/execution.jsonl"),
    }
}

fn output_path(index: usize, action: &ActionSpec, output: &str) -> String {
    recipe_output_path(&RecipeOutputBinding {
        journal_path: &journal_path(index, &action.stage_id),
        stage_id: &action.stage_id,
        action_id: &action.id,
        output_id: output,
    })
    .unwrap()
}

fn supplied_authority_digest(
    proof: &SourceBuiltFixedPointPlan,
    authority: crate::source_built_fixed_point::StageAuthorityInput,
) -> String {
    use crate::source_built_fixed_point::ProofPolicyRole;
    use crate::source_built_fixed_point::StageAuthorityInput;
    match authority {
        StageAuthorityInput::Source { role } => {
            proof.source_inputs.iter().find(|source| source.role == role).unwrap().digest_blake3.clone()
        }
        StageAuthorityInput::Policy { role } => match role {
            ProofPolicyRole::Closure => &proof.policies.closure_policy_digest_blake3,
            ProofPolicyRole::Hermeticity => &proof.policies.hermeticity_policy_digest_blake3,
            ProofPolicyRole::ProtectedExecution => &proof.policies.protected_execution_policy_digest_blake3,
            ProofPolicyRole::Effect => &proof.policies.effect_policy_digest_blake3,
            ProofPolicyRole::Normalization => &proof.policies.normalization_policy_digest_blake3,
        }
        .clone(),
        StageAuthorityInput::Output { .. } => unreachable!("outputs require a measured producer"),
    }
}

fn dependencies(action: &mut ActionSpec) {
    let mut producers = action
        .inputs
        .iter()
        .filter_map(|input| match input {
            ActionInput::Produced { action_id, .. } => Some(action_id.clone()),
            _ => None,
        })
        .collect::<BTreeSet<_>>();
    if let ActionExecutable::Produced { action_id, .. } = &action.executable {
        producers.insert(action_id.clone());
    }
    action.producer_actions = producers.into_iter().collect();
    action.output_ids.sort();
}

impl SyntheticReceiptFixture {
    pub(super) fn new() -> Self {
        let root = tempfile::tempdir().unwrap();
        let (mut template, mut adapters, _) = crate::source_built_fixed_point_action_trust::tests::fixture();
        Self::write_sources(&root, &mut template);
        Self::write_products(&root, &mut template, &mut adapters);
        template.resource_bounds.disk_bytes_max = 1_000_000;
        template.resource_bounds.protected_exec_events_max = 256;
        let proof = plan_source_built_fixed_point(SourceBuiltFixedPointPlanInput {
            proof_id: "synthetic-receipt-not-an-executed-proof".into(),
            logical_store_prefix: template.logical_store_prefix,
            source_inputs: template.source_inputs,
            initial_output_authority: InitialOutputAuthorityState {
                stagex_transition_entries: 0,
                native_provider_entries: 0,
                rust_provider_entries: 0,
                mantle_output_entries: 0,
            },
            policies: template.policies,
            resource_bounds: template.resource_bounds,
        })
        .unwrap();
        Self::write_native_publication(&root, &proof, &mut adapters);
        Self::write_mantle_actions(&root, &mut adapters);
        for adapter in &mut adapters {
            adapter.reachable_action_ids = adapter.actions.iter().map(|action| action.id.clone()).collect();
            adapter.reachable_action_ids.sort();
        }
        let actions = plan_actions(&proof, adapters).unwrap();
        let events = Self::measure_events(&root, &proof, &actions);
        let fixture = Self {
            root,
            proof,
            actions,
            events,
        };
        fixture.seal_actions();
        fixture.write_recipe_journals();
        fixture.write_manifests();
        Self::write_receipt(&fixture);
        fixture
    }

    pub(super) fn seal_actions(&self) {
        write_json(self.root.path(), PROOF_PLAN_FILE, &self.proof);
        write_json(self.root.path(), ACTION_PLAN_FILE, &self.actions);
        write_json(self.root.path(), ACTION_EVENTS_FILE, &self.events);
        let mut journal = Vec::new();
        for event in &self.events {
            serde_json::to_writer(&mut journal, event).unwrap();
            journal.push(b'\n');
        }
        write_bytes(self.root.path(), ACTION_JOURNAL_FILE, &journal);
        write_json(
            self.root.path(),
            ACTION_RECONCILIATION_FILE,
            &reconcile_actions(&self.proof, &self.actions, &self.events).unwrap(),
        );
    }

    fn write_recipe_journals(&self) {
        let evidence = load_action_evidence(self.root.path(), &self.proof).unwrap();
        for (index, adapter) in self.actions.adapters.iter().enumerate() {
            let ids = stage_action_ids(&evidence, &adapter.stage_id).unwrap();
            let selected = recipe_journal_events(&evidence, &ids);
            let mut records = vec![
                serde_json::json!({"schema":"mantle-source-recipe-execution-v1", "proof":self.proof.plan_digest_blake3, "stage":adapter.stage_id}),
            ];
            for event in &selected.protected {
                records.push(serde_json::json!({"pid":1,"syscall":59,"tracee_path":"synthetic-not-launched","resolved_path":"synthetic-not-launched","digest_blake3":event.executable_digest_blake3,"allowed":true,"reason":"supplied synthetic fact, not a launched process","is_exec":true}));
            }
            for event in &selected.statuses {
                records.push(serde_json::json!({"action":event.action_id,"normal_exit_code":event.normal_exit_code}));
            }
            let mut count = 0;
            for event in &selected.builds {
                records.push(serde_json::json!({"action":event.action_id,"actual_outputs":event.produced_outputs,"disk_bytes":event.disk_bytes,"completed":true}));
                count += event.produced_outputs.len();
            }
            records.push(serde_json::json!({"complete":true,"output_count":count}));
            let mut bytes = Vec::new();
            for record in records {
                serde_json::to_writer(&mut bytes, &record).unwrap();
                bytes.push(b'\n');
            }
            write_bytes(self.root.path(), &journal_path(index, &adapter.stage_id), &bytes);
        }
    }

    pub(super) fn write_manifests(&self) {
        for (index, manifest_path) in [(4, MANTLE_STAGE1_MANIFEST), (5, MANTLE_STAGE2_MANIFEST)] {
            let adapter = &self.actions.adapters[index];
            let binary = &adapter.actions[0];
            let orchestrator = self
                .events
                .iter()
                .find(|event| {
                    event.action_id == adapter.actions[1].id && event.kind == ExecutionEventKind::ProtectedExec
                })
                .unwrap();
            let smoke = &adapter.actions[2];
            let parent = manifest_path.rsplit_once('/').unwrap().0;
            let measured = |output| {
                let path = output_path(index, smoke, output);
                MantleMeasuredFile {
                    path: path.strip_prefix(&format!("{parent}/")).unwrap().into(),
                    digest_blake3: measure_artifact(self.root.path(), &path).unwrap().1,
                }
            };
            let stdout = measured("stdout");
            let stderr = measured("stderr");
            let commands = MantleCommandReceipts {
                schema: MANTLE_COMMAND_RECEIPTS_SCHEMA.into(),
                commands: vec![MantleCommandCompletion {
                    action_id: smoke.id.clone(),
                    exit_code: 0,
                    stdout: stdout.clone(),
                    stderr: stderr.clone(),
                }],
            };
            let command_path = format!("{parent}/commands.json");
            write_json(self.root.path(), &command_path, &commands);
            let path = output_path(index, binary, &binary.output_ids[0]);
            let manifest = MantleStageManifest {
                schema: MANTLE_STAGE_MANIFEST_SCHEMA.into(),
                stage_id: adapter.stage_id.clone(),
                proof_plan_digest_blake3: self.proof.plan_digest_blake3.clone(),
                action_plan_digest_blake3: self.actions.digest_blake3.clone(),
                closure_policy_digest_blake3: self.proof.policies.closure_policy_digest_blake3.clone(),
                binary: MantleStageBinary {
                    path: path.strip_prefix(&format!("{parent}/")).unwrap().into(),
                    digest_blake3: measure_artifact(self.root.path(), &path).unwrap().1,
                    producer_action_id: binary.id.clone(),
                    output_id: binary.output_ids[0].clone(),
                },
                orchestrator: MantleStageOrchestrator {
                    action_id: orchestrator.action_id.clone(),
                    executable_digest_blake3: orchestrator.executable_digest_blake3.clone(),
                    executable_producer: orchestrator.executable_producer.clone(),
                },
                smoke: MantleStageSmoke {
                    action_id: smoke.id.clone(),
                    exit_code: 0,
                    stdout,
                    stderr: stderr.clone(),
                },
                command_receipt: MantleMeasuredFile {
                    path: "commands.json".into(),
                    digest_blake3: hash_file(&self.root.path().join(command_path)).unwrap(),
                },
                stderr,
                semantic_unit_count: 1,
            };
            write_json(self.root.path(), manifest_path, &manifest);
        }
    }

    pub(super) fn receipt(&self) -> (DeterministicBuildProofReceipt, SourceBuiltReceiptExtension) {
        let (value, _) = read_bound_json::<serde_json::Value>(self.root.path(), FINAL_RECEIPT_FILE).unwrap();
        (
            serde_json::from_value(value.clone()).unwrap(),
            serde_json::from_value(value[EXTENSION_FIELD].clone()).unwrap(),
        )
    }

    // Recompute authentic canonical framing after a deliberately supplied change;
    // rejection must be semantic, not merely a stale outer checksum.
    pub(super) fn reseal_receipt(
        &self,
        mut core: DeterministicBuildProofReceipt,
        mut extension: SourceBuiltReceiptExtension,
    ) {
        use crunch_release_core::content_bound_rebuild_descriptor_digest_blake3;
        use crunch_release_core::rebuild_arguments_digest_blake3;
        use crunch_release_core::rebuild_authority_plan_digest_blake3;
        extension.action_evidence = load_action_evidence(self.root.path(), &self.proof).unwrap().binding;
        extension.stage_evidence_digest_blake3 = hash_file(&self.root.path().join(STAGE_EVIDENCE_FILE)).unwrap();
        extension.final_proof_bundle_digest_blake3 = proof_bundle_digest(self.root.path()).unwrap().digest_blake3;
        let descriptor = core.rebuild_descriptor.as_mut().unwrap();
        descriptor.ordered_arguments =
            receipt_arguments(&self.proof, &self.proof.policies.closure_policy_digest_blake3, &extension).unwrap();
        descriptor.arguments_blake3 = rebuild_arguments_digest_blake3(descriptor.ordered_arguments.clone()).unwrap();
        let descriptor_digest = content_bound_rebuild_descriptor_digest_blake3(descriptor.clone()).unwrap();
        core.rebuild_descriptor_blake3 = Some(descriptor_digest.clone());
        let authority = core.rebuild_authority_plan.as_mut().unwrap();
        authority.descriptor_blake3 = descriptor_digest.clone();
        let authority_digest = rebuild_authority_plan_digest_blake3(authority.clone()).unwrap();
        core.rebuild_authority_plan_blake3 = Some(authority_digest.clone());
        for run in &mut core.runs {
            run.rebuild_descriptor_blake3 = Some(descriptor_digest.clone());
            run.rebuild_authority_plan_blake3 = Some(authority_digest.clone());
        }
        core.receipt_blake3 = None;
        core.receipt_blake3 = Some(deterministic_build_proof_receipt_digest_blake3(core.clone()).unwrap());
        let mut value = serde_json::to_value(core).unwrap();
        value
            .as_object_mut()
            .unwrap()
            .insert(EXTENSION_FIELD.into(), serde_json::to_value(extension).unwrap());
        write_json(self.root.path(), FINAL_RECEIPT_FILE, &value);
    }

    fn write_sources(root: &tempfile::TempDir, template: &mut SourceBuiltFixedPointPlan) {
        for source in &mut template.source_inputs {
            let path = format!("synthetic-sources/{}", source.id);
            if source.kind == crate::source_built_fixed_point::SourceContentKind::Directory {
                write_bytes(root.path(), &format!("{path}/content"), source.id.as_bytes());
            } else {
                write_bytes(root.path(), &path, source.id.as_bytes());
            }
            let (size, digest) = measure_artifact(root.path(), &path).unwrap();
            source.size_bytes = size;
            source.digest_blake3 = digest;
        }
    }

    fn write_products(
        root: &tempfile::TempDir,
        template: &mut SourceBuiltFixedPointPlan,
        adapters: &mut [StageActionAdapter],
    ) {
        for (index, adapter) in adapters.iter_mut().enumerate() {
            let action = &mut adapter.actions[0];
            action.disk_bytes_max = 1_000_000;
            let path = output_path(index, action, &action.output_ids[0]);
            if index == 2 || index == 3 {
                write_bytes(root.path(), &format!("{path}/content"), b"synthetic provider tree\n");
            } else {
                write_bytes(root.path(), &path, b"synthetic equal Mantle bytes; never executed\n");
            }
            if index == 2 {
                action.output_ids.push("native-tool".into());
                write_bytes(root.path(), &output_path(index, action, "native-tool"), b"synthetic native tool\n");
                template.policies.expected_native_provider_digest_blake3 =
                    measure_artifact(root.path(), &path).unwrap().1;
            }
            if index == 3 {
                action.output_ids.extend(["rustc".into(), "toolchain-closure".into()]);
                let rustc = output_path(index, action, "rustc");
                write_bytes(root.path(), &rustc, b"\x7fELFsynthetic header; never executed\n");
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    fs::set_permissions(root.path().join(rustc), fs::Permissions::from_mode(0o755)).unwrap();
                }
                write_bytes(
                    root.path(),
                    &output_path(index, action, "toolchain-closure"),
                    b"synthetic source-produced closure text\n",
                );
            }
            dependencies(action);
        }
    }

    fn write_native_publication(
        root: &tempfile::TempDir,
        proof: &SourceBuiltFixedPointPlan,
        adapters: &mut [StageActionAdapter],
    ) {
        let native = adapters[2].actions[0].clone();
        let mut publication = native.clone();
        publication.id = format!("{}/publication", native.stage_id);
        publication.inputs = native
            .output_ids
            .iter()
            .map(|output_id| ActionInput::Produced {
                action_id: native.id.clone(),
                output_id: output_id.clone(),
            })
            .collect();
        publication.output_ids = vec!["native-provider-publication".into()];
        dependencies(&mut publication);
        let mut artifacts = serde_json::Map::new();
        for role in NATIVE_EXECUTABLE_ROLES.iter().chain(NATIVE_TREE_ROLES).chain(NATIVE_RUNTIME_ROLES) {
            let tree = NATIVE_TREE_ROLES.contains(role);
            let output = if tree {
                "full-source-native-provider"
            } else {
                "native-tool"
            };
            let path = output_path(2, &native, output);
            let (size, digest) = measure_artifact(root.path(), &path).unwrap();
            artifacts.insert(
                (*role).into(),
                serde_json::json!({
                    "producer": {"kind":"produced", "action_id":native.id, "output_id":output},
                    "path":path, "digest_blake3":digest, "size_bytes":size,
                    "kind":if tree {"tree"} else {"executable"},
                }),
            );
        }
        let source = proof
            .source_inputs
            .iter()
            .find(|source| source.role == crate::source_built_fixed_point::SourceAuthorityRole::NativeSourceBundle)
            .unwrap();
        write_json(
            root.path(),
            &output_path(2, &publication, "native-provider-publication"),
            &serde_json::json!({
                "schema":"mantle-source-native-provider-publication-v1",
                "proof_plan_digest_blake3":proof.plan_digest_blake3,
                "native_source_manifest_digest_blake3":source.digest_blake3,
                "expected_provider_digest_blake3":proof.policies.expected_native_provider_digest_blake3,
                "artifacts":artifacts,
            }),
        );
        adapters[2].actions.push(publication);
    }

    fn write_mantle_actions(root: &tempfile::TempDir, adapters: &mut [StageActionAdapter]) {
        for index in [4, 5] {
            let binary = adapters[index].actions[0].clone();
            let mut orchestrator = binary.clone();
            orchestrator.id = format!("{}/orchestrator", binary.stage_id);
            orchestrator.output_ids = vec!["orchestration".into()];
            write_bytes(root.path(), &output_path(index, &orchestrator, "orchestration"), b"synthetic orchestration\n");
            let mut smoke = binary.clone();
            smoke.id = format!("{}/smoke", binary.stage_id);
            smoke.executable = ActionExecutable::Produced {
                action_id: binary.id.clone(),
                output_id: binary.output_ids[0].clone(),
            };
            smoke.inputs = vec![ActionInput::Produced {
                action_id: binary.id.clone(),
                output_id: binary.output_ids[0].clone(),
            }];
            smoke.output_ids = vec!["stderr".into(), "stdout".into()];
            smoke.accepted_exit_codes = vec![0];
            dependencies(&mut smoke);
            write_bytes(
                root.path(),
                &output_path(index, &smoke, "stdout"),
                b"synthetic smoke output; never executed\n",
            );
            write_bytes(root.path(), &output_path(index, &smoke, "stderr"), b"");
            adapters[index].actions.extend([orchestrator, smoke]);
        }
    }

    fn measure_events(
        root: &tempfile::TempDir,
        proof: &SourceBuiltFixedPointPlan,
        actions: &ActionTrustPlan,
    ) -> Vec<ActionExecutionEvent> {
        let mut events = Vec::new();
        let mut produced = std::collections::BTreeMap::<(String, String), String>::new();
        for (index, adapter) in actions.adapters.iter().enumerate() {
            for action in &adapter.actions {
                let (executable_digest_blake3, executable_producer) = match &action.executable {
                    ActionExecutable::Fixed { digest_blake3 } => (digest_blake3.clone(), None),
                    ActionExecutable::Produced { action_id, output_id } => (
                        produced[&(action_id.clone(), output_id.clone())].clone(),
                        Some((action_id.clone(), output_id.clone())),
                    ),
                };
                let input_digests_blake3 = action
                    .inputs
                    .iter()
                    .map(|input| match input {
                        ActionInput::Stage { authority } => supplied_authority_digest(proof, *authority),
                        ActionInput::Produced { action_id, output_id } => {
                            produced[&(action_id.clone(), output_id.clone())].clone()
                        }
                        _ => unreachable!(),
                    })
                    .collect();
                let mut event = event_fixture();
                event.action_id = action.id.clone();
                event.inputs = action.inputs.clone();
                event.input_digests_blake3 = input_digests_blake3;
                event.executable_digest_blake3 = executable_digest_blake3;
                event.executable_producer = executable_producer;
                event.disk_bytes = 0;
                events.push(event.clone());
                event.kind = ExecutionEventKind::BuildExecution;
                event.normal_exit_code = (!action.accepted_exit_codes.is_empty()).then_some(0);
                for output in &action.output_ids {
                    let (size, digest) = measure_artifact(root.path(), &output_path(index, action, output)).unwrap();
                    event.disk_bytes += size;
                    event.produced_outputs.push((output.clone(), digest.clone()));
                    produced.insert((action.id.clone(), output.clone()), digest);
                }
                events.push(event);
            }
        }
        events
    }

    fn write_receipt(fixture: &Self) {
        let stages = fixture
            .actions
            .adapters
            .iter()
            .enumerate()
            .map(|(index, adapter)| {
                let canonical = &adapter.actions[0];
                ColdStageObservation {
                    stage_id: adapter.stage_id.clone(),
                    output_path: fixture.root.path().join(output_path(index, canonical, &canonical.output_ids[0])),
                    producer_action_id: canonical.id.clone(),
                    output_id: canonical.output_ids[0].clone(),
                    execution_journal_path: fixture.root.path().join(journal_path(index, &adapter.stage_id)),
                    native_publication_audit_path: (index == 2).then(|| {
                        fixture.root.path().join(output_path(index, &adapter.actions[1], "native-provider-publication"))
                    }),
                }
            })
            .collect();
        let rust = &fixture.actions.adapters[3].actions[0];
        let providers = ColdProviderObservation {
            stages,
            native_provider_path: fixture.root.path().join(output_path(
                2,
                &fixture.actions.adapters[2].actions[0],
                "full-source-native-provider",
            )),
            rust_provider_path: fixture.root.path().join(output_path(3, rust, "full-source-rust-provider")),
            rustc_path: fixture.root.path().join(output_path(3, rust, "rustc")),
            toolchain_closure_path: fixture.root.path().join(output_path(3, rust, "toolchain-closure")),
        };
        write_cold_source_built_fixed_point_receipt(fixture.root.path(), &fixture.proof, &providers).unwrap();
    }
}
