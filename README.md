# Stylus HedgeFund

Welcome to **Genesis HedgeFund**! Our project implements a simplified hedge fund smart contract on the Stylus environment, allowing users to deposit ETH, tokenize them into shares, then allocate funds into different assets and lending protocols. Below is a detailed overview of the project.

## What is GenesisHF?

This project is a proof-of-concept DeFi hedge fund, implemented in Rust for the Stylus execution environment. It demonstrates how a DeFi protocol can be built to manage user deposits in ETH, split them into different risk-based asset classes i.e. High, medium and low risk (like LINK, WBTC, or lending platforms), then provide a redemption process by withdrawing proportional assets.

## What does it do?

We:
- **Collect ETH** from users.
- **Tokenize deposits** by issuing *shares* corresponding to their ownership.
- **Allocate** a percentage of the ETH deposit to:
  - A high risk investment category consisiting of utility tokens like LINK.
  - A medium risk category consisting of assets like WBTC, ETH, and other crypto assets.
  - A low risk category consisting of stablecoin lending yields like depositing USDC into a lending pool.
- **Allow withdraws** of proportionate ETH, depending on the user's share balance.
- **Enable rebalancing** of leftover ETH in the contract to maintain target allocations.

## How does genesisHF do it?

1. A user first starts the CLI and logs in using their private key.
2. The user then calls the `deposit` function, sending ETH.  
3. This contract mints new shares, which track the user’s ownership stake.  
4. The contract then automatically splits the ETH into several parts: 
   - **30%** swaps to high risk investment category.
   - **30%** gets half swapped to medium risk investment category, half remains as ETH.
   - **40%** gets deposited into the low risk investment category.  
5. Upon `withdraw`, the user redeems shares for a fraction of the ETH that the contract currently holds.  
6. The `rebalance` function can reorganize leftover ETH in the same ratio (30/30/40) if for some reason there is leftover ETH.

## What changes did we make due to the gas error when deploying on mainnet?

During deployment on mainnet, we ran into **gas limit issues** which are present in the current version of the Stylus environment. As a result:
1. **Testnet Deployment** – We switched to a testnet environment to ensure our contract worked as intended.  
2. **Placeholder Addresses** – Actual mainnet pool addresses and lending pool addresses weren’t available for our testing environment. We used our own test addresses as placeholders for the swap (Uniswap-like) pools and the lending pool.  
3. **Contract Deployment** – We deployed our own pool and lending contracts on arbitrum sepolia environment to ensure it worked as intended.

## How did we use AI agents in here?

We employed AI agents via the cdp kit to:
1. **Generating the distribution** – To generate the distribution of the ETH into the different risk-based asset classes.
2. **Generate class weights** – To generate the weights of the different risk-based asset classes.

## Project Files and CLI Explanation

### Files
- **`src/lib.rs`**  
  The main logic of the HedgeFund contract. It defines:
  - Storage for user shares.  
  - The deposit, withdraw, and rebalance functionalities.  
  - The references to swap interfaces (for LINK and WBTC).  
  - The references to a lending pool interface.

- **Additional Contracts (Test Contracts)**  
  In the test environment, we have placeholder addresses for the swap and lending pools. These are simplified test versions to simulate swaps and lending operations.

- **CLI**  
  - We are using the standard CLI tooling for deploying or interacting with this contract on Stylus-compatible networks.  
  - Typical commands include `deposit`, `withdraw`, `get balance`, and `rebalance`.

## Future Goals

1. **Integrate with Real DeFi Protocols**  
   - Connect to established DeFi protocols on test networks and eventually on mainnet once the gas-related issues are resolved.

2. **Expand Token Support**  
   - Allow more diverse allocation strategies, including stablecoins and additional tokens.

3. **Automated Harvesting & Rebalancing**  
   - Introduce periodic harvest of lending interest and rebalancing of the entire portfolio to maintain the desired asset split.

4. **DAO Governance**  
   - Migrate to a governance model where token holders vote on the ratio of allotments and distribution strategies.

## Architecture Explanations

Below are some conceptual architectures showing how the pieces fit together:

### Diagram 1: Deposit Flow
```
 User (Metamask or CLI)
       |
       | send transaction + ETH
       v
 [ Smart Contract ] -- (swap) --> [Swap Pool for LINK]
       |                (swap) --> [Swap Pool for WBTC]
       |                (lend) --> [Lending Pool]
       |-> Issues "Shares"
```
1. User calls deposit.  
2. Contract receives ETH.  
3. Splits ETH across swap pools (LINK & WBTC) and lending.  
4. Mints shares to user.

---

### Diagram 2: Withdraw Flow
```
 User (with some shares)
       |
       | call "withdraw" with share_amount
       v
 [ Smart Contract ]
       |-> Burns user’s shares (removes from total)
       |-> Calculates user’s proportion of ETH holdings
       |-> Transfers ETH to user
```
1. User requests a specific share amount to withdraw.  
2. Contract calculates how much ETH that share amount represents.  
3. Sends ETH back to the user.

---

### Diagram 3: Rebalancing
```
          [ Smart Contract ]
 leftover ETH -> 30% --> [ Swap to LINK ]
                30% --> [ Swap to WBTC ] + leftover ETH
                40% --> [ Deposit to Lending ]
```
- Periodically consolidates leftover ETH into the same 30/30/40 ratio.  

---

Feel free to contribute, open issues, or suggest improvements. We hope you find **GenesisHF** a helpful demonstration of how a DeFi hedge fund might be implemented on a chain with the new Stylus environment.

Happy hacking!
