const nearAPI = require("near-api-js");
const { connect, keyStores, utils } = nearAPI;
const path = require("path");
const os = require("os");
const Big = require("big.js");
// import fetch from 'node-fetch';
const fetch = require('node-fetch');


const { getConfig } = require("./config");
function sleep(ms) {
    return new Promise(resolve => setTimeout(resolve, ms));
}
async function updatePriceFeeds() {
    for (let i = 0; i < 100; i++) {
        const response = await fetch('https://hermes-beta.pyth.network/api/latest_vaas?ids[]=27e867f0f4f61076456d1a73b14c7edc1cf5cef4f4d6193a33424288f11bd0f4&ids[]=1fc18861232290221461220bd4e2acd1dcdfbc89c84092c93c18bdc7756c1588',
            {
                method: "GET",
                headers: {
                    "accept": 'application/json'
                },
            });
        let body = await response.json();
        const rawData = body[0];

        const buffer = Buffer.from(rawData, 'base64');
        const data = buffer.toString('hex');
        const NearConfig = getConfig("development");

        const keyStore = new nearAPI.keyStores.InMemoryKeyStore();
        // const keyPath = path.join(os.homedir(), ".near-credentials", "testnet", "dev-1701055183458-23613131216293" + ".json");
        const keyPath = path.join(os.homedir(), ".near-credentials", "testnet", "zerochltest.testnet" + ".json");
        const near = await nearAPI.connect(
            Object.assign({keyPath, deps: {keyStore}}, NearConfig)
        );
        // const account = new nearAPI.Account(near.connection, "dev-1701055183458-23613131216293");
        const account = new nearAPI.Account(near.connection, "zerochltest.testnet");
        const contract = new nearAPI.Contract(
            account,
            "pyth-oracle.testnet",
            {
                viewMethods: [],
                changeMethods: ["update_price_feeds"],
            }
        );
        const result = await contract.update_price_feeds(
            {
                data
            },
            Big(10).pow(12).mul(300).toFixed(0),
            Big(10).pow(24).toFixed(0)
        )
        console.log("Update Price Feeds Result: ", result);
        await sleep(5000);

        // const priceFeed1 = await account.viewFunction("pyth-oracle.testnet", "get_price", {
        //     price_identifier: "1fc18861232290221461220bd4e2acd1dcdfbc89c84092c93c18bdc7756c1588",
        // });
        // console.log("after viewFunction")
        // // 1fc18861232290221461220bd4e2acd1dcdfbc89c84092c93c18bdc7756c1588
        // // 1fc18861232290221461220bd4e2acd1dcdfbc89c84092c93c18bdc7756c1588
        // console.log("Price Feed Data: ", priceFeed1);
        //
        // const priceFeed = await account.viewFunction("pyth-oracle.testnet", "get_price", {
        //     price_identifier: "27e867f0f4f61076456d1a73b14c7edc1cf5cef4f4d6193a33424288f11bd0f4",
        // });
        // console.log("after viewFunction")
        // // 27e867f0f4f61076456d1a73b14c7edc1cf5cef4f4d6193a33424288f11bd0f4
        // // 27e867f0f4f61076456d1a73b14c7edc1cf5cef4f4d6193a33424288f11bd0f4
        // console.log("Price Feed Data: ", priceFeed);
    }
}

updatePriceFeeds()
