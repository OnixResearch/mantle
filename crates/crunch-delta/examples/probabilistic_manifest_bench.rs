use std::time::Instant;

use crunch_delta::{bench_suite, build_receiver_manifest_probabilistic, plan_transfer};

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let suite = bench_suite();
    let started_at = Instant::now();
    let mut manifest_probes = 0u64;
    let mut wire_bytes = 0u64;
    let mut full_bytes = 0u64;
    let filter_bytes = suite.probabilistic_filter_bytes_total();
    let summary_wire_bytes = suite.probabilistic_summary_wire_bytes_total();

    for case in &suite.cases {
        let outcome = build_receiver_manifest_probabilistic(case)
            .await
            .expect("probabilistic manifest build must succeed");
        let plan = plan_transfer(&case.sender, &outcome.manifest).expect("plan must succeed");
        assert_eq!(
            plan.transferred_bytes,
            case.expected_coarse_bytes,
            "case {} wire bytes regressed",
            case.name
        );
        println!(
            "case={} manifest_probes={} wire_bytes={} full_bytes={} target_bytes={}",
            case.name,
            outcome.probes.total_probes(),
            plan.transferred_bytes,
            case.expected_full_bytes,
            case.expected_coarse_bytes,
        );
        manifest_probes = manifest_probes.saturating_add(outcome.probes.total_probes());
        wire_bytes = wire_bytes.saturating_add(plan.transferred_bytes);
        full_bytes = full_bytes.saturating_add(plan.full_transfer_bytes);
    }

    let elapsed_ms = started_at.elapsed().as_millis().min(u128::from(u64::MAX)) as u64;
    let ratio_ppm = if full_bytes == 0 {
        0
    } else {
        wire_bytes.saturating_mul(1_000_000) / full_bytes
    };

    println!("METRIC summary_wire_bytes={summary_wire_bytes}");
    println!("METRIC filter_bytes={filter_bytes}");
    println!("METRIC manifest_probes={manifest_probes}");
    println!("METRIC wire_bytes={wire_bytes}");
    println!("METRIC wall_ms={elapsed_ms}");
    println!("METRIC ratio_ppm={ratio_ppm}");
}
