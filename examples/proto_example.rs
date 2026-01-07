use nulid::Nulid;
use nulid::proto::nulid::Nulid as ProtoNulid;
use prost::Message;

fn main() -> nulid::Result<()> {
    println!("=== NULID Protobuf Example ===\n");

    let nulid = Nulid::new()?;
    println!("Generated NULID: {nulid}");
    println!("NULID (hex): 0x{:032x}", nulid.as_u128());
    println!("NULID (bytes): {:02x?}", nulid.to_bytes());

    println!();

    let proto = nulid.to_proto();
    println!("Protobuf representation:");
    println!("  High (bits 64-127): 0x{:016x}", proto.high);
    println!("  Low (bits 0-63):   0x{:016x}", proto.low);

    println!();

    let encoded = proto.encode_to_vec();
    println!("Encoded protobuf bytes ({} bytes):", encoded.len());
    println!("{encoded:02x?}");

    println!();

    #[allow(clippy::expect_used)]
    let decoded = ProtoNulid::decode(&*encoded).expect("Failed to decode protobuf");
    println!("Decoded protobuf:");
    println!("  High: 0x{:016x}", decoded.high);
    println!("  Low:  0x{:016x}", decoded.low);

    println!();

    let nulid2 = Nulid::from_proto(decoded);
    println!("Reconstructed NULID: {nulid2}");
    println!("NULID (hex): 0x{:032x}", nulid2.as_u128());

    println!();

    assert_eq!(nulid, nulid2);
    println!("✓ Round-trip successful: NULID values match");

    println!();
    println!("=== Using From trait ===");

    let nulid3 = Nulid::new()?;
    let proto2: ProtoNulid = nulid3.into();
    let nulid4: Nulid = proto2.into();

    assert_eq!(nulid3, nulid4);
    println!("✓ From trait conversion successful");

    println!();
    println!("=== Nil NULID ===");
    let nil = Nulid::nil();
    let nil_proto = nil.to_proto();
    println!(
        "Nil NULID protobuf: high={}, low={}",
        nil_proto.high, nil_proto.low
    );
    assert_eq!(nil_proto.high, 0);
    assert_eq!(nil_proto.low, 0);
    println!("✓ Nil NULID has zero high and low bits");

    println!();
    println!("=== Max NULID ===");
    let max = Nulid::max();
    let max_proto = max.to_proto();
    println!(
        "Max NULID protobuf: high=0x{:016x}, low=0x{:016x}",
        max_proto.high, max_proto.low
    );
    assert_eq!(max_proto.high, u64::MAX);
    assert_eq!(max_proto.low, u64::MAX);
    println!("✓ Max NULID has all bits set");

    println!();
    println!("=== Bit preservation test ===");
    let test_value = 0x0123_4567_89AB_CDEF_FEDC_BA98_7654_3210_u128;
    let test_nulid = Nulid::from_u128(test_value);
    let test_proto = test_nulid.to_proto();
    println!("Test NULID: 0x{test_value:032x}");
    println!("  High bits: 0x{:016x}", test_proto.high);
    println!("  Low bits:  0x{:016x}", test_proto.low);
    assert_eq!(test_proto.high, 0x0123_4567_89AB_CDEF);
    assert_eq!(test_proto.low, 0xFEDC_BA98_7654_3210);
    println!("✓ All 128 bits preserved correctly");

    println!();
    println!("=== Encoding efficiency ===");
    println!("Binary NULID (to_bytes): 16 bytes");
    println!("Protobuf encoding: {} bytes", encoded.len());
    println!("Protobuf uses 2× uint64 fields (no overhead for fixed-size values)");

    Ok(())
}
