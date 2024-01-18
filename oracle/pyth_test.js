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
        const response = await fetch('https://hermes-beta.pyth.network/api/latest_vaas?ids[]=0xca80ba6dc32e08d06f1aa886011eed1d77c77be9eb761cc10d72b7d0a2fd57a6&ids[]=0x41f3625971ca2ed2263e78573fe5ce23e13d2558ed3f2e47ab0f84fb9e7ae722',
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
    }
}

updatePriceFeeds()
