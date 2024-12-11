# stablecoin_protocolv2

**stablecoin_protocolv2** is a Solana-based program (smart contract) built using the Anchor framework. The protocol provides a comprehensive suite of features for minting, burning, staking, and governing stablecoins while ensuring robust collateralization and liquidation mechanisms.

**NOTE:** This is a modified version of a project that was in development from **September through October**. 
**THIS PROJECT WAS COMPLETED IN OCTOBER**
The project was initially developed using native Solana (https://github.com/btorressz/stablecoin_protocol); however, Anchor proved to be a better option for this project.

devnet

---

## Features

### 1. Collateralized Stablecoin Minting
- Mint stablecoins by locking collateral.
- Burn stablecoins to redeem collateral.
- Enforces collateral ratios to maintain protocol stability.

### 2. Staking and Rewards
- Stake tokens to earn rewards.
- Withdraw staked tokens with optional early withdrawal penalties.
- Reward distribution based on staking duration.

### 3. Governance
- Decentralized governance through proposals and voting.
- Minimum quorum required for proposal approval.
- Supports community-driven decision-making.

### 4. Liquidation
- Automatic liquidation of under-collateralized accounts.
- Imposes penalties to ensure proper collateralization.

---

## Program Overview (Smart Contract)

### Instructions

#### 1. Initialization
- Initialize the protocol by setting a global collateral ratio.

#### 2. Minting and Burning
- Mint stablecoins by locking collateral based on the required collateral ratio.
- Burn stablecoins to redeem locked collateral.

#### 3. Staking
- Stake collateral to earn rewards.
- Withdraw staked tokens with penalties for early withdrawal.
- Rewards calculated based on staking duration.

#### 4. Liquidation
- Partially liquidate under-collateralized accounts.
- Enforces collateralization and imposes penalties.

#### 5. Governance
- Submit proposals for community voting.
- Vote on proposals to determine protocol changes.

---

## Account Structures

### Governance
- Stores global collateral ratio and protocol settings.

### UserAccount
- Tracks user's collateral and stablecoin balances.

### StakerAccount
- Manages staking balances, lock-up periods, and rewards.

### Proposal
- Stores proposal details, votes, and status.

---

## Error Codes

- **InsufficientCollateral**: Insufficient collateral to mint stablecoins.
- **InsufficientBalance**: Insufficient stablecoin balance to burn.
- **Overflow**: Numeric calculation overflow.
- **NotEligibleForLiquidation**: User is not under-collateralized.

---

## LICENSE

This project is licensed under the **MIT License**.
