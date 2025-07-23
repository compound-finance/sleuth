// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.16;

interface MorphoVault {
  function asset() external view returns (address);
}

interface ERC20 {
  function allowance(address owner, address spender) external view returns (uint);

  function balanceOf(address owner) external view returns (uint);

  function decimals() external view returns (uint);

  function name() external view returns (string memory);

  function symbol() external view returns (string memory);
}

contract MorphoQuery {

  struct BaseAssetWithAllowance {
    address asset;
    uint decimals;
    string name;
    //uint price; Where does price come from in Morpho?
    string symbol;
    uint allowance;
  }

  function query(MorphoVault morphoVault, address payable account) public view returns (BaseAssetWithAllowance memory) {
    address baseAsset = morphoVault.asset();

    BaseAssetWithAllowance memory baseAssetWithAllowance = BaseAssetWithAllowance({
      asset: baseAsset,
      symbol: ERC20(baseAsset).symbol(),
      decimals: ERC20(baseAsset).decimals(),
      name: ERC20(baseAsset).name(),
      allowance: ERC20(baseAsset).allowance(account, address(morphoVault))
    });

    return baseAssetWithAllowance;
  }
}