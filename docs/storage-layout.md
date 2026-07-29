# Contract Storage Layout

Complete reference for every persistent storage key used by the `creator-keys`
contract: key format, stored data type, TTL policy, and which entrypoints read
or write it.

This document is the exhaustive index. For narrower, topic-specific detail see:

- [storage-key-invariants.md](./storage-key-invariants.md) — invariants that must hold across operations (supply/balance conservation, holder count, etc.)
- [quote-storage-keys.md](./quote-storage-keys.md) — storage keys touched by `get_buy_quote` / `get_sell_quote`
- [creator-state-storage-ttl.md](./creator-state-storage-ttl.md) — TTL/rent-bump background for creator profile and holder balance entries
- [key-staking-storage.md](./key-staking-storage.md) — data model for the key-staking feature (`DataKey::StakePosition`, `DataKey::NextStakeId`); **not yet part of the `DataKey` enum below** — staking storage is a design document for work in progress, not shipped storage

All keys are defined as variants of the single `DataKey` enum in
[`creator-keys/src/lib.rs`](../creator-keys/src/lib.rs) and are only ever
written to Soroban **persistent** storage (`env.storage().persistent()`); the
contract does not use `temporary()` or `instance()` storage for any of the
entries below.

## Composite key naming convention

Keys that scope data to one creator, or to one (creator, holder) pair, are
modeled as **enum variants carrying the scoping `Address` values as fields**,
not as string concatenation. For example:

```rust
KeyBalance(Address, Address)   // (creator, holder)
CreatorFeeBalance(Address)     // (creator)
```

Two same-shaped variants with different address arguments always encode as
different keys, and the variant name itself becomes part of the on-chain XDR
encoding (as a leading `Symbol` discriminator), so a `KeyBalance(a, b)` key can
never collide with a `Creator(a)` key even though they share the address `a`.

The convention for building these keys is: never construct a `DataKey`
variant directly at a call site. Always go through the matching helper in
`constants::storage` (e.g. `constants::storage::holder_balance_key(creator,
holder)`, `constants::storage::creator_fee_balance(creator)`). This keeps key
construction centralized so a future change to a key's shape only needs to
update one function. See
[`creator-keys/tests/holder_balance_key_format.rs`](../creator-keys/tests/holder_balance_key_format.rs)
for unit tests asserting this helper's output is deterministic and
distinguishable from other key types.

## Full key reference

| Key | Scope | Value type | Written by | Read by |
| --- | --- | --- | --- | --- |
| `Creator(Address)` | Per-creator | `CreatorProfile` | `register_creator`, `buy_key`/`sell_key`/`buyback` (profile updates), `write_creator_supply` | `read_creator_profile`, `read_registered_creator_profile`, `get_creator`, `get_creator_details`, `get_creators_batch`, quote methods |
| `FeeConfig` | Global | `fee::FeeConfig { creator_bps, protocol_bps }` | `set_fee_config` (admin) | `read_protocol_fee_config`, `get_fee_config`, `get_protocol_fee_view`, `get_creator_fee_config`, buy/sell quote paths |
| `KeyPrice` | Global | `i128` | `set_key_price` (admin) | `buy_key`, `sell_key`, `buyback`, `airdrop_keys`, `get_buy_quote`, `get_sell_quote`, `resolve_quote_inputs` |
| `KeyBalance(Address, Address)` | Per-creator-holder (composite) | `u32` | `buy_key`, `sell_key`, `buyback`, `airdrop_keys`, `transfer_keys`, `claim_locked_allocation` | `get_key_balance`, `get_holder_key_count`, `get_sell_quote`, dividend settlement |
| `TreasuryAddress` | Global | `Address` | `set_treasury_address` (admin) | `get_treasury_address` |
| `AdminAddress` | Global | `Address` | `set_protocol_admin` (admin) | `assert_is_admin`, `get_protocol_admin` |
| `ProtocolFeeRecipient` | Global | `Address` | `set_protocol_fee_recipient`, `update_protocol_fee_recipient` (admin) | `get_protocol_fee_recipient`, `get_protocol_fee_view`, fee accrual paths |
| `ProtocolFeeRecipientBalance` | Global | `i128` | `credit_protocol_fee_recipient_balance` (internal, called from buy/sell/buyback fee accrual) | `read_protocol_fee_recipient_balance`, `get_protocol_recipient_balance` |
| `CreatorFeeBalance(Address)` | Per-creator | `i128` | `credit_creator_fee_recipient_balance` (internal fee accrual) | `read_creator_fee_recipient_balance`, `get_creator_fee_balance` |
| `ProtocolStateVersion` | Global | `u32` | `set_fee_config` (incremented on each config update) | `get_protocol_state_version` |
| `Paused` | Global | `bool` | `pause`, `unpause` (admin) | `is_paused`, `assert_not_paused`, `get_is_paused` |
| `DividendPerKeyAccumulated(Address)` | Per-creator | `i128` | `distribute_dividend` | `read_dividend_accumulator`, `settle_holder_dividends`, `compute_claimable_dividend` |
| `HolderDividendCheckpoint(Address, Address)` | Per-creator-holder (composite) | `i128` | `settle_holder_dividends`, `claim_dividend`, `batch_claim_dividend` | `compute_claimable_dividend` |
| `HolderDividendPending(Address, Address)` | Per-creator-holder (composite) | `i128` | `settle_holder_dividends`, `claim_dividend`, `batch_claim_dividend` | `compute_claimable_dividend`, `get_claimable_dividend` |
| `LockedAllocation(Address)` | Per-creator | `LockedAllocation { amount, unlock_ledger, claimed }` | `register_creator` (initial write), `claim_locked_allocation` (marks claimed) | `get_locked_allocation`, `claim_locked_allocation` |
| `MaxSupply(Address)` | Per-creator | `u32` | `register_creator` (optional, if a cap is supplied) | `buy_key`, `airdrop_keys` (cap enforcement), `get_max_supply` |
| `CurveSlope` | Global | `i128` | `set_curve_slope` (admin) | `read_curve_slope`, `compute_bonding_curve_price`, `get_curve_slope` |
| `CurvePreset(Address)` | Per-creator | `CurvePreset` (`Linear` \| `Quadratic` \| `Flat`) | `register_creator` (always set; defaults to `Linear` if not specified — presets are immutable after registration) | `compute_bonding_curve_price`, `get_curve_preset` |
| `TreasuryBalance` | Global | `i128` | `credit_treasury_balance`, `withdraw_treasury` (admin) | `read_treasury_balance`, `get_treasury_balance` |
| `CoCreator(Address)` | Per-creator | `CoCreatorConfig { address, share_bps }` | `register_creator` (optional, immutable once set) | `read_co_creator_config`, `get_co_creator` |
| `CoCreatorFeeBalance(Address, Address)` | Per-creator-cocreator (composite) | `i128` | `credit_co_creator_fee_balance` (internal fee-split accrual) | `read_co_creator_fee_balance`, `get_co_creator_fee_balance` |
| `Whitelist(Address)` | Per-creator | `WhitelistConfig { addresses, window_ledgers }` | `register_creator` (optional) | `read_whitelist_config`, `whitelist_status`, `assert_whitelist_allows_buy`, `get_whitelist_status` |
| `MaxKeysPerWallet(Address)` | Per-creator | `u32` | `register_creator` (optional) | `buy_key`, `airdrop_keys` (cap enforcement), `get_wallet_cap` |
| `ReferralFeeBps` | Global | `u32` | `set_referral_fee_bps` (admin) | `buy_key_with_referrer` (referral split), `get_referral_fee_bps` |
| `DiscountTiers` | Global | `Vec<DiscountTier>` | `update_discount_tiers` (admin) | `get_discount_tiers` (volume-based fee discount evaluation) |
| `CreatorVolume(Address)` | Per-creator | `i128` | *(not currently written by any entrypoint)* | `get_creator_volume` |

