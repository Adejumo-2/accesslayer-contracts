//! Unit tests for `get_total_key_supply` scoping per creator (issue #701).
//!
//! Verifies that supply is tracked independently per creator and does not bleed
//! across registrations. Covers the acceptance criteria:
//!   - Supply scoped correctly per creator
//!   - Creator A supply unaffected by creator B buys
//!   - Sell correctly decrements supply
//!   - Unregistered creator returns `NotRegistered` from the checked supply view

mod contract_test_env;

use contract_test_env::{
    register_creator_keys, register_test_creator, set_pricing_and_fees,
    test_env_with_auths,
};
use creator_keys::ContractError;
use soroban_sdk::testutils::Address as _;

/// Buy `count` keys for `buyer` from `creator`, fetching a live quote before each purchase.
fn buy_n_keys(
    client: &creator_keys::CreatorKeysContractClient<'_>,
    creator: &soroban_sdk::Address,
    buyer: &soroban_sdk::Address,
    count: u32,
) {
    for _ in 0..count {
        let quote = client.get_buy_quote(creator);
        client.buy_key(creator, buyer, &quote.total_amount, &None);
    }
}

#[test]
fn test_supply_scoped_per_creator_after_ten_buys() {
    let env = test_env_with_auths();
    let (client, _) = register_creator_keys(&env);
    set_pricing_and_fees(&env, &client, 1_000_i128, 9_000, 1_000);

    let creator_a = register_test_creator(&env, &client, "alice");
    let buyer_a = soroban_sdk::Address::generate(&env);

    buy_n_keys(&client, &creator_a, &buyer_a, 10);

    assert_eq!(client.get_total_key_supply(&creator_a), 10);
}

#[test]
fn test_supply_scoped_per_creator_after_five_buys() {
    let env = test_env_with_auths();
    let (client, _) = register_creator_keys(&env);
    set_pricing_and_fees(&env, &client, 1_000_i128, 9_000, 1_000);

    let creator_b = register_test_creator(&env, &client, "bob");
    let buyer_b = soroban_sdk::Address::generate(&env);

    buy_n_keys(&client, &creator_b, &buyer_b, 5);

    assert_eq!(client.get_total_key_supply(&creator_b), 5);
}

#[test]
fn test_creator_a_supply_unaffected_by_creator_b_buys() {
    let env = test_env_with_auths();
    let (client, _) = register_creator_keys(&env);
    set_pricing_and_fees(&env, &client, 1_000_i128, 9_000, 1_000);

    let creator_a = register_test_creator(&env, &client, "alice");
    let creator_b = register_test_creator(&env, &client, "bob");
    let buyer_a = soroban_sdk::Address::generate(&env);
    let buyer_b = soroban_sdk::Address::generate(&env);

    buy_n_keys(&client, &creator_a, &buyer_a, 10);
    assert_eq!(client.get_total_key_supply(&creator_a), 10);

    buy_n_keys(&client, &creator_b, &buyer_b, 5);
    assert_eq!(client.get_total_key_supply(&creator_b), 5);

    // Creator A supply must still be 10 after creator B bought 5 keys
    assert_eq!(
        client.get_total_key_supply(&creator_a),
        10,
        "creator A supply changed after creator B buys"
    );
}

#[test]
fn test_sell_decrements_creator_a_supply() {
    let env = test_env_with_auths();
    let (client, _) = register_creator_keys(&env);
    set_pricing_and_fees(&env, &client, 1_000_i128, 9_000, 1_000);

    let creator_a = register_test_creator(&env, &client, "alice");
    let buyer_a = soroban_sdk::Address::generate(&env);

    buy_n_keys(&client, &creator_a, &buyer_a, 10);
    assert_eq!(client.get_total_key_supply(&creator_a), 10);

    // Sell 3 keys — supply should drop to 7
    for _ in 0..3 {
        client.sell_key(&creator_a, &buyer_a, &None);
    }

    assert_eq!(client.get_total_key_supply(&creator_a), 7);
}

#[test]
fn test_sell_does_not_affect_other_creator_supply() {
    let env = test_env_with_auths();
    let (client, _) = register_creator_keys(&env);
    set_pricing_and_fees(&env, &client, 1_000_i128, 9_000, 1_000);

    let creator_a = register_test_creator(&env, &client, "alice");
    let creator_b = register_test_creator(&env, &client, "bob");
    let buyer_a = soroban_sdk::Address::generate(&env);
    let buyer_b = soroban_sdk::Address::generate(&env);

    buy_n_keys(&client, &creator_a, &buyer_a, 10);
    buy_n_keys(&client, &creator_b, &buyer_b, 5);

    for _ in 0..3 {
        client.sell_key(&creator_a, &buyer_a, &None);
    }

    assert_eq!(client.get_total_key_supply(&creator_a), 7);
    // Creator B supply must be unchanged
    assert_eq!(
        client.get_total_key_supply(&creator_b),
        5,
        "creator B supply changed after creator A sold keys"
    );
}

#[test]
fn test_unregistered_creator_returns_not_registered_error() {
    let env = test_env_with_auths();
    let (client, _) = register_creator_keys(&env);
    set_pricing_and_fees(&env, &client, 1_000_i128, 9_000, 1_000);

    let unregistered = soroban_sdk::Address::generate(&env);

    let result = client.try_get_creator_supply(&unregistered);
    assert_eq!(result, Err(Ok(ContractError::NotRegistered)));
}
