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
