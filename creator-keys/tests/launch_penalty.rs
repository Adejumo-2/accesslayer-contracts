//! Integration tests for the launch penalty fee (issue #798).
//!
//! Early flippers who buy at launch and sell within days extract value and
//! create negative price pressure. This feature applies an additional fee
//! to sells within 7 days of key creation to discourage this behaviour.

mod contract_test_env;

use contract_test_env::{
    register_creator_keys, register_test_creator, set_pricing_and_fees, test_env_with_auths,
};
use creator_keys::LAUNCH_PENALTY_WINDOW_LEDGERS;
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    Address, Env,
};

const KEY_PRICE: i128 = 100;

/// Setup a client, register a creator, and configure pricing.
/// Returns the client, the contract id, and the creator address.
///
/// A minimal 1/1 fee split is configured so `compute_sell_proceeds` succeeds
/// (it requires a fee config); the bps are small enough that the fees floor
/// to 0 on the 100-unit price, so sell proceeds stay at 100 and the launch
/// penalty is fully observable.
fn setup(
    env: &Env,
) -> (
    creator_keys::CreatorKeysContractClient<'_>,
    Address,
    Address,
) {
    let (client, contract_id) = register_creator_keys(env);
    set_pricing_and_fees(env, &client, KEY_PRICE, 1, 1);
    let creator = register_test_creator(env, &client, "alice");
    (client, contract_id, creator)
}

/// Advance the ledger sequence by one so a sell is not rejected by the
/// same-ledger flash-loan guard while remaining inside the launch window.
fn advance_one_ledger(env: &Env) {
    env.ledger().with_mut(|l| l.sequence_number += 1);
}

/// Advance the ledger sequence by `n` steps (each step ~5 seconds).
fn advance_ledgers(env: &Env, n: u32) {
    let mut ledger = env.ledger().get();
    ledger.sequence_number += n;
    ledger.timestamp += (n as u64) * 5;
    env.ledger().set(ledger);
}

// ============================================================================
// Sell within 7 days incurs the launch penalty
// ============================================================================
#[test]
fn test_sell_within_launch_window_applies_penalty() {
    let env = test_env_with_auths();
    let (client, _contract_id, creator) = setup(&env);

    let buyer = Address::generate(&env);
    client.buy_key(&creator, &buyer, &KEY_PRICE, &None);
    assert_eq!(client.get_key_balance(&creator, &buyer), 1);

    // Sell within the launch window. Advance one ledger so the sell is not
    // blocked by the same-ledger flash-loan guard; the 7-day window is
    // 120,960 ledgers, so one ledger stays well inside it.
    advance_one_ledger(&env);
    client.sell_key(&creator, &buyer, &None);

    // The penalty (default 500 bps = 5% of the 100-unit proceeds) is credited
    // to the staking rewards pool in full.
    assert_eq!(client.get_staking_rewards_pool(&creator), 5);
}

// ============================================================================
// Sell after 7 days proceeds with no launch penalty
// ============================================================================
#[test]
fn test_sell_after_launch_window_no_penalty() {
    let env = test_env_with_auths();
    let (client, contract_id, creator) = setup(&env);

    let buyer = Address::generate(&env);
    client.buy_key(&creator, &buyer, &KEY_PRICE, &None);

    // The launch window is measured in ledgers, so escaping it requires
    // advancing 120,961 ledgers. Jumping that far in the test env archives
    // the contract instance unless its TTL is extended first, so bump the
    // instance + code TTL past the jump target before advancing.
    env.deployer()
        .extend_ttl(contract_id, LAUNCH_PENALTY_WINDOW_LEDGERS + 1, 200_000);

    // Advance past the 7-day window (120,960 ledgers).
    advance_ledgers(&env, 120_961);

    // Sell after the window — no penalty.
    client.sell_key(&creator, &buyer, &None);

    // The staking rewards pool must be untouched: no launch penalty applies.
    assert_eq!(client.get_staking_rewards_pool(&creator), 0);
}

// ============================================================================
// set_launch_penalty configures custom penalty bps
// ============================================================================
#[test]
fn test_set_launch_penalty_custom_bps() {
    let env = test_env_with_auths();
    let (client, _contract_id, creator) = setup(&env);

    // Set a custom 1000 bps (10%) penalty.
    client.set_launch_penalty(&creator, &1_000);
    assert_eq!(client.get_launch_penalty_bps(&creator), Some(1_000));

    let buyer = Address::generate(&env);
    client.buy_key(&creator, &buyer, &KEY_PRICE, &None);

    // Sell within the launch window (next ledger, past the flash-loan guard).
    advance_one_ledger(&env);
    client.sell_key(&creator, &buyer, &None);

    // The penalty applied is 10% instead of the default 5%: 10% of the
    // 100-unit proceeds is credited to the staking rewards pool.
    assert_eq!(client.get_staking_rewards_pool(&creator), 10);
}

// ============================================================================
// set_launch_penalty above 2000 panics
// ============================================================================
#[test]
#[should_panic(expected = "PenaltyTooHigh")]
fn test_set_launch_penalty_above_max_panics() {
    let env = test_env_with_auths();
    let (client, _contract_id, creator) = setup(&env);

    client.set_launch_penalty(&creator, &2_001);
}

// ============================================================================
// Default penalty bps is 500 (5%)
// ============================================================================
#[test]
fn test_get_launch_penalty_bps_returns_none_by_default() {
    let env = test_env_with_auths();
    let (client, _contract_id, creator) = setup(&env);

    // No custom penalty set — should return None (uses default 500 bps).
    assert_eq!(client.get_launch_penalty_bps(&creator), None);
}

// ============================================================================
// get_created_at_ledger returns the creation ledger
// ============================================================================
#[test]
fn test_get_created_at_ledger_after_buy() {
    let env = test_env_with_auths();
    let (client, _contract_id, creator) = setup(&env);

    // Before any buy, no created_at_ledger.
    assert_eq!(client.get_created_at_ledger(&creator), None);

    let buyer = Address::generate(&env);
    let current_seq = env.ledger().get().sequence_number;
    client.buy_key(&creator, &buyer, &KEY_PRICE, &None);

    // After first buy, created_at_ledger should be set.
    assert_eq!(client.get_created_at_ledger(&creator), Some(current_seq));
}

// ============================================================================
// created_at_ledger is only set on the FIRST buy
// ============================================================================
#[test]
fn test_created_at_ledger_only_set_on_first_buy() {
    let env = test_env_with_auths();
    let (client, _contract_id, creator) = setup(&env);

    let buyer1 = Address::generate(&env);
    let seq1 = env.ledger().get().sequence_number;
    client.buy_key(&creator, &buyer1, &KEY_PRICE, &None);
    assert_eq!(client.get_created_at_ledger(&creator), Some(seq1));

    // Second buy from a different buyer should not change created_at_ledger.
    let buyer2 = Address::generate(&env);
    advance_ledgers(&env, 100);
    client.buy_key(&creator, &buyer2, &KEY_PRICE, &None);
    assert_eq!(client.get_created_at_ledger(&creator), Some(seq1));
}
