# 3 Days of DeFi: the_anchor_founder x Solana Turbine

![Watch the Highlight Video](https://i.ibb.co/SXs8sLd0/Screenshot-from-2026-01-14-17-57-59.png) https://x.com/solanaturbine/status/2010366944642494631

## About

This repository serves as proof of attendance and implementation for the "3 Days of DeFi" workshop hosted by The Anchor Founder and Solana Turbine.

The goal of this project is to build a comprehensive DeFi suite on Solana using the Anchor framework. The repository is structured to evolve over the three-day curriculum, covering Automated Market Makers (AMM), Lending markets, and Staking protocols.

## Roadmap

### Day 1: Constant Product AMM

Implementation of a decentralized exchange using the standard $X \times Y = K$ constant product curve. This module allows users to create pools, provide liquidity, and swap tokens.

**Key Features:**

- **Secure Liquidity Provision:** Implements geometric mean calculations ($\sqrt{X \times Y}$) for initial liquidity deposits to prevent first-depositor inflation attacks.
- **Slippage Protection:** Users specify maximum token inputs for a target LP output, ensuring transactions revert if the pool ratio shifts unfavorably.
- **Administrative Controls:**
  - **Lock:** Permanently locks the pool to prevent any further interaction.
  - **Pause:** Temporarily halts deposits and withdrawals.
  - **Timed Pause:** Allows the setting of specific time windows where pool operations are restricted (e.g., during maintenance or upgrades).

### Day 2: Lending Protocol (Upcoming)

Implementation of a money market allowing users to lend assets for interest and borrow against collateral.

### Day 3: Staking System (Upcoming)

Implementation of a staking rewards program where users lock tokens to earn yield over time.

---

## Technical Architecture: Day 1 (AMM)

The AMM program interacts with the SPL Token Interface to manage two distinct vaults (Vault X and Vault Y) and a Mint account for Liquidity Provider (LP) tokens.

### Deposit Logic

The deposit instruction differentiates between creating a new pool and adding to an existing one:

1.  **New Pool:**

    - The protocol ignores the user-supplied LP amount.
    - It calculates the initial LP supply using the geometric mean of the deposited assets: `Math.sqrt(token_x * token_y)`.
    - This mitigates manipulation risks where an attacker could mint excessive shares for dust amounts.

2.  **Existing Pool:**
    - The protocol calculates the required token inputs based on the current pool ratio.
    - Slippage checks ensure the user does not deposit more than their specified `max_x` or `max_y`.

### Security & Access Control

The `Config` account manages the state of the pool, including authority keys and boolean flags for pausing operations.

- **Pool Locked:** A global state that freezes the contract permanently.
- **Timed Pause:** Logic checks against the `UnixTimestamp` (clock) to determine if the current slot falls within a restricted window.

---

## Directory Structure

```text
.
├── programs
│   └── amm
│       ├── src
│       │   ├── instructions
│       │   │   ├── deposit.rs
│       │   │   ├── initialize.rs
│       │   │   └── mod.rs
│       │   ├── state
│       │   │   ├── config.rs
│       │   │   └── mod.rs
│       │   ├── constants.rs
│       │   ├── error.rs
│       │   └── lib.rs
│       └── Cargo.toml
├── tests
│   └── amm.ts
├── Anchor.toml
└── package.json

```

## Build Note

If `anchor build` fails with an error related to `constant_time_eq` (e.g., "failed to parse manifest"), it is due to a dependency update incompatible with the current Solana BPF toolchain.

To fix this, pin `blake3` to version 1.8.2 by running the following command in your terminal:

```bash
cargo update -p blake3 --precise 1.8.2

```

This prevents the package manager from pulling in the incompatible Rust 2024 edition crate.
