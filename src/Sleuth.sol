// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.23;

type Id is bytes32;

interface Morpho {
  struct Market {
      uint128 totalSupplyAssets;
      uint128 totalSupplyShares;
      uint128 totalBorrowAssets;
      uint128 totalBorrowShares;
      uint128 lastUpdate;
      uint128 fee;
  }

  struct MarketParams {
    address loanToken;
    address collateralToken;
    address oracle;
    address irm;
    uint256 lltv;
  }

  function idToMarketParams(Id id) external view returns (address loanToken, address collateralToken, address oracle, address irm, uint256 lltv);

  function market(Id id) external view
        returns (
            uint128 totalSupplyAssets,
            uint128 totalSupplyShares,
            uint128 totalBorrowAssets,
            uint128 totalBorrowShares,
            uint128 lastUpdate,
            uint128 fee
        );

  function position(Id id, address user) external view returns (uint256 supplyShares, uint128 borrowShares, uint128 collateral);
}

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

  function query(MorphoVault morphoVault, Id [] memory marketIds, address baseTokenUsdFeed, address payable account) public view returns (BaseAssetWithAllowance memory) {
    address baseAsset = morphoVault.asset();

    //TODO: Base asset as no oracle from Base to USD but we really need one likely
    //      only Morpho collateral tokens have prices and that is in the base (loan)
    //      token terms. In order to support Morpho markets we need an oracle that goes
    //      from base (loan) token to USD, chainlink is probably fine here?

    BaseAssetWithAllowance memory baseAssetWithAllowance = BaseAssetWithAllowance({
      asset: baseAsset,
      symbol: ERC20(baseAsset).symbol(),
      decimals: ERC20(baseAsset).decimals(),
      name: ERC20(baseAsset).name(),
      allowance: ERC20(baseAsset).allowance(account, address(morphoVault))
    });

    //TODO: We should accept a list of Morpho marketIds here that we can use to
    //      query each individual vaultMarket for data and then return it
    //
    //      As part of the vault market, we should reutrn wallet balance, collateral supply balance
    //      allowance, price (priceUSD?)

    return baseAssetWithAllowance;
  }
}