// Import the API, Keyring and some utility functions
const { ApiPromise, WsProvider } = require('@polkadot/api');
const { Keyring } = require('@polkadot/keyring');
const to = require("await-to-js").default;

const delay_promise = function (ms) {
    return new Promise(function (resolve, reject) {
        setTimeout(() => {
            reject(`Timeout in ${ms} ms`);
        }, ms);
    });
};

exports.sleep = function sleep(ms) {
    return new Promise(res => setTimeout(res, ms));
}

async function get_api(url_string) {
    // example of url_string: wss://138.68.229.14:443
    process.env.NODE_TLS_REJECT_UNAUTHORIZED = "0";
    const wsProvider = new WsProvider(url_string);
    console.log("start wsprovider...");

    let promiseA = ApiPromise.create({
        provider: wsProvider,
        types: {
            Address: "MultiAddress",
            LookupSource: "MultiAddress",
            AccountInfo: 'AccountInfoWithDualRefCount',
            Balance: 'u128',
            Timestamp: 'Moment',
            BlockNumber: 'u32',
            IpV4: 'Vec<u8>',
            CountryRegion: 'Vec<u8>',
            Duration: 'u8',
            Node: {
                account_id: 'AccountId',
                ipv4: 'IpV4',
                country: 'CountryRegion',
                expire: 'BlockNumber',
            },
            ChannelOf: {
                sender: 'AccountId',
                receiver: 'AccountId',
                balance: 'Balance',
                nonce: 'u64',
                opened: 'BlockNumber',
                expiration: 'BlockNumber',
            },
            CreditDelegateInfo: {
                delegator: 'AccountId',
                score: 'u64',
                validators: 'Vec<AccountId>',
            },
        },
    });

    let promiseB = delay_promise(30000);

    let race = Promise.race([promiseA, promiseB]);
    let [err, api] = await to(race);
    console.log(`hehe err: ${err}`);
    console.log(`hehe api: ${api}`);
    return api;
}

exports.get_api = get_api;