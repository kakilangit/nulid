//! Integration tests for the Id derive macro.

use nulid::{Id, Nulid};
use std::str::FromStr;

#[derive(Id)]
struct UserId(Nulid);

#[derive(Id)]
struct OrderId(pub Nulid);

#[derive(Id)]
struct ProductId(Nulid);

#[test]
fn test_try_from_str() {
    let nulid = Nulid::new().unwrap();
    let s = nulid.to_string();

    let user_id = UserId::try_from(s.as_str()).unwrap();
    assert_eq!(Nulid::from(user_id), nulid);
}

#[test]
fn test_try_from_string() {
    let nulid = Nulid::new().unwrap();
    let s = nulid.to_string();

    let user_id = UserId::try_from(s).unwrap();
    assert_eq!(Nulid::from(user_id), nulid);
}

#[test]
fn test_from_str_trait() {
    let nulid = Nulid::new().unwrap();
    let s = nulid.to_string();

    let user_id: UserId = s.parse().unwrap();
    assert_eq!(Nulid::from(user_id), nulid);
}

#[test]
fn test_from_nulid() {
    let nulid = Nulid::new().unwrap();
    let user_id = UserId::from(nulid);

    assert_eq!(Nulid::from(user_id), nulid);
}

#[test]
fn test_into_nulid() {
    let nulid = Nulid::new().unwrap();
    let user_id = UserId::from(nulid);
    let back: Nulid = user_id.into();

    assert_eq!(back, nulid);
}

#[test]
fn test_as_ref_nulid() {
    let nulid = Nulid::new().unwrap();
    let user_id = UserId::from(nulid);
    let nulid_ref: &Nulid = user_id.as_ref();

    assert_eq!(nulid_ref, &nulid);
}

#[test]
fn test_display() {
    let nulid = Nulid::new().unwrap();
    let user_id = UserId::from(nulid);

    assert_eq!(user_id.to_string(), nulid.to_string());
}

#[test]
fn test_round_trip() {
    let original = Nulid::new().unwrap();
    let user_id = UserId::from(original);
    let s = user_id.to_string();
    let parsed: UserId = s.parse().unwrap();
    let result: Nulid = parsed.into();

    assert_eq!(result, original);
}

#[test]
fn test_invalid_string() {
    let result = UserId::try_from("invalid-nulid");
    assert!(result.is_err());
}

#[test]
fn test_invalid_length() {
    let result = UserId::try_from("SHORT");
    assert!(result.is_err());
}

#[test]
fn test_empty_string() {
    let result = UserId::try_from("");
    assert!(result.is_err());
}

#[test]
fn test_multiple_wrapper_types() {
    let nulid1 = Nulid::new().unwrap();
    let nulid2 = Nulid::new().unwrap();

    let user_id = UserId::from(nulid1);
    let order_id = OrderId::from(nulid2);

    // They should have different values
    assert_ne!(Nulid::from(user_id), Nulid::from(order_id));
}

#[test]
fn test_public_field_wrapper() {
    let nulid = Nulid::new().unwrap();
    let order_id = OrderId(nulid);

    // Can access public field
    assert_eq!(order_id.0, nulid);

    // All traits still work
    let s = order_id.to_string();
    let parsed: OrderId = s.parse().unwrap();
    assert_eq!(parsed, order_id);
}

#[test]
fn test_hash_consistency() {
    use std::collections::HashSet;

    let nulid = Nulid::new().unwrap();
    let user_id1 = UserId::from(nulid);
    let user_id2 = UserId::from(nulid);

    let mut set = HashSet::new();
    set.insert(user_id1);

    // Same value should be found in set
    assert!(set.contains(&user_id2));
}

#[test]
fn test_equality() {
    let nulid = Nulid::new().unwrap();
    let user_id1 = UserId::from(nulid);
    let user_id2 = UserId::from(nulid);

    assert_eq!(user_id1, user_id2);
}

#[test]
fn test_clone() {
    let nulid = Nulid::new().unwrap();
    let user_id1 = UserId::from(nulid);
    let user_id2 = user_id1;

    assert_eq!(user_id1, user_id2);
}

