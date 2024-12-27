# Home of Dumplings / \$DPL Tokenomics

## 1. Overview

The Home of Dumplings protocol deploys AI agents that solve optimization problems for DeFi users and protocols, addressing tasks such as liquidity management and parameter tuning. The ecosystem operates through **Tasks** hosted on-chain and populations of solution programs evolved off-chain by the Optimizer Pool.

**\$DPL** is the native token powering this ecosystem. Its primary roles include:

1. **Payment**: Clients pay invocation or subscription fees in \$DPL.
2. **Staking and Governance**: Token holders stake \$DPL to secure the protocol, join the optimizer pool or participate in governance.
3. **Rewards**: Optimizer Pool nodes earn \$DPL for validated contributions to the evolution of programs.

---

## 2. Token Supply and Distribution

### Fixed Supply

- **Total Supply**: \$DPL has a fixed supply (e.g., 100M tokens), minted at genesis.
- **No Inflation**: No further token minting or emission beyond the initial allocation.

### Allocation of Supply

1. **Uniswap V3 Liquidity Pool**

   - All of the supply is allocated to a Uniswap V3 liquidity pool over a wide price range.
   - Multiple price ranges hold extra liquidity to ensure stable price discovery.

2. **Team**

   - A vesting contract purchases a small portion (e.g., 10-15%) of the cheapest tokens from the pool right after deployment and allocates them to the team.

3. **Treasury**

   - An NFT minting contract purchases a small portion (e.g., 5-10%) right after the vesting contract before other buyers come in. The tokens are packaged into nice-looking NFTs.
   - These NFTs are sold on marketplaces like OpenSea, redeemable for tokens. The proceeds are transferred to the team multisig, forming the treasury.

---

## 3. Revenue Model

### Fee Structure

1. **Invocation Fees**

   - Charged for AI agents evolved with synthetic data.
   - Paid in \$DPL or other currencies auto-converted into \$DPL and routed to the protocol reward pools.

2. **Subscription Fees**

   - Charged for AI agents evolved with synthetic data initially, but are continuously improved by backtesting with real data.
   - Periodic payment in \$DPL or other currencies auto-converted into \$DPL and routed to the protocol reward pools.

### Reward Allocation

- A portion of the fees collected funds rewards for the Optimizer Pool nodes that contribute validated solutions to Tasks.
- Governance determines the exact split between:
  - Optimizer Pool rewards.
  - Treasury for operational funding.
  - Potential staking rewards.

---

## 4. Staking

### Types of Stakers

1. **Council Node Stakers**

   - Nodes in the Optimizer Pool stake \$DPL to become eligible for selection to the Council.
   - Council members validate and vote on improved solutions to Tasks.
   - **Slashing Mechanism**: If a Council member votes against the majority decision, they are penalized by slashing a portion of their staked tokens.

2. **Investor Stakers**

   - Investors stake \$DPL to earn a share of the protocol’s revenue.
   - Revenue is comprised of invocation and subscription fees, a part is distributed as staking rewards.

### Risk Backstop

- Stakers collectively backstop potential risks associated with the AI agents.
- If an AI agent malfunctions (e.g., outputs suboptimal or incorrect decisions due to flaws in the algorithm), affected users may be compensated via the staking pool, reducing the system’s reputational and financial risk.
- Governance defines the conditions for such compensations, ensuring fairness and transparency.

---

## 5. Governance

Voting weight is proportional to the amount of \$DPL staked.

### Governance Mechanics

1. **Voting Rights**

   - All stakers can participate in governance, regardless of whether they are nodes or investors.
   - Proposals focus on:
     - Adjusting fee parameters (e.g., invocation and subscription fees).
     - Allocating Optimizer Pool rewards.
     - Defining compensation rules for Risk Backstop events.

2. **Council Randomness**

   - Optimizer Pool nodes with staked \$DPL can be selected for the Council via a verifiable random selection process.
   - The Council validates and votes on improved solutions to Tasks.

---

## 6. NFT-Based Early Bird Mechanism

1. **Purpose**

   - Raise initial funding and incentivize early supporters.

2. **Mechanics**

   - NFTs represent the right to claim a specified quantity of \$DPL at a discounted rate.
   - Sold on marketplaces like OpenSea.

3. **Redemption**

   - Upon redemption, the NFT is burned, and the holder receives their allocated \$DPL.
   - Vesting or time locks may apply to align with protocol growth.

---

## 7. Fee Usage and Distribution

- **Optimizer Pool Rewards**: A fraction of collected fees funds rewards for Optimizer Pool nodes.
- **Treasury**: Supports operations, development, and marketing initiatives.
- **Stakers**: Receive a share of fees as staking rewards, subject to governance decisions.

---

## 8. Launch and Go-to-Market Strategy

### Initial Liquidity Deployment

- Deploy a majority of \$DPL supply into Uniswap V3 pools.
- Use multiple concentrated price ranges to establish liquidity depth and facilitate market stability.

### Early Bird NFT Sale

- Launch a curated NFT collection for discounted \$DPL rights.
- Drive initial funding and engage early adopters.

### Target Customers

- Position DeFi protocols as key customers for SoLUSH’s optimization services.
- Focus marketing on showcasing cost-saving and performance-enhancing use cases.

---

## 9. Conclusion

The \$DPL tokenomics framework is designed to:

- Provide immediate utility through payment and staking mechanisms.
- Incentivize Optimizer Pool participation via rewards for validated solutions.
- Ensure sustainable growth through fixed supply, governance, and clear fee allocation.

This structure ensures the long-term viability and scalability of the SoLUSH ecosystem, while fostering community involvement and alignment with protocol goals.


