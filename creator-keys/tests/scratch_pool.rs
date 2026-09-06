mod contract_test_env;

use contract_test_env::{
    register_creator_keys, register_test_creator, set_pricing_and_fees, test_env_with_auths,
};
use soroban_sdk::{
    testutils::{Address as _, Ledger as _},
    Address, Env,
};

#[test]
fn scratch_pool_after_buy_and_sell() {
    let env = test_env_with_auths();
    let (client, _) = register_creator_keys(&env);
    set_pricing_and_fees(&env, &client, 100, 5000, 5000);
    let creator = register_test_creator(&env, &client, "alice");

    let buyer = Address::generate(&env);
    client.buy_key(&creator, &buyer, &100, &None);
    println!("pool after buy: {}", client.get_staking_rewards_pool(&creator));

    env.ledger().with_mut(|l| l.sequence_number += 1);
    client.sell_key(&creator, &buyer, &None);
    println!("pool after sell: {}", client.get_staking_rewards_pool(&creator));
}