#[test]
fn test_specific_nulid_value() {
    let nulid_str = "01ARZ3NDEKTSV4RRFFQ69G5FAV";
    let nulid = Nulid::from_str(nulid_str).unwrap();

    let user_id = UserId::try_from(nulid_str).unwrap();
    assert_eq!(Nulid::from(user_id), nulid);
    assert_eq!(user_id.to_string(), nulid_str);
}

#[test]
fn test_min_max_values() {
    let min_id = UserId::from(Nulid::MIN);
    let max_id = UserId::from(Nulid::MAX);

    assert_eq!(Nulid::from(min_id), Nulid::MIN);
    assert_eq!(Nulid::from(max_id), Nulid::MAX);
}

#[test]
fn test_zero_value() {
    let zero_id = UserId::from(Nulid::ZERO);
    assert_eq!(Nulid::from(zero_id), Nulid::ZERO);
}

#[test]
fn test_ordering() {
    let nulid1 = Nulid::new().unwrap();
    std::thread::sleep(std::time::Duration::from_millis(1));
    let nulid2 = Nulid::new().unwrap();

    let user_id1 = UserId::from(nulid1);
    let user_id2 = UserId::from(nulid2);

    assert!(user_id1 < user_id2);
    assert!(user_id2 > user_id1);
    assert!(user_id1 <= user_id2);
    assert!(user_id2 >= user_id1);
}

#[test]
fn test_partial_ord() {
    let nulid = Nulid::new().unwrap();
    let user_id1 = UserId::from(nulid);
    let user_id2 = UserId::from(nulid);

    assert_eq!(
        user_id1.partial_cmp(&user_id2),
        Some(std::cmp::Ordering::Equal)
    );
}

#[test]
fn test_ord() {
    let nulid = Nulid::new().unwrap();
    let user_id1 = UserId::from(nulid);
    let user_id2 = UserId::from(nulid);

    assert_eq!(user_id1.cmp(&user_id2), std::cmp::Ordering::Equal);
}

#[test]
fn test_ordering_with_vec() {
    let mut ids = vec![
        UserId::from(Nulid::new().unwrap()),
        UserId::from(Nulid::new().unwrap()),
        UserId::from(Nulid::new().unwrap()),
    ];

    let sorted = ids.clone();
    ids.reverse();
    ids.sort();

    // Should be sorted by timestamp (creation order)
    assert_eq!(ids, sorted);
}

#[test]
fn test_debug_trait() {
    let nulid = Nulid::new().unwrap();
    let user_id = UserId::from(nulid);

    let debug_str = format!("{user_id:?}");
    assert!(debug_str.contains("UserId"));
}

#[test]
fn test_copy_trait() {
    let nulid = Nulid::new().unwrap();
    let user_id1 = UserId::from(nulid);
    let user_id2 = user_id1; // Copy

    // Both should be usable
    assert_eq!(user_id1, user_id2);
    assert_eq!(Nulid::from(user_id1), nulid);
    assert_eq!(Nulid::from(user_id2), nulid);
}

#[test]
fn test_min_max_ordering() {
    let min_id = UserId::from(Nulid::MIN);
    let max_id = UserId::from(Nulid::MAX);
    let middle_id = UserId::from(Nulid::new().unwrap());

    assert!(min_id < middle_id);
    assert!(middle_id < max_id);
    assert!(min_id < max_id);
}

#[test]
fn test_partial_eq_with_nulid() {
    let nulid = Nulid::new().unwrap();
    let user_id = UserId::from(nulid);

    // Can compare wrapper with Nulid directly
    assert_eq!(user_id, nulid);
    assert!(user_id == nulid);

    let different_nulid = Nulid::new().unwrap();
    assert_ne!(user_id, different_nulid);
}

