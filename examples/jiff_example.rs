//! Example demonstrating jiff Timestamp integration with NULID.
//!
//! Run with: cargo run --example jiff_example --features jiff

#![allow(
    clippy::expect_used,
    clippy::similar_names,
    clippy::items_after_statements,
    clippy::too_many_lines,
    clippy::doc_markdown,
    clippy::uninlined_format_args
)]

use jiff::Timestamp;
use nulid::Nulid;

fn main() -> nulid::Result<()> {
    println!("=== NULID Jiff Timestamp Example ===\n");

    let id = Nulid::new()?;
    println!("Generated NULID: {}", id);

    let ts: Timestamp = id.jiff_timestamp()?;
    println!("As Timestamp:    {}", ts);
    println!();

    println!("=== Timestamp Components ===");
    println!("Timestamp (nanos):  {}", id.nanos());
    println!("Timestamp (micros): {}", id.micros());
    println!("Timestamp (millis): {}", id.millis());
    println!("Timestamp (seconds):{}", ts.as_second());
    println!();

    println!("=== Nanosecond Precision ===");
    let timestamp_nanos = 1_704_067_200_123_456_789u128;
    let precise_id = Nulid::from_nanos(timestamp_nanos, 12345);
    let precise_ts = precise_id.jiff_timestamp()?;

    println!("NULID timestamp:    {} nanos", precise_id.nanos());
    println!("Timestamp:          {}", precise_ts);
    println!("Timestamp seconds:  {}", precise_ts.as_second());
    println!(
        "Full precision:     {}.{:09}",
        precise_ts.as_second(),
        precise_ts.subsec_nanosecond()
    );
    println!();

    println!("=== Chronological Ordering ===");
    let mut ids = Vec::new();
    for i in 0..5 {
        let id = Nulid::new()?;
        ids.push(id);
        std::thread::sleep(std::time::Duration::from_millis(10));
        let ts = id.jiff_timestamp()?;
        println!("ID {}: {} -> {}", i + 1, id, ts);
    }

    println!("\n=== Sorting Verification ===");
    let mut sorted_ids = ids.clone();
    sorted_ids.sort();

    println!("IDs are chronologically sorted: {}", ids == sorted_ids);

    println!("\n=== Working with Specific Dates ===");
    let epoch = Nulid::from_nanos(0, 0);
    let epoch_ts = epoch.jiff_timestamp()?;
    println!("Unix Epoch NULID: {} -> {}", epoch, epoch_ts);

    let y2k_nanos = 946_684_800_000_000_000u128;
    let y2k = Nulid::from_nanos(y2k_nanos, 0);
    let y2k_ts = y2k.jiff_timestamp()?;
    println!("Y2K NULID:        {} -> {}", y2k, y2k_ts);

    println!("\n=== Creating NULID from Timestamp ===");
    let ts = Timestamp::from_second(1_704_067_200).expect("valid timestamp");
    let from_ts = Nulid::from_jiff_timestamp(ts)?;
    println!("Timestamp:        {}", ts);
    println!("Created NULID:    {}", from_ts);
    let roundtrip_ts = from_ts.jiff_timestamp()?;
    println!("Roundtrip check:  {}", roundtrip_ts);

    println!("\n=== Comparison with SystemTime ===");
    let now_id = Nulid::now()?;
    let system_time = now_id.datetime();
    let jiff_time = now_id.jiff_timestamp()?;

    println!("NULID:        {}", now_id);
    println!("SystemTime:   {:?}", system_time);
    println!("jiff::Timestamp: {}", jiff_time);

    println!("\n=== Time Calculations ===");
    let id1 = Nulid::new()?;
    std::thread::sleep(std::time::Duration::from_millis(100));
    let id2 = Nulid::new()?;

    let ts1 = id1.jiff_timestamp()?;
    let ts2 = id2.jiff_timestamp()?;

    let diff = ts2 - ts1;
    let diff_secs = diff.get_seconds();

    println!("First ID:  {} at {:?}", id1, ts1);
    println!("Second ID: {} at {:?}", id2, ts2);
    println!("Duration:  {} seconds", diff_secs);
    println!();

    Ok(())
}
