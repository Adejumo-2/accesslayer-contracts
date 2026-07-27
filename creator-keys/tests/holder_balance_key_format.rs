//! Unit tests for the `holder_balance_key` helper (Issue #619).
//!
//! `holder_balance_key` builds the composite `DataKey::KeyBalance(creator, holder)`
//! storage key. These tests confirm the encoded key has a consistent format:
//! non-empty, deterministic for a given (creator, holder) pair, equal length
//! across different holders of the same creator, and distinguishable from
//! other `DataKey` variants.

use creator_keys::constants;
use soroban_sdk::{testutils::Address as _, xdr::ToXdr, Address, Env};

#[test]
fn test_holder_balance_key_is_non_empty() {
    let env = Env::default();
    let creator = Address::generate(&env);
    let holder = Address::generate(&env);

    let key = constants::storage::holder_balance_key(&creator, &holder);
    let bytes = key.to_xdr(&env);

    assert!(!bytes.is_empty(), "holder balance key must not be empty");
}

#[test]
fn test_holder_balance_key_is_deterministic_for_same_inputs() {
    let env = Env::default();
    let creator = Address::generate(&env);
    let holder = Address::generate(&env);

    let key_a = constants::storage::holder_balance_key(&creator, &holder);
    let key_b = constants::storage::holder_balance_key(&creator, &holder);

    assert_eq!(
        key_a.to_xdr(&env),
        key_b.to_xdr(&env),
        "same creator/holder pair must always produce the same key encoding"
    );
}

#[test]
fn test_holder_balance_keys_for_different_holders_have_equal_length() {
    let env = Env::default();
    let creator = Address::generate(&env);
    let holder_a = Address::generate(&env);
    let holder_b = Address::generate(&env);

    let bytes_a = constants::storage::holder_balance_key(&creator, &holder_a).to_xdr(&env);
    let bytes_b = constants::storage::holder_balance_key(&creator, &holder_b).to_xdr(&env);

    assert_ne!(
        bytes_a, bytes_b,
        "different holders must produce distinct key encodings"
    );
    assert_eq!(
        bytes_a.len(),
        bytes_b.len(),
        "keys for the same creator with different holders must have equal encoded length"
    );
}

#[test]
fn test_holder_balance_key_distinguishable_from_other_key_types() {
    let env = Env::default();
    let creator = Address::generate(&env);
    let holder = Address::generate(&env);

    let balance_key = constants::storage::holder_balance_key(&creator, &holder);
    let creator_key = constants::storage::creator(&creator);
    let price_key = constants::storage::KEY_PRICE;

    let balance_bytes: std::vec::Vec<u8> = balance_key.to_xdr(&env).iter().collect();
    let creator_bytes: std::vec::Vec<u8> = creator_key.to_xdr(&env).iter().collect();
    let price_bytes: std::vec::Vec<u8> = price_key.to_xdr(&env).iter().collect();

    assert_ne!(
        balance_bytes, creator_bytes,
        "KeyBalance key encoding must differ from a Creator key encoding, \
         even when built from the same creator address"
    );
    assert_ne!(
        balance_bytes, price_bytes,
        "KeyBalance key encoding must differ from the global KeyPrice key encoding"
    );

    // The enum variant name is encoded as a leading Symbol discriminator in the
    // XDR payload. A KeyBalance key must carry that discriminator, and a
    // same-address Creator key must not spuriously contain it.
    let variant_marker = b"KeyBalance";
    let balance_key_has_marker = balance_bytes
        .windows(variant_marker.len())
        .any(|window| window == variant_marker);
    assert!(
        balance_key_has_marker,
        "KeyBalance key encoding should embed its variant discriminator"
    );

    let creator_key_has_balance_marker = creator_bytes
        .windows(variant_marker.len())
        .any(|window| window == variant_marker);
    assert!(
        !creator_key_has_balance_marker,
        "Creator key encoding must not contain the KeyBalance discriminator"
    );
}