> **Note:** `CreatorVolume(Address)` is read by `get_creator_volume` but has no
> writer in the current implementation, so it always resolves to its `0`
> default today. Treat it as a reserved key for a not-yet-wired volume-tracking
> feature rather than active state.

## TTL extension behavior

The contract does **not** use the `ttl::should_extend(current_ttl, threshold)`
pure helper (`creator-keys/src/lib.rs`) to decide *whether* to bump TTL on the
hot buy/sell path — that helper is an isolated, unit-testable decision
function (see its tests in `lib.rs`) but is not currently wired into
`extend_creator_ttl`.

Instead, `extend_creator_ttl(env, creator)` runs **unconditionally** after
every successful `register_creator`, `buy_key`/`buy_key_with_referrer`, and
`sell_key` call, and extends the TTL of every creator-scoped key that is
currently present in storage:

- `Creator(creator)` — always extended (the key that must exist for the call to have succeeded)
- `CreatorFeeBalance(creator)` — extended only if present
- `DividendPerKeyAccumulated(creator)` — extended only if present
- `LockedAllocation(creator)` — extended only if present
- `MaxSupply(creator)` — extended only if present
- `CurvePreset(creator)` — extended only if present
- `CoCreator(creator)` and, if set, `CoCreatorFeeBalance(creator, co_creator)` — extended only if present

Each extension call is `env.storage().persistent().extend_ttl(&key,
threshold, extend_to)` where:

- `threshold = env.ledger().sequence()` (the current ledger)
- `extend_to = env.ledger().sequence() + CREATOR_TTL_LEDGERS`
- `CREATOR_TTL_LEDGERS = 6_311_520` (~2 years at 5s per ledger)

Passing the current ledger sequence as `threshold` means the Soroban runtime's
"only extend if remaining TTL is below `threshold` ledgers" check is
effectively always satisfied for any contract that has been live longer than
`CREATOR_TTL_LEDGERS` ledgers behind its current entries, so in practice every
successful buy/sell/register call re-bumps creator-scoped TTLs by the full
`CREATOR_TTL_LEDGERS` window.

A successful `extend_creator_ttl` call emits a TTL-extension event (see
`events::ttl_extended_topics`) once per invocation, regardless of how many
individual keys were extended underneath it.

Entries **not** covered by `extend_creator_ttl` (global config keys such as
`FeeConfig`, `KeyPrice`, `AdminAddress`, `TreasuryAddress`,
`ProtocolFeeRecipient`, `CurveSlope`, `ReferralFeeBps`, `DiscountTiers`, and
per-holder keys like `KeyBalance(creator, holder)` and the dividend
checkpoint/pending pair) do not receive automatic TTL bumps from trade
activity and should be covered by an operational maintenance job if long-term
persistence is required — see
[creator-state-storage-ttl.md](./creator-state-storage-ttl.md) for the
recommended bump strategy.
