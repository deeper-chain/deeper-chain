const { ApiPromise, WsProvider, Keyring } = require("@polkadot/api");
const { blake2AsU8a, secp256k1KeypairFromSeed, cryptoWaitReady } = require("@polkadot/util-crypto");
const stringToU8a = require("@polkadot/util/string/toU8a").default;
const path = require("path");
const fs = require("fs");
const BN = require("bn.js");
const to = require("await-to-js").default;

const WSS_URL = "wss://138.68.229.14:443";
const PUBLIC_ADDR = "5C4xaPznTFhENxuEqbuRMLh7aKuV3Jb8neRFLtV6dRM6xPs1";
const SECRET_KEY = "0x056336a6b9eb2fb5165664c640e83bd5130fc2cdb108126906db3e6610e9ae31//stash";

const delayPromise = function (ms) {
    return new Promise(function (resolve) {
        setTimeout(resolve, ms);
    });
};

const delay_promise = function (ms) {
    return new Promise(function (resolve, reject) {
        setTimeout(() => {
            reject(`Timeout in ${ms} ms`);
        }, ms);
    });
};

function toHexString(byteArray) {
    return Array.from(byteArray, function (byte) {
        return ("0" + (byte & 0xff).toString(16)).slice(-2);
    }).join("");
}

// nonce:u64, session_id:u32
function construct_byte_array(addr, nonce, session_id, amount) {
    let arr = [];
    nonce = nonce.toArray("be", 8);
    session_id = session_id.toArray("be", 4);
    amount = amount.toArray("le", 16); // amount is le encoded
    arr.push(...addr, ...nonce, ...session_id, ...amount);
    return arr;
}

