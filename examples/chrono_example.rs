//! Example demonstrating chrono DateTime integration with NULID.
//!
//! Run with: cargo run --example chrono_example --features chrono

#![allow(
    clippy::expect_used,
    clippy::similar_names,
    clippy::items_after_statements,
    clippy::too_many_lines,
    clippy::doc_markdown,
    clippy::uninlined_format_args
)]

use chrono::{DateTime, Datelike, TimeZone, Timelike, Utc};
use nulid::Nulid;

fn main() -> nulid::Result<()> {
    println!("=== NULID Chrono DateTime Example ===\n");

    // Generate a new NULID
    let id = Nulid::new()?;
    println!("Generated NULID: {}", id);

    // Convert to DateTime<Utc>
    let dt: DateTime<Utc> = id.chrono_datetime();
    println!("As DateTime:     {}", dt);
    println!("ISO 8601:        {}", dt.to_rfc3339());
    println!();

    // Extract timestamp components
    println!("=== Timestamp Components ===");
    println!("Timestamp (nanos):    {}", id.nanos());
    println!("Timestamp (micros):   {}", id.micros());
    println!("Timestamp (millis):   {}", id.millis());
    println!("DateTime year:        {}", dt.year());
    println!("DateTime month:       {}", dt.month());
    println!("DateTime day:         {}", dt.day());
    println!("DateTime hour:        {}", dt.hour());
    println!("DateTime minute:      {}", dt.minute());
    println!("DateTime second:      {}", dt.second());
    println!("DateTime nanosecond:  {}", dt.timestamp_subsec_nanos());
    println!();

    // Demonstrate precision preservation
    println!("=== Nanosecond Precision ===");
    let timestamp_nanos = 1_704_067_200_123_456_789u128; // 2024-01-01 00:00:00.123456789 UTC
    let precise_id = Nulid::from_nanos(timestamp_nanos, 12345);
    let precise_dt = precise_id.chrono_datetime();

    println!("NULID timestamp:      {} nanos", precise_id.nanos());
    println!("DateTime:             {}", precise_dt);
    println!("DateTime timestamp:   {}", precise_dt.timestamp());
    println!(
        "Subsecond nanos:      {}",
        precise_dt.timestamp_subsec_nanos()
    );
    println!(
        "Full precision:       {}.{:09}",
        precise_dt.timestamp(),
        precise_dt.timestamp_subsec_nanos()
    );
    println!();

    // Compare multiple NULIDs
    println!("=== Chronological Ordering ===");
    let mut ids = Vec::new();
    for i in 0..5 {
        let id = Nulid::new()?;
        ids.push(id);
        // Small delay to ensure different timestamps
        std::thread::sleep(std::time::Duration::from_millis(10));
        println!("ID {}: {} -> {}", i + 1, id, id.chrono_datetime());
    }

    // Verify sorting
    println!("\n=== Sorting Verification ===");
    let mut sorted_ids = ids.clone();
    sorted_ids.sort();

    println!("IDs are chronologically sorted: {}", ids == sorted_ids);

    // Working with specific dates
    println!("\n=== Working with Specific Dates ===");
    let epoch = Nulid::from_nanos(0, 0);
    let epoch_dt = epoch.chrono_datetime();
    println!("Unix Epoch NULID: {} -> {}", epoch, epoch_dt);

    let y2k_nanos = 946_684_800_000_000_000u128; // 2000-01-01 00:00:00 UTC
    let y2k = Nulid::from_nanos(y2k_nanos, 0);
    let y2k_dt = y2k.chrono_datetime();
    println!("Y2K NULID:        {} -> {}", y2k, y2k_dt);

    // Create NULID from DateTime
    println!("\n=== Creating NULID from DateTime ===");
    let dt = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();
    let from_dt = Nulid::from_chrono_datetime(dt)?;
    println!("DateTime:         {}", dt);
    println!("Created NULID:    {}", from_dt);
    println!("Roundtrip check:  {}", from_dt.chrono_datetime());

    // Demonstrate with SystemTime comparison
    println!("\n=== Comparison with SystemTime ===");
    let now_id = Nulid::now()?;
    let system_time = now_id.datetime();
    let chrono_time = now_id.chrono_datetime();

    println!("NULID:            {}", now_id);
    println!("SystemTime:       {:?}", system_time);
    println!("DateTime<Utc>:    {}", chrono_time);

    // Calculate time differences
    println!("\n=== Time Calculations ===");
    let id1 = Nulid::new()?;
    std::thread::sleep(std::time::Duration::from_millis(100));
    let id2 = Nulid::new()?;

    let dt1 = id1.chrono_datetime();
    let dt2 = id2.chrono_datetime();
    let duration = dt2.signed_duration_since(dt1);

    println!("First ID:  {} at {}", id1, dt1);
    println!("Second ID: {} at {}", id2, dt2);
    println!("Duration:  {} milliseconds", duration.num_milliseconds());
    println!();

    Ok(())
}
