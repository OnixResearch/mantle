use alloc::vec;

use super::*;

fn tool(name: &str, version: &str) -> Tool {
    let root = format!("/nix/store/{}-{name}", "a".repeat(32));
    Tool {
        executable: FileInput {
            name: name.into(),
            path: format!("{root}/bin/{name}"),
            blake3: "a".repeat(64),
        },
        version: version.into(),
        closure: vec![root],
    }
}

fn file(name: &str) -> FileInput {
    FileInput {
        name: name.into(),
        path: format!("/source/{name}"),
        blake3: "b".repeat(64),
    }
}

fn profile() -> Profile {
    Profile {
        schema: SCHEMA.into(),
        part: PART.into(),
        package: PACKAGE.into(),
        rtl: vec![file("top.sv")],
        includes: vec![file("include/config.vh")],
        include_dirs: vec!["include".into()],
        top: "top".into(),
        defines: BTreeMap::new(),
        parameters: BTreeMap::new(),
        synthesis_script: file("synth.ys"),
        synthesis_passes: vec![SynthesisPass::Check],
        flatten: true,
        pins: BTreeMap::from([("clk".into(), "21".into()), ("q".into(), "99".into())]),
        clocks: vec![Clock {
            port: "clk".into(),
            report_net: "clk_global".into(),
            frequency_khz: 10_000,
        }],
        constraints: file("pins.pcf"),
        seed: 1,
        threads: 1,
        yosys: tool("yosys", "0.68"),
        nextpnr: tool("nextpnr-ice40", "0.10"),
        packer: tool("icepack", "0-unstable-2025-06-03"),
        shell: tool("sh", "1.37.0"),
        bwrap: tool("bwrap", "0.11.0"),
        database: EmbeddedDatabase {
            part: PART.into(),
            package: PACKAGE.into(),
            nextpnr_binary_blake3: "a".repeat(64),
        },
        utilization_limits: BTreeMap::from([("ICESTORM_LC".into(), 100)]),
        timeout_ms: 120_000,
        max_log_bytes: 65_536,
        max_output_bytes: MAX_ARTIFACT_BYTES,
    }
}

fn report() -> Report {
    Report {
        utilization: BTreeMap::from([("ICESTORM_LC".into(), UtilizationFact {
            used: 3,
            available: 1280,
        })]),
        fmax: BTreeMap::from([("clk_global".into(), FrequencyFact {
            achieved: 20.0,
            constraint: 10.0,
        })]),
        critical_paths: vec![],
        detailed_net_timings: vec![],
    }
}

#[test]
fn stage_merkle_boundaries_track_only_affected_inputs() {
    let baseline = profile();
    let original = identities(baseline.clone()).unwrap().stages;
    let mut changed = baseline.clone();
    changed.rtl[0].blake3 = "c".repeat(64);
    let rtl = identities(changed).unwrap().stages;
    assert_ne!(original.synthesis, rtl.synthesis);
    assert_ne!(original.place_route, rtl.place_route);
    assert_ne!(original.packing, rtl.packing);
    changed = baseline.clone();
    changed.seed = 2;
    let seed = identities(changed).unwrap().stages;
    assert_eq!(original.synthesis, seed.synthesis);
    assert_ne!(original.place_route, seed.place_route);
    assert_ne!(original.packing, seed.packing);
    changed = baseline.clone();
    changed.constraints.blake3 = "c".repeat(64);
    let constraints = identities(changed).unwrap().stages;
    assert_eq!(original.synthesis, constraints.synthesis);
    assert_ne!(original.place_route, constraints.place_route);
    changed = baseline.clone();
    changed.nextpnr.executable.blake3 = "c".repeat(64);
    assert!(matches!(validate(changed.clone()), Err(AdmissionError::EmbeddedDatabase)));
    changed.database.nextpnr_binary_blake3 = changed.nextpnr.executable.blake3.clone();
    let db = identities(changed).unwrap().stages;
    assert_eq!(original.synthesis, db.synthesis);
    assert_ne!(original.place_route, db.place_route);
    changed = baseline.clone();
    changed.packer.executable.blake3 = "c".repeat(64);
    let packer = identities(changed).unwrap().stages;
    assert_eq!(original.synthesis, packer.synthesis);
    assert_eq!(original.place_route, packer.place_route);
    assert_ne!(original.packing, packer.packing);
}

