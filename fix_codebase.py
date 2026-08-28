import re

with open("creator-keys/src/lib.rs", "r") as f:
    content = f.read()

# Remove AuctionConfig struct
content = re.sub(
    r'/// Pre-launch fixed-price auction configuration.*?\n/// transitions back to the curve automatically once the auction supply is exhausted\.\n#\[derive\(Clone, Debug, Eq, PartialEq\)\]\n#\[contracttype\]\npub struct AuctionConfig \{[^}]+\}\n\n',
    '', content, flags=re.DOTALL)

# Remove RoyaltyConfig struct  
content = re.sub(
    r'/// Creator royalty configuration for buy and sell fees\.\n#\[derive\(Clone, Debug, Eq, PartialEq\)\]\n#\[contracttype\]\npub struct RoyaltyConfig \{[^}]+\}\n\n',
    '', content, flags=re.DOTALL)

# Remove read_royalty_config function
content = re.sub(
    r'fn read_royalty_config\(env: &Env, creator: &Address\) -> Option<RoyaltyConfig> \{[^}]+\}\n\n',
    '', content, flags=re.DOTALL)

# Remove read_curve_exponent function
content = re.sub(
    r'fn read_curve_exponent\(env: &Env, creator: &Address\) -> Option<u32> \{[^}]+\}\n\n',
    '', content, flags=re.DOTALL)

# Remove read_staking_rewards_pool
content = re.sub(
    r'/// Reads the accumulated staking rewards pool.*?\npub fn read_staking_rewards_pool\(env: &Env, creator: &Address\) -> i128 \{[^}]+\}\n\n',
    '', content, flags=re.DOTALL)

# Remove read_total_staked
content = re.sub(
    r'/// Reads the total keys currently staked.*?\npub fn read_total_staked\(env: &Env, creator: &Address\) -> u32 \{[^}]+\}\n\n',
    '', content, flags=re.DOTALL)

# Remove credit_staking_rewards_pool
content = re.sub(
    r'/// Routes a share of a protocol fee collection.*?\nfn credit_staking_rewards_pool\([^)]+\) -> Result<\(\), ContractError> \{[^}]+\}\n\n',
    '', content, flags=re.DOTALL)

# Remove MAX_AUCTION_SUPPLY constant
content = content.replace(
    "/// Maximum number of keys a pre-launch auction can allocate at the fixed\n/// auction price before the bonding curve takes over.\npub const MAX_AUCTION_SUPPLY: u32 = 10_000;\n\n",
    "")

# Remove STAKE_LOCK_LEDGERS constant
content = content.replace(
    "/// Lock duration for staked keys before a reward claim is permitted (30 days\n/// at 5s per ledger).\npub const STAKE_LOCK_LEDGERS: u32 = 518_400;\n\n",
    "")

# Remove STAKING_REWARD_SHARE_BPS constant
content = content.replace(
    "/// Share of each protocol fee collection routed into a creator's staking\n/// rewards pool (10%), on top of the existing treasury/recipient split.\npub const STAKING_REWARD_SHARE_BPS: u32 = 1_000;\n\n",
    "")

# Remove the credit_staking_rewards_pool call in buy_key
content = content.replace(
    "            credit_staking_rewards_pool(&env, &creator, protocol_fee)?;\n",
    "")

with open("creator-keys/src/lib.rs", "w") as f:
    f.write(content)

print("Done removing structs, functions, and constants")
