// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "../lib/openzeppelin-contracts/contracts/token/ERC20/ERC20.sol";

contract GeneralUniswapV2 {
    IERC20 public token;
    uint256 public totalLiquidity;
    mapping(address => uint256) public liquidity;

    event LiquidityAdded(address indexed provider, uint256 ethAmount, uint256 tokenAmount);
    event LiquidityRemoved(address indexed provider, uint256 ethAmount, uint256 tokenAmount);
    event Swapped(address indexed user, uint256 ethIn, uint256 tokenOut);
    event SwappedReverse(address indexed user, uint256 tokenIn, uint256 ethOut);

    constructor(address _token) {
        token = IERC20(_token);
    }

    function addLiquidity(uint256 tokenAmount) external payable {
        require(msg.value > 0 && tokenAmount > 0, "Must provide ETH and seth token");
        
        token.transferFrom(msg.sender, address(this), tokenAmount);
        liquidity[msg.sender] += msg.value;
        totalLiquidity += msg.value;

        emit LiquidityAdded(msg.sender, msg.value, tokenAmount);
    }

    function removeLiquidity(uint256 ethAmount) external {
        require(liquidity[msg.sender] >= ethAmount, "Not enough liquidity");

        uint256 tokenAmount = (address(this).balance * ethAmount) / totalLiquidity;
        liquidity[msg.sender] -= ethAmount;
        totalLiquidity -= ethAmount;

        payable(msg.sender).transfer(ethAmount);
        token.transfer(msg.sender, tokenAmount);
        
        emit LiquidityRemoved(msg.sender, ethAmount, tokenAmount);
    }

    function getAmountOut(uint256 inputAmount, uint256 inputReserve, uint256 outputReserve) public pure returns (uint256) {
        uint256 inputAmountWithFee = inputAmount * 997; // 0.3% fee
        uint256 numerator = inputAmountWithFee * outputReserve;
        uint256 denominator = (inputReserve * 1000) + inputAmountWithFee;
        return numerator / denominator;
    }

    function swapETHForToken() external payable {
        require(msg.value > 0, "Must send ETH");

        uint256 tokenOut = getAmountOut(msg.value, address(this).balance - msg.value, token.balanceOf(address(this)));
        require(tokenOut > 0, "Insufficient output amount");
        
        token.transfer(msg.sender, tokenOut);
        emit Swapped(msg.sender, msg.value, tokenOut);
    }

    function swapTokenForETH(uint256 tokenAmount) external {
        require(tokenAmount > 0, "Must send token");

        uint256 ethOut = getAmountOut(tokenAmount, token.balanceOf(address(this)), address(this).balance);
        require(ethOut > 0, "Insufficient output amount");
        
        token.transferFrom(msg.sender, address(this), tokenAmount);
        payable(msg.sender).transfer(ethOut);

        emit SwappedReverse(msg.sender, tokenAmount, ethOut);
    }

    receive() external payable {}
}