#[test]
fn timing_checks_actual_facts_not_process_success() {
    let profile = profile();
    let mut facts = report();
    accept_report(ReportRequest {
        profile: profile.clone(),
        report: facts.clone(),
    })
    .unwrap();
    facts.fmax.get_mut("clk_global").unwrap().achieved = 9.999;
    assert!(matches!(
        accept_report(ReportRequest {
            profile: profile.clone(),
            report: facts.clone()
        }),
        Err(AdmissionError::TimingViolation { .. })
    ));
    facts.fmax.get_mut("clk_global").unwrap().achieved = f64::NAN;
    assert!(
        accept_report(ReportRequest {
            profile: profile.clone(),
            report: facts.clone()
        })
        .is_err()
    );
    facts = report();
    facts.fmax.get_mut("clk_global").unwrap().constraint = 9.0;
    assert!(matches!(
        accept_report(ReportRequest {
            profile: profile.clone(),
            report: facts.clone()
        }),
        Err(AdmissionError::ClockConstraint(_))
    ));
    facts = report();
    facts.fmax.clear();
    assert!(
        accept_report(ReportRequest {
            profile: profile.clone(),
            report: facts.clone()
        })
        .is_err()
    );
    facts = report();
    facts.fmax.insert("undeclared_clock".into(), FrequencyFact {
        achieved: 100.0,
        constraint: 10.0,
    });
    assert!(accept_report(ReportRequest { profile, report: facts }).is_err());
}

#[test]
fn utilization_limits_and_device_capacity_are_both_enforced() {
    let profile = profile();
    let mut facts = report();
    facts.utilization.get_mut("ICESTORM_LC").unwrap().used = 101;
    assert!(matches!(
        accept_report(ReportRequest {
            profile: profile.clone(),
            report: facts.clone()
        }),
        Err(AdmissionError::UtilizationViolation(_))
    ));
    facts.utilization.get_mut("ICESTORM_LC").unwrap().used = 1281;
    assert!(matches!(
        accept_report(ReportRequest {
            profile: profile.clone(),
            report: facts.clone()
        }),
        Err(AdmissionError::DeviceCapacity(_))
    ));
    facts.utilization.clear();
    assert!(accept_report(ReportRequest { profile, report: facts }).is_err());
}

#[test]
fn netlist_ports_must_exactly_match_required_pins() {
    let mut profile = profile();
    let netlist = serde_json::to_vec(&serde_json::json!({"modules":{"top":{"ports":{
        "clk":{"direction":"input","bits":[2]}, "q":{"direction":"output","bits":[3]}
    }}}}))
    .unwrap();
    profile = accept_netlist(NetlistRequest {
        profile,
        netlist: netlist.clone(),
    })
    .unwrap()
    .profile;
    profile.pins.remove("q");
    assert!(matches!(
        accept_netlist(NetlistRequest {
            profile: profile.clone(),
            netlist: netlist.clone()
        }),
        Err(AdmissionError::UnconstrainedPin)
    ));
    profile.pins.insert("unknown".into(), "99".into());
    assert!(accept_netlist(NetlistRequest { profile, netlist }).is_err());
}

#[test]
fn unsupported_part_missing_constraints_and_ambient_tool_paths_fail_closed() {
    let baseline = profile();
    let mut changed = baseline.clone();
    changed.part = "ice40-up5k".into();
    assert!(validate(changed).is_err());
    changed = baseline.clone();
    changed.pins.clear();
    assert!(validate(changed).is_err());
    changed = baseline.clone();
    changed.clocks.clear();
    assert!(validate(changed).is_err());
    changed = baseline.clone();
    changed.yosys.executable.path = "/usr/bin/yosys".into();
    assert!(validate(changed).is_err());
    changed = baseline;
    changed.threads = 2;
    assert!(validate(changed).is_err());
}

#[test]
fn report_json_facts_preserve_nested_numbers_and_nulls() {
    let mut wire = serde_json::to_value(report()).unwrap();
    wire["critical_paths"] = serde_json::json!([{
        "unsigned": u64::MAX, "signed": i64::MIN,
        "delay": 0.125, "facts": [null, true, "clock"],
    }]);
    wire["detailed_net_timings"] = serde_json::json!([{"delay": -0.5}]);
    let decoded: Report = serde_json::from_value(wire.clone()).unwrap();
    assert_eq!(serde_json::to_value(decoded).unwrap(), wire);
}
