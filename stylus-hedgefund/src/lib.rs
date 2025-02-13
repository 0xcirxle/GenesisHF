#![cfg_attr(not(any(feature = "export-abi", test)), no_main)]
extern crate alloc;

use stylus_sdk::{
    prelude::*,
    msg,
    alloy_primitives::{U256, Address, FixedBytes},
    storage::{StorageMap, StorageU256, StorageAddress},
    call::{Call, transfer_eth},
    contract,
};

use alloc::string::String;

// Define the lending pool address as a constant
pub const LENDING_POOL: Address = Address(FixedBytes([
    0xec, 0x7d, 0x48, 0x9A, 0x9a, 0x8E, 0xB4, 0x1D,
    0xA7, 0x09, 0xe1, 0xcf, 0x91, 0x4C, 0xAe, 0x9B,
    0x37, 0x62, 0xC4, 0x7D
]));

// Define the swap pool address as a constant
pub const SWAP_POOL: Address = Address(FixedBytes([
    0xF8, 0x48, 0x8c, 0x0b, 0xdD, 0x0D, 0x02, 0xd2,
    0xB7, 0x7d, 0xE8, 0x00, 0xb5, 0x57, 0xb2, 0x6E,
    0xA2, 0x8e, 0x56, 0xAb
]));

// 1) Define swap interface for LINK/WBTC
sol_interface! {
    interface IGeneralUniswapV2 {
        function swapETHForToken() external payable;
    }
}

// 2) Define lending interface
sol_interface! {
    interface ILendingPool {
        function deposit() external payable;
        function withdraw(uint256 amount) external;
    }
}

/// 3) Our HedgeFund storage
#[storage]
#[entrypoint]
pub struct HedgeFund {
    total_shares: StorageU256,
    shares_of: StorageMap<Address, StorageU256>,

    swap_link: StorageAddress,
    swap_wbtc: StorageAddress,
    lending_pool: StorageAddress,

    link_token: StorageAddress,
    wbtc_token: StorageAddress,
}

/// 4) Implementation of core methods
#[public]
impl HedgeFund {
    /// init(...) sets the addresses for your swap/lending
    pub fn init(
        &mut self,
        link_swap_addr: Address,
        wbtc_swap_addr: Address,
        lending_addr: Address,
        link_addr: Address,
        wbtc_addr: Address
    ) {
        self.swap_link.set(link_swap_addr);
        self.swap_wbtc.set(wbtc_swap_addr);
        self.lending_pool.set(lending_addr);

        self.link_token.set(link_addr);
        self.wbtc_token.set(wbtc_addr);
    }

    /// deposit() - user sends ETH, we mint them shares 1:1, then split the ETH:
    ///  (A) 30% for LINK, (B) 30% half->ETH & half->WBTC, (C) 40% => lending
    #[payable]
    pub fn deposit(&mut self) {
        let caller = msg::sender();
        let value_in = msg::value();

        // Mint shares 1:1
        let total = self.total_shares.get();
        let new_shares = value_in; 
        let mut user_shares = self.shares_of.setter(caller);
        let current_shares = user_shares.get();
        user_shares.set(current_shares + new_shares);
        self.total_shares.set(total + new_shares);

        // 30% => portion_a => swapETHForToken (LINK)
        let portion_a = value_in * U256::from(30) / U256::from(100);
        if portion_a > U256::ZERO {
            let uniswap_a = IGeneralUniswapV2::new(SWAP_POOL);
            let call_conf = Call::new_in(self).value(portion_a);
            let _ = uniswap_a.swap_eth_for_token(call_conf);
        }

        // 30% => portion_b => half remains ETH, half => WBTC
        let portion_b = value_in * U256::from(30) / U256::from(100);
        let half_b = portion_b / U256::from(2);
        let to_swap_b = portion_b - half_b;
        if to_swap_b > U256::ZERO {
            let uniswap_b = IGeneralUniswapV2::new(SWAP_POOL);
            let call_conf = Call::new_in(self).value(to_swap_b);
            let _ = uniswap_b.swap_eth_for_token(call_conf);
        }
        // The half_b just remains as ETH in this contract

        // 40% => portion_c => deposit into lending
        let portion_c = value_in - portion_a - portion_b;
        if portion_c > U256::ZERO {
            let pool = ILendingPool::new(LENDING_POOL);
            let call_conf = Call::new_in(self).value(portion_c);
            let _ = pool.deposit(call_conf);
        }

        // no logs or events
    }

    /// withdraw(shares)
    /// naive approach: user gets share_amount / total_shares fraction of the contract's ETH
    /// ignoring tokens/lending
    pub fn withdraw(&mut self, share_amount: U256) {
        let caller = msg::sender();
        let mut user_shares = self.shares_of.setter(caller);
        let old_shares = user_shares.get();
        if old_shares < share_amount {
            panic!("Not enough shares");
        }
        user_shares.set(old_shares - share_amount);

        let total = self.total_shares.get();
        self.total_shares.set(total - share_amount);

        // proportionate from contract's ETH
        let contract_balance = contract::balance();
        let payout = if total > U256::ZERO {
            (contract_balance * share_amount) / total
        } else {
            contract_balance
        };

        if payout > U256::ZERO {
            transfer_eth(caller, payout).ok();
        }
    }

    pub fn rebalance(&mut self) {
        let contract_balance = contract::balance();
    
        // If no leftover ETH, nothing to do
        if contract_balance == U256::ZERO {
            return;
        }
    
        // We'll reuse the same 30/30/40 approach
        let portion_a = contract_balance * U256::from(30) / U256::from(100); // for LINK
        let portion_b = contract_balance * U256::from(30) / U256::from(100); // half/half for WBTC
        let portion_c = contract_balance - portion_a - portion_b;            // deposit to Lending
    
        // 1) Swap portion_a for LINK
        if portion_a > U256::ZERO {
            let uniswap_a = IGeneralUniswapV2::new(SWAP_POOL);
            let call_conf = Call::new_in(self).value(portion_a);
            let _ = uniswap_a.swap_eth_for_token(call_conf);
        }
    
        // 2) portion_b => half remains ETH, half => WBTC
        let half_b = portion_b / U256::from(2);
        let to_swap_b = portion_b - half_b;
        if to_swap_b > U256::ZERO {
            let uniswap_b = IGeneralUniswapV2::new(SWAP_POOL);
            let call_conf = Call::new_in(self).value(to_swap_b);
            let _ = uniswap_b.swap_eth_for_token(call_conf);
        }
        // half_b remains as ETH
    
        // 3) portion_c => deposit to Lending
        if portion_c > U256::ZERO {
            let pool = ILendingPool::new(LENDING_POOL);
            let call_conf = Call::new_in(self).value(portion_c);
            let _ = pool.deposit(call_conf);
        }
    }
    
    

    /// get_user_info(address)
    pub fn get_user_info(&self, user: Address) -> String {
        let bal = self.shares_of.get(user);
        let tot = self.total_shares.get();
        format!("User: 0x{:x}, Shares: {}, totalSupply: {}", user, bal, tot)
    }

    /// get_agent_invests()
    pub fn get_agent_invests(&self) -> String {
        let tot = self.total_shares.get();
        let eth_bal = contract::balance();
        format!("Total Shares: {}, Contract ETH: {}", tot, eth_bal)
    }
}