#[test]
fn test_partial_ord_with_nulid() {
    let nulid1 = Nulid::new().unwrap();
    std::thread::sleep(std::time::Duration::from_millis(1));
    let nulid2 = Nulid::new().unwrap();

    let user_id = UserId::from(nulid1);

    // Can compare wrapper with Nulid directly
    assert!(user_id < nulid2);
    assert!(user_id <= nulid1);
    assert!(user_id >= nulid1);

    let user_id2 = UserId::from(nulid2);
    assert!(user_id < nulid2);
    assert!(user_id2 > nulid1);
}

#[test]
fn test_comparison_with_nulid_constants() {
    let min_id = UserId::from(Nulid::MIN);
    let max_id = UserId::from(Nulid::MAX);

    assert_eq!(min_id, Nulid::MIN);
    assert_eq!(max_id, Nulid::MAX);
    assert!(min_id < Nulid::MAX);
    assert!(max_id > Nulid::MIN);
}

#[test]
fn test_new() {
    let user_id = UserId::new().unwrap();
    let nulid: Nulid = user_id.into();

    // Should be a valid, non-zero NULID
    assert_ne!(nulid, Nulid::ZERO);
    assert!(nulid.nanos() > 0);
}

#[test]
fn test_new_creates_different_ids() {
    let user_id1 = UserId::new().unwrap();
    let user_id2 = UserId::new().unwrap();

    // Each call to new() should create a different ID
    assert_ne!(user_id1, user_id2);
}

#[test]
fn test_default() {
    let user_id = UserId::default();
    let nulid: Nulid = user_id.into();

    // Default should be ZERO
    assert_eq!(nulid, Nulid::ZERO);
    assert_eq!(user_id, Nulid::ZERO);
}

#[test]
fn test_default_trait() {
    // Test that Default is properly implemented
    let user_id: UserId = Default::default();
    assert_eq!(user_id, UserId::from(Nulid::ZERO));
}

#[test]
fn test_new_vs_default() {
    let new_id = UserId::new().unwrap();
    let default_id = UserId::default();

    // new() should create a fresh ID, default() should be ZERO
    assert_ne!(new_id, default_id);
    assert_eq!(default_id, Nulid::ZERO);
    assert_ne!(new_id, Nulid::ZERO);
}

#[test]
fn test_multiple_types_new() {
    let user_id = UserId::new().unwrap();
    let order_id = OrderId::new().unwrap();
    let product_id = ProductId::new().unwrap();

    // All should be valid, non-zero IDs
    assert_ne!(Nulid::from(user_id), Nulid::ZERO);
    assert_ne!(Nulid::from(order_id), Nulid::ZERO);
    assert_ne!(Nulid::from(product_id), Nulid::ZERO);
}

#[test]
fn test_multiple_types_default() {
    let user_id = UserId::default();
    let order_id = OrderId::default();
    let product_id = ProductId::default();

    // All should be ZERO
    assert_eq!(Nulid::from(user_id), Nulid::ZERO);
    assert_eq!(Nulid::from(order_id), Nulid::ZERO);
    assert_eq!(Nulid::from(product_id), Nulid::ZERO);
}

#[test]
fn test_deref_nanos() {
    let nulid = Nulid::new().unwrap();
    let user_id = UserId::from(nulid);

    // Can call nanos() directly on UserId via Deref
    assert_eq!(user_id.nanos(), nulid.nanos());
}

#[test]
fn test_deref_micros() {
    let nulid = Nulid::new().unwrap();
    let user_id = UserId::from(nulid);

    // Can call micros() directly on UserId via Deref
    assert_eq!(user_id.micros(), nulid.micros());
}

#[test]
fn test_deref_millis() {
    let nulid = Nulid::new().unwrap();
    let user_id = UserId::from(nulid);

    // Can call millis() directly on UserId via Deref
    assert_eq!(user_id.millis(), nulid.millis());
}

#[test]
fn test_deref_random() {
    let nulid = Nulid::new().unwrap();
    let user_id = UserId::from(nulid);

    // Can call random() directly on UserId via Deref
    assert_eq!(user_id.random(), nulid.random());
}

#[test]
fn test_deref_parts() {
    let nulid = Nulid::new().unwrap();
    let user_id = UserId::from(nulid);

    // Can call parts() directly on UserId via Deref
    assert_eq!(user_id.parts(), nulid.parts());
}