async function get_api(url_string) {
    // example of url_string: wss://138.68.229.14:443
    process.env.NODE_TLS_REJECT_UNAUTHORIZED = "0";
    const wsProvider = new WsProvider(url_string);
    console.log("start wsprovider...");

    let promiseA = ApiPromise.create({
        provider: wsProvider,
        types: {
            TokenBalance: "u64",
            Timestamp: "Moment",
            Node: {
                account_id: "AccountId",
                ipv4: "Vec<u8>",
                country: "u16",
            },
            ChannelOf: {
                sender: "AccountId",
                receiver: "AccountId",
                nonce: "u64",
                opened: "Timestamp",
                expiration: "Timestamp",
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

async function getApiInstance(url_string) {
    // example of url_string: wss://138.68.229.14:443
    process.env.NODE_TLS_REJECT_UNAUTHORIZED = "0";
    const wsProvider = new WsProvider(url_string);

    let api = await ApiPromise.create({
        provider: wsProvider,
        types: {
            TokenBalance: "u64",
            Timestamp: "Moment",
            Node: {
                account_id: "AccountId",
                ipv4: "Vec<u8>",
                country: "u16",
            },
            ChannelOf: {
                sender: "AccountId",
                receiver: "AccountId",
                nonce: "u64",
                opened: "Timestamp",
                expiration: "Timestamp",
            },
        },
    });

    return api;
}

async function test() {
    let api = await getApiInstance(WSS_URL);

    // accounts
    const keyring = new Keyring({ type: "sr25519" });

    const alice = keyring.addFromUri("//Alice");
    const alice_stash = keyring.addFromUri("//Alice//stash");
    console.log(`Alice: ${alice.address}, Alice_Stash: ${alice_stash.address}`);

    const bob = keyring.addFromUri("//Bob");
    const bob_stash = keyring.addFromUri("//Bob//stash");
    console.log(`Bob: ${bob.address}, Bob_Stash: ${bob_stash.address}`);

    const charlie = keyring.addFromUri("//Charlie");
    console.log(`Charlie: ${charlie.address}`);

    const dave = keyring.addFromUri("//Dave");
    console.log(`Dave: ${dave.address}`);

    const eve = keyring.addFromUri("//Eve");
    console.log(`Eve: ${eve.address}`);

    const ferdie = keyring.addFromUri("//Ferdie");
    console.log(`Ferdie: ${ferdie.address}`);

    const chao0 = keyring.addFromMnemonic("wet wait more hammer glass drastic reform detect corn resource lake bomb");
    const chao0_stash = keyring.addFromUri("wet wait more hammer glass drastic reform detect corn resource lake bomb//stash");
    console.log(`Chao0: ${chao0.address}, Chao0_stash: ${chao0_stash.address}`);

    const chao1 = keyring.addFromMnemonic("license trigger sight gallery trophy before rough village clean become blur blast");
    const chao1_stash = keyring.addFromUri("license trigger sight gallery trophy before rough village clean become blur blast//stash");
    console.log(`Chao1: ${chao1.address}, Chao1_stash: ${chao1_stash.address}`);

    const chao2 = keyring.addFromMnemonic("discover despair state general virtual method ten someone rookie learn damage artefact");
    const chao2_stash = keyring.addFromUri("discover despair state general virtual method ten someone rookie learn damage artefact//stash");
    console.log(`Chao2: ${chao2.address}, Chao2_stash: ${chao2_stash.address}`);

    let nonce = new BN("3", 10);
    let session_id = new BN("11", 10);
    let base = new BN("1000000000000000", 10); // base = 1e15
    let amount = new BN("11", 10);
    let amt = amount.mul(base);
    //let res = construct_byte_array(bob.publicKey, nonce, session_id, amt);
    let res = construct_byte_array(charlie.publicKey, nonce, session_id, amt);
    let msg = blake2AsU8a(res);

    //let signature = alice.sign(msg);
    let signature = dave.sign(msg);
    let hexsig = toHexString(signature);
    console.log(`nonce: ${nonce}, session_id: ${session_id}, amt: ${amount}, signature: ${hexsig}`);

    api.tx.micropayment.claimPayment(dave.address, session_id, amt, "0x" + hexsig).signAndSend(charlie, ({ events = [], status }) => {
        console.log("Transaction status:", status.type);
        if (status.isInBlock) {
            console.log("Included at block hash", status.asInBlock.toHex());
            console.log("Events:");
            events.forEach(({ event: { data, method, section }, phase }) => {
                console.log("\t", phase.toString(), `: ${section}.${method}`, data.toString());
            });
        } else if (status.isFinalized) {
            console.log("Finalized block hash", status.asFinalized.toHex());
        }
    });
}

//test();
//-------------------------------------------------------------------------------------

async function getBalance(api) {
    let bal = await api.query.system.account(PUBLIC_ADDR);
    let free = bal.data.free.toString(10);
    console.log(`hehe, account ${PUBLIC_ADDR} has balance ${free}`);
}

async function register(api) {
    const keyring = new Keyring({ type: "sr25519" });
    const chao_stash = keyring.addFromUri(SECRET_KEY);
    // registerDevice
    let [err, res] = await to(api.tx.deeperNode.registerDevice("0x1234", 1).signAndSend(chao_stash));
    if (!err && res) {
        console.log(`${chao_stash.address} registered with IP ${ip}`);
        return true;
    }
    console.log(`${chao_stash.address} failed to register: ${err}`);
    return false;
}

async function registerIfNotYet(api) {
    let deviceInfo = await api.query.deeperNode.deviceInfo(PUBLIC_ADDR);
    let pubIP = "22.22.22.22.";
    if (deviceInfo.ipv4 == "0x") {
        isRegistered = await register(api, pubIP);
    } else {
        isRegistered = true;
    }
    console.log(`deviceInfo: ${deviceInfo}`);
}

async function deeperChainThread() {
    console.log("start deeperChainThread...");
    //let api = await getApiInstance(WSS_URL);
    let api = await get_api(WSS_URL);
    await registerIfNotYet(api);
    if (!isRegistered) {
        console.log("Cannot register as device, exit...");
        return;
    }
    console.log(`isRegistered: ${isRegistered}`);
    let unsubBalance = await getBalance(api);
    while (true) {
        await delayPromise(5000);
    }
}

deeperChainThread();
