// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "../openzeppelin-contracts/contracts/token/ERC20/ERC20.sol";

contract LendingPool is ERC20 {
    address public owner;

    constructor() ERC20("my Lending Pool Token", "mLPT") {
        owner = msg.sender;
    }

    // Deposit ETH and mint LP tokens 1:1
    function deposit() public payable {
        require(msg.value > 0, "Must deposit ETH");
        _mint(msg.sender, msg.value);
    }

    // Withdraw ETH by burning LP tokens
    function withdraw(uint256 amount) external {
        require(balanceOf(msg.sender) >= amount, "Insufficient LP tokens");
        _burn(msg.sender, amount);
        payable(msg.sender).transfer(amount);
    }

    // Fallback function to accept ETH deposits
    receive() external payable {
        deposit();
    }
}