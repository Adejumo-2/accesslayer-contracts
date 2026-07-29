//! Integration test for sell event emitted with correct fields on successful sale (#581).

mod contract_test_env;

use contract_test_env::{register_creator_keys, set_pricing_and_fees, test_env_with_auths};
use creator_keys::events;
use soroban_sdk::{
    testutils::{Address as _, Events, Ledger},
    Address, IntoVal, String, Symbol,
};

const KEY_PRICE: i128 = 1000;
const CREATOR_BPS: u32 = 9000;
const PROTOCOL_BPS: u32 = 1000;

fn compute_expected_proceeds(price: i128, protocol_bps: u32) -> i128 {
    let protocol_fee = (price * protocol_bps as i128) / 10_000;
    let creator_fee = price - protocol_fee;
    price - creator_fee - protocol_fee
}

#[test]
fn test_sell_event_emitted_with_correct_fields() {
    let env = test_env_with_auths();
    let (client, _) = register_creator_keys(&env);

    let admin = set_pricing_and_fees(&env, &client, KEY_PRICE, CREATOR_BPS, PROTOCOL_BPS);
    let protocol_recipient = Address::generate(&env);
    client.set_protocol_fee_recipient(&admin, &protocol_recipient);

    let creator = Address::generate(&env);
    client.register_creator(
        &creator_keys::RegisterCreatorParams {
            creator: creator.clone(),
            handle: String::from_str(&env, "alice"),
        },
        &None,
        &None,
        &None,
        &None,
        &None,
        &None,
    );

    let holder = Address::generate(&env);

    // Buy 2 keys so holder has balance >= 1 before selling
    let quote1 = client.get_buy_quote(&creator);
    client.buy_key(&creator, &holder, &quote1.total_amount, &None);
    let quote2 = client.get_buy_quote(&creator);
    client.buy_key(&creator, &holder, &quote2.total_amount, &None);

    assert_eq!(client.get_total_key_supply(&creator), 2);
    assert_eq!(client.get_key_balance(&creator, &holder), 2);

    // Set a known ledger sequence
    let test_ledger = 42u32;
    let mut ledger_info = env.ledger().get();
    ledger_info.sequence_number = test_ledger;
    env.ledger().set(ledger_info);

    // Clear event log before the sell
    env.events().all();

    // Sell 1 key
    client.sell_key(&creator, &holder, &None);

    // Extract the sell event
    let event_log = env.events().all();
    let (_, topics, data) = event_log
        .iter()
        .find(|(_, topics, _)| {
            topics
                .get(events::TOPIC_EVENT_NAME_INDEX)
                .map(|v| {
                    let name: Symbol = v.into_val(&env);
                    name == events::SELL_EVENT_NAME
                })
                .unwrap_or(false)
        })
        .expect("exactly one sell event must be emitted");

    // Assert exactly one sell event
    let sell_event_count = event_log
        .iter()
        .filter(|(_, t, _)| {
            t.get(events::TOPIC_EVENT_NAME_INDEX)
                .map(|v| {
                    let name: Symbol = v.into_val(&env);
                    name == events::SELL_EVENT_NAME
                })
                .unwrap_or(false)
        })
        .count();
    assert_eq!(
        sell_event_count, 1,
        "exactly one sell event per transaction"
    );

    // Assert all five fields
    let payload: events::KeysSoldEvent = data.into_val(&env);

    assert_eq!(payload.seller, holder, "seller must match caller");
    assert_eq!(payload.creator_id, creator, "creator_id must match");
    assert_eq!(payload.quantity, 1u32, "quantity must be 1");
    assert_eq!(
        payload.ledger, test_ledger,
        "ledger must match transaction ledger"
    );

    // proceeds = price - creator_fee - protocol_fee
    let sell_price = KEY_PRICE;
    let expected_proceeds = compute_expected_proceeds(sell_price, PROTOCOL_BPS);
    assert_eq!(
        payload.proceeds, expected_proceeds,
        "proceeds must equal sell price minus creator and protocol fees"
    );
    assert!(
        payload.proceeds < sell_price,
        "proceeds must be less than raw sell price"
    );

    // Verify seller is also in topics
    let topic_seller: Address = topics
        .get(events::TOPIC_BUYER_INDEX)
        .expect("seller topic must be present")
        .into_val(&env);
    assert_eq!(topic_seller, holder);
}