#[test]
fn test_deref_as_u128() {
    let nulid = Nulid::new().unwrap();
    let user_id = UserId::from(nulid);

    // Can call as_u128() directly on UserId via Deref
    assert_eq!(user_id.as_u128(), nulid.as_u128());
}

#[test]
fn test_deref_to_bytes() {
    let nulid = Nulid::new().unwrap();
    let user_id = UserId::from(nulid);

    // Can call to_bytes() directly on UserId via Deref
    assert_eq!(user_id.to_bytes(), nulid.to_bytes());
}

#[test]
fn test_deref_is_nil() {
    let zero_id = UserId::default();
    let normal_id = UserId::new().unwrap();

    // Can call is_nil() directly on UserId via Deref
    assert!(zero_id.is_nil());
    assert!(!normal_id.is_nil());
}

#[test]
fn test_deref_seconds_and_subsec_nanos() {
    let nulid = Nulid::new().unwrap();
    let user_id = UserId::from(nulid);

    // Can call seconds() and subsec_nanos() directly on UserId via Deref
    assert_eq!(user_id.seconds(), nulid.seconds());
    assert_eq!(user_id.subsec_nanos(), nulid.subsec_nanos());
}

#[test]
fn test_deref_multiple_types() {
    let user_id = UserId::new().unwrap();
    let order_id = OrderId::new().unwrap();
    let product_id = ProductId::new().unwrap();

    // All wrapper types can access Nulid methods via Deref
    assert!(user_id.nanos() > 0);
    assert!(order_id.nanos() > 0);
    assert!(product_id.nanos() > 0);

    assert!(user_id.random() > 0);
    assert!(order_id.random() > 0);
    assert!(product_id.random() > 0);
}

#[test]
fn test_deref_coercion() {
    let user_id = UserId::new().unwrap();

    // Deref coercion allows UserId to be used where &Nulid is expected
    fn takes_nulid_ref(nulid: &Nulid) -> u128 {
        nulid.as_u128()
    }

    let result = takes_nulid_ref(&user_id);
    assert_eq!(result, user_id.as_u128());
}

#[test]
fn test_deref_all_timestamp_methods() {
    let user_id = UserId::new().unwrap();

    // Access all timestamp-related methods directly
    let nanos = user_id.nanos();
    let micros = user_id.micros();
    let millis = user_id.millis();
    let seconds = user_id.seconds();
    let subsec = user_id.subsec_nanos();

    // Verify consistency
    assert_eq!(nanos / 1_000, micros);
    assert_eq!(nanos / 1_000_000, millis);
    assert_eq!(nanos / 1_000_000_000, u128::from(seconds));
    assert_eq!(nanos % 1_000_000_000, u128::from(subsec));
}

#[test]
fn test_from_bytes() {
    let bytes = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];
    let user_id = UserId::from_bytes(bytes);
    let nulid = Nulid::from_bytes(bytes);

    assert_eq!(Nulid::from(user_id), nulid);
    assert_eq!(user_id.to_bytes(), bytes);
}

#[test]
fn test_from_u128() {
    let value = 0x0123_4567_89AB_CDEF_FEDC_BA98_7654_3210u128;
    let user_id = UserId::from_u128(value);
    let nulid = Nulid::from_u128(value);

    assert_eq!(Nulid::from(user_id), nulid);
    assert_eq!(user_id.as_u128(), value);
}

#[test]
fn test_from_nanos() {
    let timestamp = 1_000_000_000_000u128;
    let random = 12345u64;

    let user_id = UserId::from_nanos(timestamp, random);
    let nulid = Nulid::from_nanos(timestamp, random);

    assert_eq!(Nulid::from(user_id), nulid);
    assert_eq!(user_id.nanos(), nulid.nanos());
    assert_eq!(user_id.random(), nulid.random());
}

#[test]
fn test_nil() {
    let user_id = UserId::nil();
    let nulid = Nulid::nil();

    assert_eq!(Nulid::from(user_id), nulid);
    assert!(user_id.is_nil());
    assert_eq!(user_id.as_u128(), 0);
}

