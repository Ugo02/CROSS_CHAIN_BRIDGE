// script/DeployTokenBridge.s.sol
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "forge-std/Script.sol";
import "../src/Bridge.sol";
import "../src/Basic_ERC20.sol";

contract DeployTokenBridge is Script {
    function run() external {
        uint256 deployerPrivateKey = vm.envUint("PRIVATE_KEY");
        
        vm.startBroadcast(deployerPrivateKey);

        TestToken token = new TestToken();        
        TokenBridge bridge = new TokenBridge();
        
        vm.stopBroadcast();
        
        console.log("TokenBridge deployed to:", address(bridge));
        console.log("TestToken deployed to:", address(token));
    }
}