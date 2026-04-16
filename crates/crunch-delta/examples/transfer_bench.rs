use std::time::Instant;

use crunch_delta::{bench_suite, plan_transfer};

fn main() {
    let suite = bench_suite();
    let started_at = Instant::now();
    let mut transferred_bytes = 0u64;
    let mut full_bytes = 0u64;

    for case in &suite.cases {
        let plan = plan_transfer(&case.sender, &case.receiver).expect("benchmark cases must plan");
        println!(
            "case={} transfer_bytes={} full_bytes={} target_bytes={}",
            case.name, plan.transferred_bytes, case.expected_full_bytes, case.expected_coarse_bytes,
        );
        transferred_bytes = transferred_bytes.saturating_add(plan.transferred_bytes);
        full_bytes = full_bytes.saturating_add(plan.full_transfer_bytes);
    }

    let elapsed_ms = started_at.elapsed().as_millis().min(u128::from(u64::MAX)) as u64;
    let ratio_ppm = if full_bytes == 0 {
        0
    } else {
        transferred_bytes.saturating_mul(1_000_000) / full_bytes
    };

    println!("METRIC wire_bytes={transferred_bytes}");
    println!("METRIC wall_ms={elapsed_ms}");
    println!("METRIC ratio_ppm={ratio_ppm}");
}