#[test]
fn test_from_datetime() {
    use std::time::SystemTime;

    let time = SystemTime::now();
    let user_id = UserId::from_datetime(time).unwrap();

    // Verify it's a valid, non-nil ID
    assert!(!user_id.is_nil());
    assert!(user_id.nanos() > 0);
}

#[test]
fn test_now() {
    let user_id = UserId::now().unwrap();

    // Should create a valid, non-nil ID
    assert!(!user_id.is_nil());
    assert!(user_id.nanos() > 0);
}

#[test]
fn test_increment_via_deref() {
    // increment() is an instance method, available via Deref
    let user_id = UserId::from_nanos(1000, 100);
    let next = user_id.increment().unwrap();

    // Same timestamp, incremented random
    assert_eq!(user_id.nanos(), next.nanos());
    assert_eq!(next.random(), 101);
}

#[test]
fn test_all_constructors_create_valid_ids() {
    // Test that all constructor methods create valid, usable IDs
    let id1 = UserId::new().unwrap();
    let id2 = UserId::nil();
    let id3 = UserId::from_bytes([0; 16]);
    let id4 = UserId::from_u128(12345);
    let id5 = UserId::from_nanos(1000, 500);
    let id6 = UserId::from_datetime(std::time::SystemTime::now()).unwrap();
    let id7 = UserId::now().unwrap();

    // All should be valid UserId instances
    assert!(!id1.is_nil());
    assert!(id2.is_nil());
    assert!(id3.is_nil());
    assert_eq!(id4.as_u128(), 12345);
    assert_eq!(id5.nanos(), 1000);
    assert!(!id6.is_nil());
    assert!(!id7.is_nil());
}

#[test]
fn test_from_u128_trait() {
    let value = 0x0123_4567_89AB_CDEF_FEDC_BA98_7654_3210u128;
    let user_id = UserId::from(value);

    assert_eq!(user_id.as_u128(), value);
}

#[test]
fn test_into_u128_trait() {
    let user_id = UserId::from_u128(12345);
    let value: u128 = user_id.into();

    assert_eq!(value, 12345);
}

#[test]
fn test_from_bytes_array_trait() {
    let bytes = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];
    let user_id = UserId::from(bytes);

    assert_eq!(user_id.to_bytes(), bytes);
}

#[test]
fn test_into_bytes_array_trait() {
    let bytes = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];
    let user_id = UserId::from_bytes(bytes);
    let result: [u8; 16] = user_id.into();

    assert_eq!(result, bytes);
}

#[test]
fn test_as_ref_u128() {
    let user_id = UserId::from_u128(12345);
    let value_ref: &u128 = user_id.as_ref();

    assert_eq!(*value_ref, 12345);
}

#[test]
fn test_try_from_byte_slice() {
    let bytes: &[u8] = &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];
    let user_id = UserId::try_from(bytes).unwrap();

    assert_eq!(user_id.to_bytes(), bytes);
}

#[test]
fn test_try_from_byte_slice_invalid_length() {
    let bytes: &[u8] = &[1, 2, 3, 4, 5];
    let result = UserId::try_from(bytes);

    assert!(result.is_err());
}

#[test]
fn test_all_trait_conversions() {
    // Test round-trip conversions through all trait implementations
    let original = UserId::new().unwrap();

    // Via u128
    let as_u128: u128 = original.into();
    let from_u128 = UserId::from(as_u128);
    assert_eq!(original, from_u128);

    // Via bytes
    let as_bytes: [u8; 16] = original.into();
    let from_bytes = UserId::from(as_bytes);
    assert_eq!(original, from_bytes);

    // Via Nulid
    let as_nulid: Nulid = original.into();
    let from_nulid = UserId::from(as_nulid);
    assert_eq!(original, from_nulid);

    // Via byte slice
    let byte_slice = original.to_bytes();
    let from_slice = UserId::try_from(&byte_slice[..]).unwrap();
    assert_eq!(original, from_slice);
}
