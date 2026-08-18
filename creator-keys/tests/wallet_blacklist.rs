//! Tests for the admin-managed wallet blacklist.
//!
//! Covers: blacklisted wallets rejected on buy/sell/creator registration,
//! restored access after removal from the blacklist, and admin-only
//! access to the blacklist mutators.

mod contract_test_env;

use contract_test_env::{
    register_creator_keys, register_test_creator, set_key_price_for_tests, test_env_with_auths,
};
use creator_keys::{ContractError, RegisterCreatorParams};
use soroban_sdk::{testutils::Address as _, Address, String};

/// Register a protocol admin into contract storage and return the admin address.
fn set_protocol_admin(
    env: &soroban_sdk::Env,
    client: &creator_keys::CreatorKeysContractClient<'_>,
) -> Address {
    let admin = Address::generate(env);
    client.set_protocol_admin(&admin, &admin);
    admin
}

// ---------------------------------------------------------------------------
// buy reverts with WalletBlacklisted when the buyer is blacklisted
// ---------------------------------------------------------------------------

#[test]
fn test_buy_key_reverts_for_blacklisted_buyer() {
    let env = test_env_with_auths();
    let (client, _) = register_creator_keys(&env);
    let admin = set_protocol_admin(&env, &client);
    set_key_price_for_tests(&env, &client, 100);
    let creator = register_test_creator(&env, &client, "alice");
    let buyer = Address::generate(&env);

    client.blacklist_wallet(&admin, &buyer);

    let result = client.try_buy_key(&creator, &buyer, &100, &None);
    assert_eq!(result, Err(Ok(ContractError::WalletBlacklisted)));
}

#[test]
fn test_buy_key_no_state_change_for_blacklisted_buyer() {
    let env = test_env_with_auths();
    let (client, _) = register_creator_keys(&env);
    let admin = set_protocol_admin(&env, &client);
    set_key_price_for_tests(&env, &client, 100);
    let creator = register_test_creator(&env, &client, "alice");
    let buyer = Address::generate(&env);

    client.blacklist_wallet(&admin, &buyer);

    let supply_before = client.get_total_key_supply(&creator);
    let _ = client.try_buy_key(&creator, &buyer, &100, &None);
    let supply_after = client.get_total_key_supply(&creator);

    assert_eq!(
        supply_before, supply_after,
        "supply must not change when a blacklisted buyer's purchase is rejected"
    );
    assert_eq!(client.get_key_balance(&creator, &buyer), 0);
}
