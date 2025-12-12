//! Example demonstrating UUID conversion with NULID.
//!
//! Run with:
//! ```
//! cargo run --example uuid_conversion --features uuid
//! ```

fn main() -> nulid::Result<()> {
    use nulid::Nulid;
    use uuid::Uuid;

    println!("NULID ↔ UUID Conversion Examples\n");
    println!("═══════════════════════════════════════════════════════════");

    // Example 1: Create NULID and convert to UUID
    println!("\n1. NULID → UUID");
    println!("───────────────────────────────────────────────────────────");
    let nulid = Nulid::new()?;
    let uuid = nulid.to_uuid();

    println!("NULID:     {nulid}");
    println!("UUID:      {uuid}");
    println!("Hex (u128): 0x{:032X}", nulid.as_u128());
    println!("Same bits:  {}", nulid.as_u128() == uuid.as_u128());

    // Example 2: Create UUID and convert to NULID
    println!("\n2. UUID → NULID");
    println!("───────────────────────────────────────────────────────────");
    let uuid_v4 = Uuid::new_v4();
    let nulid_from_uuid = Nulid::from_uuid(uuid_v4);

    println!("UUID:      {uuid_v4}");
    println!("NULID:     {nulid_from_uuid}");
    println!("Hex (u128): 0x{:032X}", uuid_v4.as_u128());

    // Example 3: Using From/Into traits
    println!("\n3. Using From/Into Traits");
    println!("───────────────────────────────────────────────────────────");
    let nulid2 = Nulid::new()?;

    // Convert using Into
    let uuid2: Uuid = nulid2.into();
    println!("Into:      {nulid2} → {uuid2}");

    // Convert using From
    let nulid3 = Nulid::from(uuid2);
    println!("From:      {uuid2} → {nulid3}");

    // Example 4: Round-trip conversion
    println!("\n4. Round-Trip Conversion");
    println!("───────────────────────────────────────────────────────────");
    let original_nulid = Nulid::new()?;
    let as_uuid = original_nulid.to_uuid();
    let back_to_nulid = Nulid::from_uuid(as_uuid);

    println!("Original:    {original_nulid}");
    println!("As UUID:     {as_uuid}");
    println!("Back:        {back_to_nulid}");
    println!("Preserved:   {}", original_nulid == back_to_nulid);

    // Example 5: Use case - database interop
    println!("\n5. Database Interoperability Example");
    println!("───────────────────────────────────────────────────────────");
    println!("Generate NULID for application use:");
    let record_id = Nulid::new()?;
    println!("  NULID:   {record_id}");
    let timestamp = record_id.nanos();
    let random = record_id.random();
    println!("  Parts:   timestamp={timestamp}, random={random}");

    println!("\nStore as UUID in PostgreSQL/MySQL:");
    let db_uuid = record_id.to_uuid();
    println!("  UUID:    {db_uuid}");
    let bytes = db_uuid.as_bytes();
    println!("  Bytes:   {bytes:?}");

    println!("\nRetrieve from database and convert back:");
    let retrieved = Nulid::from_uuid(db_uuid);
    println!("  NULID:   {retrieved}");
    println!("  Match:   {}", record_id == retrieved);

    // Example 6: Nanosecond precision preserved
    println!("\n6. Nanosecond Precision Comparison");
    println!("───────────────────────────────────────────────────────────");
    let nulid_precise = Nulid::new()?;
    let timestamp_nanos = nulid_precise.nanos();
    let secs = nulid_precise.seconds();
    let subsec = nulid_precise.subsec_nanos();

    println!("NULID timestamp: {timestamp_nanos} nanoseconds");
    println!("  Seconds:       {secs}");
    println!("  Sub-second:    {subsec} ns");

    let uuid_precise = nulid_precise.to_uuid();
    let nulid_recovered = Nulid::from_uuid(uuid_precise);
    println!("\nAfter UUID round-trip:");
    println!(
        "  Preserved:     {}",
        nulid_precise.nanos() == nulid_recovered.nanos()
    );
    println!("  Full equality: {}", nulid_precise == nulid_recovered);

    println!("\n═══════════════════════════════════════════════════════════");
    println!("✅ All conversions preserve the full 128-bit value!");

    Ok(())
}

#[cfg(not(feature = "uuid"))]
fn main() {
    eprintln!("This example requires the 'uuid' feature.");
    eprintln!("Run with: cargo run --example uuid_conversion --features uuid");
    std::process::exit(1);
}
