const Big = require("big.js");
module.exports = {
  getConfig: (env) => {
    const config = (() => {
      switch (env) {
        case "production":
        case "mainnet":
          return {
            networkId: "mainnet",
            nodeUrl: process.env.NODE_URL || "https://rpc.mainnet.near.org",
            walletUrl: "https://wallet.near.org",
            helperUrl: "https://helper.mainnet.near.org",
            explorerUrl: "https://explorer.mainnet.near.org",
            refFinanceContractId: "v2.ref-finance.near",
            priceOracleContractId: "priceoracle.near",
            burrowContractId: "contract.main.burrow.near",
            accountId: process.env.NEAR_ACCOUNT_ID,
            wrapNearAccountId: "wrap.near",
          };
        case "development":
          // return {
          //   networkId: "testnet",
          //   nodeUrl: process.env.NODE_URL || "https://rpc.testnet.near.org",
          //   walletUrl: "https://wallet.testnet.near.org",
          //   helperUrl: "https://helper.testnet.near.org",
          //   explorerUrl: "https://explorer.testnet.near.org",
          //   // refFinanceContractId: "dev-1702352304469-22165716719418",
          //   refFinanceContractId: "ref-finance-101.testnet",
          //   priceOracleContractId: "dev-1700791085144-86637101874849",
          //   burrowContractId: "dev-1702353298377-47206148439586",
          //   accountId: process.env.NEAR_ACCOUNT_ID,
          //   wrapNearAccountId: "wrap.testnet",
          // };
          return {
            networkId: "testnet",
            nodeUrl: process.env.NODE_URL || "https://rpc.testnet.near.org",
            walletUrl: "https://wallet.testnet.near.org",
            helperUrl: "https://helper.testnet.near.org",
            explorerUrl: "https://explorer.testnet.near.org",
            refFinanceContractId: "dev-1702262174571-14898860791369",
            priceOracleContractId: "mock-priceoracle.testnet",
            burrowContractId: "dev-1702262471840-76656511729131",
            accountId: process.env.NEAR_ACCOUNT_ID,
            wrapNearAccountId: "wrap.testnet",
          };
        case "testnet":
          return {
            networkId: "testnet",
            nodeUrl: process.env.NODE_URL || "https://rpc.testnet.near.org",
            walletUrl: "https://wallet.testnet.near.org",
            helperUrl: "https://helper.testnet.near.org",
            explorerUrl: "https://explorer.testnet.near.org",
            refFinanceContractId: "ref-finance-101.testnet",
            priceOracleContractId: "priceoracle.testnet",
            burrowContractId: "contract.1638481328.burrow.testnet",
            accountId: process.env.NEAR_ACCOUNT_ID,
            wrapNearAccountId: "wrap.testnet",
          };
        default:
          throw Error(
            `Unconfigured environment '${env}'. Can be configured in src/config.js.`
          );
      }
    })();
    config.minProfit = Big(process.env.MIN_PROFIT || "1.0");
    config.minDiscount = Big(process.env.MIN_DISCOUNT || "0.025");
    config.showWhales = !!process.env.SHOW_WHALES;
    config.minSwapAmount = Big(process.env.MIN_SWAP_AMOUNT || "1");
    config.minRepayAmount = Big(process.env.MIN_REPAY_AMOUNT || "0.5");
    config.maxSlippage = Big(process.env.MAX_SLIPPAGE || "0.5");
    config.maxLiquidationAmount = Big(
      process.env.MAX_LIQUIDATION_AMOUNT || "20000"
    );
    config.maxWithdrawCount = parseInt(process.env.MAX_WITHDRAW_COUNT || "5");
    config.forceClose = !!process.env.FORCE_CLOSE;
    return config;
  },
};