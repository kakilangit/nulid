//! Regression test for `#[derive(Id)]` in a downstream crate.
#![deny(unexpected_cfgs)]

use nulid::{Id, Nulid};

/// A wrapper as downstream crates write it.
#[derive(Id)]
pub struct OrderId(Nulid);

#[cfg(test)]
mod tests {
    use crate::OrderId;
    use nulid::Nulid;

    #[test]
    fn serde_impls_follow_nulids_features() {
        let id = OrderId::from(Nulid::from_nanos(1_700_000_000_000_000_000, 42));
        let json = serde_json::to_string(&id).unwrap();
        assert_eq!(json, serde_json::to_string(&Nulid::from(id)).unwrap());
        let back: OrderId = serde_json::from_str(&json).unwrap();
        assert_eq!(back, id);
    }

    #[test]
    fn uuid_impls_follow_nulids_features() {
        let id = OrderId::from(Nulid::from_nanos(1_700_000_000_000_000_000, 42));
        assert_eq!(OrderId::from_uuid(id.to_uuid()), id);
    }
}
